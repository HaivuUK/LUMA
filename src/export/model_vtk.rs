use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use vtkio::model::{Attribute, Attributes, ByteOrder, CellType, Cells, DataSet, IOBuffer, UnstructuredGridPiece, VertexNumbers, Version};
use vtkio::Vtk;

use crate::mesh::{Element, ElementKind, Mesh};
use crate::params::Params;

#[derive(Debug, Clone, Default)]
pub struct VtkExportFields<'a> {
    pub elemental_density: Option<&'a [f64]>,
    pub youngs_modulus: Option<&'a [f64]>,
}

pub fn export_mesh<P: AsRef<Path>>(path: P, mesh: &Mesh, use_binary: bool) -> Result<()> {
    export_mesh_with_fields(path, mesh, None, use_binary)
}

pub fn write_vtk(mesh: &Mesh, per_part_moduli: &[Vec<f64>], params: &Params, orig_mesh_path: &str, use_binary: bool) -> Result<String> {
    let out_path = vtk_output_path(orig_mesh_path);

    let mut youngs_modulus = Vec::new();
    for (pi, part) in mesh.parts.iter().enumerate() {
        if part.ignore {
            continue;
        }
        if pi < per_part_moduli.len() {
            youngs_modulus.extend_from_slice(&per_part_moduli[pi]);
        }
    }

    let elemental_density: Vec<f64> = if params.back_calculation {
        youngs_modulus
            .iter()
            .map(|&modulus| params.density_back_calculation(modulus))
            .collect()
    } else {
        youngs_modulus.clone()
    };

    let element_count: usize = mesh
        .parts
        .iter()
        .filter(|part| !part.ignore)
        .map(|part| part.elements.len())
        .sum();

    let fields = if elemental_density.len() == element_count && youngs_modulus.len() == element_count {
        Some(VtkExportFields {
            elemental_density: Some(&elemental_density),
            youngs_modulus: Some(&youngs_modulus),
        })
    } else {
        None
    };

    export_mesh_with_fields(&out_path, mesh, fields.as_ref(), use_binary)?;
    Ok(out_path)
}

pub fn export_mesh_with_fields<P: AsRef<Path>>(
    path: P,
    mesh: &Mesh,
    fields: Option<&VtkExportFields<'_>>,
    use_binary: bool,
) -> Result<()> {
    let path_ref = path.as_ref();
    let is_vtu = path_ref
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("vtu"))
        .unwrap_or(false);

    let (points, elements, point_lookup) = flatten_mesh(mesh);
    let cell_count = elements.len();
    let point_count = points.len() / 3;

    let elemental_density = fields.and_then(|value| value.elemental_density);
    let youngs_modulus = fields.and_then(|value| value.youngs_modulus);

    if let Some(values) = elemental_density && values.len() != cell_count {
        return Err(anyhow!(
            "Elemental density count ({}) does not match element count ({})",
            values.len(),
            cell_count
        ));
    }

    if let Some(values) = youngs_modulus && values.len() != cell_count {
        return Err(anyhow!(
            "Young's modulus count ({}) does not match element count ({})",
            values.len(),
            cell_count
        ));
    }

    let mut cell_types = Vec::with_capacity(cell_count);
    let mut cell_vertices = Vec::new();

    for element in &elements {
        let (node_count, cell_type) = element_traits(&element.kind);
        cell_types.push(cell_type);
        cell_vertices.push(node_count as u32);
        for node_id in &element.nodes {
            let point_index = point_lookup
                .get(node_id)
                .context("Element references a node that is missing from the VTK point list")?;
            cell_vertices.push(*point_index as u32);
        }
    }

    let cells = Cells {
        cell_verts: VertexNumbers::Legacy {
            num_cells: cell_count as u32,
            vertices: cell_vertices,
        },
        types: cell_types,
    };

    let mut point_attributes = Attributes::new();
    if let Some(nodal_density) = elemental_density.map(|values| compute_nodal_density(point_count, &elements, values, &point_lookup)) {
        point_attributes.point.push(attribute_scalars("Nodal_Density", nodal_density));
    }

    let mut cell_attributes = Attributes::new();
    if let Some(values) = elemental_density {
        cell_attributes.cell.push(attribute_scalars("Element_Density", values.to_vec()));
    }
    if let Some(values) = youngs_modulus {
        cell_attributes.cell.push(attribute_scalars("Youngs_Modulus", values.to_vec()));
    }

    let piece = UnstructuredGridPiece {
        points: IOBuffer::from(points),
        cells,
        data: Attributes {
            point: point_attributes.point,
            cell: cell_attributes.cell,
        },
    };

    let vtk_output = Vtk {
        version: Version::new(), // with v0.6.3 this was if is_vtu { (2, 0) } else { (4, 2) }),
        title: format!("Exported Mesh with LUMA {}", env!("CARGO_PKG_VERSION")),
        byte_order: if is_vtu {
            ByteOrder::native()
        } else {
            ByteOrder::BigEndian
        },
        data: DataSet::inline(piece),
        file_path: Some(path_ref.to_path_buf()),
    };

    if is_vtu {
        let file = File::create(path_ref).context("Failed to create VTU file")?;
        vtk_output
            .write_xml(BufWriter::new(file))
            .context("Failed to write VTU file")?;
    } else if use_binary {
        vtk_output
            .export_be(path_ref)
            .context("Failed to write binary VTK file")?;
    } else {
        vtk_output
            .export_ascii(path_ref)
            .context("Failed to write ASCII VTK file")?;
    }

    Ok(())
}

fn flatten_mesh(mesh: &Mesh) -> (Vec<f64>, Vec<Element>, HashMap<u32, usize>) {
    let mut points = Vec::new();
    let mut elements = Vec::new();
    let mut point_lookup = HashMap::new();

    for part in &mesh.parts {
        for node in &part.nodes {
            if point_lookup.contains_key(&node.id) {
                continue;
            }
            let index = points.len() / 3;
            point_lookup.insert(node.id, index);
            points.push(node.x);
            points.push(node.y);
            points.push(node.z);
        }

        for element in &part.elements {
            elements.push(element.clone());
        }
    }

    (points, elements, point_lookup)
}

fn compute_nodal_density(
    num_nodes: usize,
    elements: &[Element],
    elemental_density: &[f64],
    point_lookup: &HashMap<u32, usize>,
) -> Vec<f64> {
    let mut nodal_sums = vec![0.0f64; num_nodes];
    let mut nodal_counts = vec![0u32; num_nodes];

    for (element_index, element) in elements.iter().enumerate() {
        let density = elemental_density[element_index];
        for node_id in &element.nodes {
            if let Some(&point_index) = point_lookup.get(node_id) {
                nodal_sums[point_index] += density;
                nodal_counts[point_index] += 1;
            }
        }
    }

    for (sum, count) in nodal_sums.iter_mut().zip(nodal_counts.iter()) {
        if *count > 0 {
            *sum /= f64::from(*count);
        }
    }

    nodal_sums
}

fn element_traits(element_kind: &ElementKind) -> (usize, CellType) {
    match element_kind {
        ElementKind::Tet4 => (4, CellType::Tetra),
        ElementKind::Tet10 => (10, CellType::QuadraticTetra),
        ElementKind::Hex8 => (8, CellType::Hexahedron),
        ElementKind::Wedge6 => (6, CellType::Wedge),
    }
}

fn attribute_scalars(name: &str, values: Vec<f64>) -> Attribute {
    Attribute::scalars(name.to_string(), 1).with_data(IOBuffer::from(values))
}

fn vtk_output_path(orig_mesh_path: &str) -> String {
    let path = Path::new(orig_mesh_path);
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        if ext.eq_ignore_ascii_case("vtu") {
            let stem = &orig_mesh_path[..orig_mesh_path.len() - ext.len() - 1];
            return format!("{}MAT.vtu", stem);
        }
        if ext.eq_ignore_ascii_case("vtk") {
            let stem = &orig_mesh_path[..orig_mesh_path.len() - ext.len() - 1];
            return format!("{}MAT.vtk", stem);
        }
    }
    format!("{orig_mesh_path}.MAT.vtk")
}