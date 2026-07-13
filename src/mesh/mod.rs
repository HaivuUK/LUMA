use anyhow::{Result, anyhow};

pub mod abaqus_inp;
pub mod ansys_cdb;
pub mod model_vtk;
pub mod febio_feb;

#[derive(Debug, Clone)]
pub struct Node { pub id: u32, pub x: f64, pub y: f64, pub z: f64 }

#[derive(Debug, Clone)]
pub enum ElementKind { Tet4, Tet10, Hex8, Wedge6 }

#[derive(Debug, Clone)]
pub struct Element { pub id: u32, pub nodes: Vec<u32>, pub kind: ElementKind }

#[derive(Debug, Clone, Default)]
pub struct MeshFormatInfo {
    pub nblock_format: Option<String>,
    pub eblock_format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Part { 
    pub name: Option<String>,
    pub elements: Vec<Element>, 
    pub nodes: Vec<Node>, 
    pub node_index: std::collections::HashMap<u32, usize>, 
    pub ignore: bool 
}

#[derive(Debug, Clone)]
pub struct Mesh { 
    pub parts: Vec<Part>,
    pub mesh_format_info: Option<MeshFormatInfo>,
}

impl Mesh {
	/// Apply transformation (translation and rotation) to all nodes in the mesh
	pub fn apply_transformation(&mut self,
		translate_x: Option<f64>,
		translate_y: Option<f64>,
		translate_z: Option<f64>,
		rotate_x_deg: Option<f64>,
		rotate_y_deg: Option<f64>,
		rotate_z_deg: Option<f64>) {

		// Convert rotation angles from degrees to radians
		let rotate_x = rotate_x_deg.map(|deg| deg.to_radians());
		let rotate_y = rotate_y_deg.map(|deg| deg.to_radians());
		let rotate_z = rotate_z_deg.map(|deg| deg.to_radians());

		// Apply transformations to all parts
		for part in &mut self.parts {
			for node in &mut part.nodes {
				// Apply rotation first (around origin)
				if let Some(rx) = rotate_x {
					let y = node.y * rx.cos() - node.z * rx.sin();
					let z = node.y * rx.sin() + node.z * rx.cos();
					node.y = y;
					node.z = z;
				}

				if let Some(ry) = rotate_y {
					let x = node.x * ry.cos() + node.z * ry.sin();
					let z = -node.x * ry.sin() + node.z * ry.cos();
					node.x = x;
					node.z = z;
				}

				if let Some(rz) = rotate_z {
					let x = node.x * rz.cos() - node.y * rz.sin();
					let y = node.x * rz.sin() + node.y * rz.cos();
					node.x = x;
					node.y = y;
				}

				// Apply translation
				if let Some(tx) = translate_x {
					node.x += tx;
				}
				if let Some(ty) = translate_y {
					node.y += ty;
				}
				if let Some(tz) = translate_z {
					node.z += tz;
				}
			}
		}
	}
}

pub fn load_mesh(path: &str, ignore: &[String]) -> Result<Mesh> {
	if has_extension(path, "inp") { 
		abaqus_inp::parse_inp(path, ignore)
	} else if has_extension(path, "cdb") {
		ansys_cdb::parse_cdb(path, ignore)
	} else if has_extension(path, "vtk") || has_extension(path, "vtu") {
		model_vtk::import_vtk_mesh(path)
	} else if has_extension(path, "feb") {
		febio_feb::parse_feb(path, ignore)
	} else { 
		Err(anyhow!("Only .inp, .cdb, .vtk, .vtu, and .feb files supported")) 
	}
}

fn has_extension(path: &str, expected: &str) -> bool {
	std::path::Path::new(path)
		.extension()
		.and_then(|ext| ext.to_str())
		.map(|ext| ext.eq_ignore_ascii_case(expected))
		.unwrap_or(false)
}

pub fn load_mesh_with_transformation(path: &str, ignore: &[String], params: &crate::params::Params, transformation_override: Option<&crate::MeshTransformation>) -> Result<Mesh> {
	let mut mesh = load_mesh(path, ignore)?;
	
	// Use transformation override if provided, otherwise use parameters from file
	let (translate_x, translate_y, translate_z, rotate_x, rotate_y, rotate_z) = if let Some(transform) = transformation_override {
		(transform.translate_x, transform.translate_y, transform.translate_z,
		 transform.rotate_x, transform.rotate_y, transform.rotate_z)
	} else {
		(params.mesh_translate_x, params.mesh_translate_y, params.mesh_translate_z,
		 params.mesh_rotate_x, params.mesh_rotate_y, params.mesh_rotate_z)
	};
	
	// Apply transformations if any are specified
	let has_transformations = translate_x.is_some() || 
		translate_y.is_some() || 
		translate_z.is_some() ||
		rotate_x.is_some() || 
		rotate_y.is_some() || 
		rotate_z.is_some();
	
	if has_transformations {
                crate::log_status("Applying mesh transformations...");
                mesh.apply_transformation(
			translate_x,
			translate_y,
			translate_z,
			rotate_x,
			rotate_y,
			rotate_z
		);
		
		let transforms = vec![
			translate_x.map(|v| format!("translate X: {}", v)),
			translate_y.map(|v| format!("translate Y: {}", v)),
			translate_z.map(|v| format!("translate Z: {}", v)),
			rotate_x.map(|v| format!("rotate X: {}°", v)),
			rotate_y.map(|v| format!("rotate Y: {}°", v)),
			rotate_z.map(|v| format!("rotate Z: {}°", v)),
		].into_iter().flatten().collect::<Vec<_>>();
		
		crate::log_success(&format!("Applied transformations: {}", transforms.join(", ")));
        }

        Ok(mesh)
}

pub fn load_mesh_with_materials(path: &str) -> Result<(Mesh, Vec<Vec<f64>>)> {
	if has_extension(path, "vtk") || has_extension(path, "vtu") {
		let (mesh, fields) = model_vtk::import_vtk_mesh_with_fields(path)?;
		crate::log_status("Parsing material assignments from VTK scalar fields...");

		let mut per_part = vec![vec![0.0; 0]; mesh.parts.len()];
		for (i, p) in mesh.parts.iter().enumerate() {
			per_part[i] = vec![0.0; p.elements.len()];
		}

		let values = if let Some(values) = fields.youngs_modulus {
			values
		} else if let Some(values) = fields.elemental_density {
			values
		} else {
			return Err(anyhow!(
				"VTK mesh does not contain 'Youngs_Modulus' or 'Element_Density' cell data for visualisation"
			));
		};

		let expected: usize = mesh.parts.iter().map(|p| p.elements.len()).sum();
		if values.len() != expected {
			return Err(anyhow!(
				"VTK scalar field length ({}) does not match element count ({})",
				values.len(),
				expected
			));
		}

		let mut offset = 0usize;
		for (pi, part) in mesh.parts.iter().enumerate() {
			let count = part.elements.len();
			per_part[pi].copy_from_slice(&values[offset..offset + count]);
			offset += count;
		}

		crate::log_status(&format!(
			"Loaded VTK material field for visualisation: {} elements",
			expected
		));
		return Ok((mesh, per_part));
	}

    let mesh = load_mesh(path, &[])?;
    crate::log_status("Parsing material assignments from mesh file...");
    let content = std::fs::read_to_string(path)?;
    let mut per_part = vec![vec![0.0; 0]; mesh.parts.len()];
    for (i, p) in mesh.parts.iter().enumerate() {
        per_part[i] = vec![0.0; p.elements.len()];
    }

	if has_extension(path, "inp") {
		// Parse Abaqus materials and element assignments in a format-agnostic way.
		// We do not rely on a specific material name prefix or a specific section keyword.
		let lines: Vec<&str> = content.lines().collect();
		let mut mat_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
		let mut current_mat = String::new();
		let mut parsing_elastic = false;
		let mut elset_members: std::collections::HashMap<String, Vec<u32>> = std::collections::HashMap::new();
		let mut section_bindings: Vec<(String, String)> = Vec::new(); // (elset, material)

		let mut i = 0;
		while i < lines.len() {
			let l = lines[i].trim();
			let l_lower = l.to_ascii_lowercase();

			if l.starts_with("**") {
				i += 1;
				continue;
			}

			if l_lower.starts_with("*material") {
				if let Some(name) = abaqus_inp::extract_kwarg(l, "name") {
					current_mat = name;
				}
				parsing_elastic = false;
				i += 1;
				continue;
			}

			if l_lower.starts_with("*elastic") {
				parsing_elastic = true;
				i += 1;
				continue;
			}

			if l_lower.starts_with("*elset") {
				let set_name = abaqus_inp::extract_kwarg(l, "elset");
				let is_generate = l_lower.split(',').any(|tok| tok.trim() == "generate");
				i += 1;

				if let Some(name) = set_name {
					let ids = elset_members.entry(name).or_default();
					while i < lines.len() {
						let data_line = lines[i].trim();
						if abaqus_inp::is_abaqus_keyword_line(data_line) {
							break;
						}
						if data_line.starts_with("**") {
							i += 1;
							continue;
						}
						if !data_line.is_empty() {
							abaqus_inp::parse_elset_data_line(data_line, is_generate, ids);
						}
						i += 1;
					}
				} else {
					while i < lines.len() {
						if abaqus_inp::is_abaqus_keyword_line(lines[i].trim()) {
							break;
						}
						i += 1;
					}
				}
				continue;
			}

			if l_lower.starts_with("*element") {
				let set_name = abaqus_inp::extract_kwarg(l, "elset");
				i += 1;
				if let Some(name) = set_name {
					let ids = elset_members.entry(name).or_default();
					while i < lines.len() {
						let data_line = lines[i].trim();
						if abaqus_inp::is_abaqus_keyword_line(data_line) {
							break;
						}
						if data_line.starts_with("**") {
							i += 1;
							continue;
						}
						if let Some(eid) = abaqus_inp::parse_element_id_from_line(data_line) {
							ids.push(eid);
						}
						i += 1;
					}
				} else {
					while i < lines.len() {
						if abaqus_inp::is_abaqus_keyword_line(lines[i].trim()) {
							break;
						}
						i += 1;
					}
				}
				continue;
			}

			if abaqus_inp::is_abaqus_keyword_line(l) {
				parsing_elastic = false;
				if let (Some(elset), Some(material)) = (abaqus_inp::extract_kwarg(l, "elset"), abaqus_inp::extract_kwarg(l, "material")) {
					section_bindings.push((elset, material));
				}
				i += 1;
				continue;
			}

			if parsing_elastic {
				if let Some(val_str) = l.split(',').next()
					&& let Ok(modulus) = val_str.trim().parse::<f64>()
						&& !current_mat.is_empty() {
							mat_map.insert(current_mat.clone(), modulus);
						}

				parsing_elastic = false;
			}
			i += 1;
		}

		// Element target is global index, so we need to map element id -> (part_idx, element_idx)
		let mut el_to_part = std::collections::HashMap::new();
		for (pi, part) in mesh.parts.iter().enumerate() {
			for (ei, el) in part.elements.iter().enumerate() {
				el_to_part.insert(el.id, (pi, ei));
			}
		}

		let mut assigned_count = 0usize;

		// Primary assignment path: keyword lines with both elset= and material=.
		for (elset_name, material_name) in &section_bindings {
			if let Some(&modulus) = abaqus_inp::get_material_modulus_case_insensitive(&mat_map, material_name)
				&& let Some(ids) = abaqus_inp::get_elset_ids_case_insensitive(&elset_members, elset_name) {
					for &eid in ids {
						if let Some(&(pi, ei)) = el_to_part.get(&eid) {
							per_part[pi][ei] = modulus;
							assigned_count += 1;
						}
					}
				}
		}

		// Legacy fallback for old files that encode material in the elset name.
		if assigned_count == 0 {
			for (set_name, ids) in &elset_members {
				let set_lower = set_name.to_ascii_lowercase();
				if set_lower.contains("bonemat") {
					let guessed_material = set_name
						.replace("Set_", "")
						.replace("SET_", "")
						.replace("set_", "");
					if let Some(&modulus) = abaqus_inp::get_material_modulus_case_insensitive(&mat_map, &guessed_material) {
						for &eid in ids {
							if let Some(&(pi, ei)) = el_to_part.get(&eid) {
								per_part[pi][ei] = modulus;
								assigned_count += 1;
							}
						}
					}
				}
			}
		}

		crate::log_status(&format!(
			"Parsed Abaqus materials: {} materials, {} elsets, {} section bindings, {} assigned elements",
			mat_map.len(),
			elset_members.len(),
			section_bindings.len(),
			assigned_count
		));
	} else if has_extension(path, "cdb") {
        let mut mat_map = std::collections::HashMap::new();
        // Parse MPDATA
        for line in content.lines() {
            let l = line.trim();
            if l.to_ascii_lowercase().starts_with("mpdata") && l.to_ascii_lowercase().contains("ex") {
                // MPDATA,R5.0, 1,EX,     1, 1, 14500.0
                let parts: Vec<&str> = l.split(',').collect();
                if parts.len() >= 7
                    && let Ok(mat_id) = parts[4].trim().parse::<u32>()
                        && let Ok(modulus) = parts[6].trim().parse::<f64>() {
                            mat_map.insert(mat_id, modulus);
                        }
            }
        }

        let mut el_to_part = std::collections::HashMap::new();
        for (pi, part) in mesh.parts.iter().enumerate() {
            for (ei, el) in part.elements.iter().enumerate() {
                el_to_part.insert(el.id, (pi, ei));
            }
        }

        // Parse elements block from cdb. Need to know format width
        // Parse lines that look like element definitions in EBLOCK
        let mut in_eblock = false;
        for line in content.lines() {
            let l = line.trim();
            if l.starts_with("EBLOCK") {
                in_eblock = true;
            } else if l == "-1" {
                in_eblock = false;
            } else if in_eblock {
                if l.starts_with('(') || l.is_empty() { continue; }
                // EBLOCK lines might be formatted without spaces between large numbers.
                // Assuming it has spaces or we split by standard lengths if needed.
                // For simplicity, attempt standard split by whitespace:
                let tokens: Vec<&str> = l.split_whitespace().collect();
                if tokens.len() > 10
                    && let Ok(mat_id) = tokens[0].parse::<u32>()
                        && let Ok(eid) = tokens[10].parse::<u32>()
                            && let Some(&(pi, ei)) = el_to_part.get(&eid) {
                                per_part[pi][ei] = *mat_map.get(&mat_id).unwrap_or(&0.0);
                            }
            }
        }
	} else if has_extension(path, "feb") {
		crate::log_status("Parsing FEBIO materials and element assignments...");
		let (mesh, per_part_moduli) = febio_feb::parse_feb_with_materials(path, &[])?;
		crate::log_status(&format!(
			"Parsed FEBIO materials: {} parts, {} assigned elements",
			mesh.parts.len(),
			per_part_moduli.iter().map(|v| v.len()).sum::<usize>()
		));
		return Ok((mesh, per_part_moduli));
	}

    Ok((mesh, per_part))
}