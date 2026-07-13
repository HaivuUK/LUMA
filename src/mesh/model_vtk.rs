use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use vtkio::model::{Attribute, CellType, DataSet, IOBuffer, VertexNumbers};
use vtkio::Vtk;

use super::{Element, ElementKind, Mesh, Node, Part};

#[derive(Debug, Clone, Default)]
pub struct VtkImportedFields {
    pub nodal_density: Option<Vec<f64>>,
    pub elemental_density: Option<Vec<f64>>,
    pub youngs_modulus: Option<Vec<f64>>,
}

pub fn import_vtk_mesh<P: AsRef<Path>>(path: P) -> Result<Mesh> {
    import_vtk_mesh_with_fields(path).map(|(mesh, _)| mesh)
}

pub fn import_vtk_mesh_with_fields<P: AsRef<Path>>(path: P) -> Result<(Mesh, VtkImportedFields)> {
    let mut vtk_file = Vtk::import(path.as_ref()).context("Failed to read VTK file")?;
    vtk_file.load_all_pieces().context("Failed to load VTK pieces")?;

    match vtk_file.data {
        DataSet::UnstructuredGrid { pieces, .. } => {
            let mut nodes = Vec::new();
            let mut elements = Vec::new();
            let mut nodal_density: Option<Vec<f64>> = Some(Vec::new());
            let mut elemental_density: Option<Vec<f64>> = Some(Vec::new());
            let mut youngs_modulus: Option<Vec<f64>> = Some(Vec::new());

            for piece in pieces {
                let piece = piece
                    .into_loaded_piece_data(vtk_file.file_path.as_deref())
                    .context("Failed to load VTK piece data")?;

                let piece_points = buffer_to_f64_vec(piece.points)
                    .context("Failed to read VTK point coordinates")?;
                if piece_points.len() % 3 != 0 {
                    return Err(anyhow!("VTK point array does not contain xyz triplets"));
                }

                let point_count = piece_points.len() / 3;
                let node_offset = nodes.len() as u32;
                for (idx, coords) in piece_points.chunks_exact(3).enumerate() {
                    nodes.push(Node {
                        id: node_offset + idx as u32,
                        x: coords[0],
                        y: coords[1],
                        z: coords[2],
                    });
                }

                let cell_connectivity = decode_cell_connectivity(&piece.cells.cell_verts)
                    .context("Failed to decode VTK cell connectivity")?;
                if cell_connectivity.len() != piece.cells.types.len() {
                    return Err(anyhow!("VTK cell connectivity count does not match cell type count"));
                }

                for (cell_nodes, cell_type) in cell_connectivity.into_iter().zip(piece.cells.types.iter()) {
                    let kind = vtk_cell_type_to_element_kind(*cell_type)?;
                    let expected_nodes = expected_nodes_per_element(&kind);
                    if cell_nodes.len() != expected_nodes {
                        return Err(anyhow!(
                            "VTK cell type {:?} expected {} nodes but found {}",
                            cell_type,
                            expected_nodes,
                            cell_nodes.len()
                        ));
                    }

                    elements.push(Element {
                        id: elements.len() as u32,
                        nodes: cell_nodes.into_iter().map(|node_id| node_id + node_offset).collect(),
                        kind,
                    });
                }

                append_scalar_field(
                    &mut nodal_density,
                    extract_scalar_field(&piece.data.point, &["Nodal_Density", "nodal_density", "density"]),
                    point_count,
                );
                append_scalar_field(
                    &mut elemental_density,
                    extract_scalar_field(&piece.data.cell, &["Element_Density", "element_density", "density"]),
                    piece.cells.types.len(),
                );
                append_scalar_field(
                    &mut youngs_modulus,
                    extract_scalar_field(&piece.data.cell, &["Youngs_Modulus", "YoungsModulus", "youngs_modulus"]),
                    piece.cells.types.len(),
                );
            }

            let mut node_index = HashMap::with_capacity(nodes.len());
            for (idx, node) in nodes.iter().enumerate() {
                node_index.insert(node.id, idx);
            }

            Ok((
                Mesh {
                    parts: vec![Part {
                        name: Some("VTK".to_string()),
                        elements,
                        nodes,
                        node_index,
                        ignore: false,
                    }],
                    mesh_format_info: None,
                },
                VtkImportedFields {
                    nodal_density,
                    elemental_density,
                    youngs_modulus,
                },
            ))
        }
        _ => Err(anyhow!("Only UnstructuredGrid (.vtk/.vtu) files are supported")),
    }
}

fn buffer_to_f64_vec(buffer: IOBuffer) -> Option<Vec<f64>> {
    let values: Option<Vec<f64>> = buffer.clone().into();
    if values.is_some() {
        return values;
    }

    let values_f32: Option<Vec<f32>> = buffer.into();
    values_f32.map(|values| values.into_iter().map(f64::from).collect())
}

fn decode_cell_connectivity(cell_verts: &VertexNumbers) -> Result<Vec<Vec<u32>>> {
    match cell_verts {
        VertexNumbers::Legacy { num_cells, vertices } => {
            let mut cells = Vec::with_capacity(*num_cells as usize);
            let mut index = 0usize;
            for _ in 0..*num_cells as usize {
                let node_count = *vertices.get(index).context("Legacy VTK cell is missing its node count")? as usize;
                index += 1;
                let end = index + node_count;
                let cell = vertices
                    .get(index..end)
                    .context("Legacy VTK cell connectivity is truncated")?
                    .to_vec();
                cells.push(cell);
                index = end;
            }
            Ok(cells)
        }
        VertexNumbers::XML { connectivity, offsets } => {
            let mut cells = Vec::with_capacity(offsets.len());
            let mut start = 0usize;
            for &end in offsets {
                let end = usize::try_from(end).context("VTK cell offset does not fit into usize")?;
                let cell = connectivity
                    .get(start..end)
                    .context("XML VTK cell connectivity is truncated")?
                    .iter()
                    .map(|&value| u32::try_from(value).context("VTK point index does not fit into u32"))
                    .collect::<Result<Vec<u32>>>()?;
                cells.push(cell);
                start = end;
            }
            Ok(cells)
        }
    }
}

fn vtk_cell_type_to_element_kind(cell_type: CellType) -> Result<ElementKind> {
    match cell_type {
        CellType::Tetra => Ok(ElementKind::Tet4),
        CellType::QuadraticTetra => Ok(ElementKind::Tet10),
        CellType::Hexahedron => Ok(ElementKind::Hex8),
        CellType::Wedge => Ok(ElementKind::Wedge6),
        other => Err(anyhow!("Unsupported cell type: {:?}", other)),
    }
}

fn expected_nodes_per_element(kind: &ElementKind) -> usize {
    match kind {
        ElementKind::Tet4 => 4,
        ElementKind::Tet10 => 10,
        ElementKind::Hex8 => 8,
        ElementKind::Wedge6 => 6,
    }
}

fn extract_scalar_field(attributes: &[Attribute], wanted_names: &[&str]) -> Option<Vec<f64>> {
    for wanted in wanted_names {
        if let Some(values) = attributes
            .iter()
            .find(|attribute| attribute.name().eq_ignore_ascii_case(wanted))
            .and_then(attribute_to_f64_values)
        {
            return Some(values);
        }
    }

    attributes.iter().find_map(attribute_to_f64_values)
}

fn attribute_to_f64_values(attribute: &Attribute) -> Option<Vec<f64>> {
    match attribute {
        Attribute::DataArray(data_array) => {
            if data_array.elem.num_comp() != 1 {
                return None;
            }

            let values: Option<Vec<f64>> = data_array.data.clone().into();
            if values.is_some() {
                return values;
            }

            let values_f32: Option<Vec<f32>> = data_array.data.clone().into();
            values_f32.map(|values| values.into_iter().map(f64::from).collect())
        }
        Attribute::Field { .. } => None,
    }
}

fn append_scalar_field(target: &mut Option<Vec<f64>>, values: Option<Vec<f64>>, expected_len: usize) {
    match values {
        Some(values) if values.len() == expected_len => {
            if let Some(target_values) = target.as_mut() {
                target_values.extend(values);
            }
        }
        _ => {
            *target = None;
        }
    }
}