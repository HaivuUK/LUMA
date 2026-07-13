use super::{MeshVisualisationData, MeshBounds, VisualisationConfig};
use crate::export::histogram::HistogramData;
use crate::mesh::{Mesh, ElementKind};
use anyhow::Result;
use palette::{Hsv, Srgb, FromColor};
use std::fs;
use std::collections::HashMap;

/// Create visualisation data from processed mesh with material assignments
pub fn create_visualisation_data(
    mesh: &Mesh,
    material_data: &[Vec<f64>],
    config: &VisualisationConfig,
    histogram: Option<HistogramData>,
) -> Result<MeshVisualisationData> {
    println!("Creating visualisation data...");
    
    // Find material value range for color mapping
    let material_range = find_material_range(material_data);
    println!("Material range: {:.3} to {:.3}", material_range.0, material_range.1);
    
    // Pre-compute color mapping parameters for efficiency
    let color_scale = if material_range.1 > material_range.0 {
        1.0 / (material_range.1 - material_range.0)
    } else {
        0.0
    };
    let color_offset = material_range.0;
    
    let mut total_elements = 0;
    let mut total_faces_estimate = 0;
    
    // Count total elements and estimate faces
    for (part_idx, part) in mesh.parts.iter().enumerate() {
        if !part.ignore && part_idx < material_data.len() {
            let elements = part.elements.len().min(material_data[part_idx].len());
            let sampled_elements = (elements as f32 * config.element_sampling) as usize;
            total_elements += sampled_elements;
            // Estimate faces: Tet4=4, Tet10=4, Hex8=12, Wedge6=8
            total_faces_estimate += sampled_elements * 8; // Use average estimate
        }
    }
    
    if config.element_sampling < 1.0 {
        println!("Using element sampling ratio: {:.2} ({} sampled elements)", 
                config.element_sampling, total_elements);
    }
    
    println!("Processing {} elements from {} parts...", total_elements, mesh.parts.len());
    
    let mut vertices = Vec::with_capacity(total_faces_estimate * 3 * 3);
    let mut faces = Vec::with_capacity(total_faces_estimate * 3);
    let mut colors = Vec::with_capacity(total_faces_estimate * 3 * 3);
    let mut vertex_values = Vec::with_capacity(total_faces_estimate * 3);
    let mut next_vertex_id = 0;

    let mut processed_elements = 0;
    let progress_interval = (total_elements / 10).max(1);

    // Process each part
    for (part_idx, part) in mesh.parts.iter().enumerate() {
        if part.ignore || part_idx >= material_data.len() {
            continue;
        }
        
        let part_materials = &material_data[part_idx];
        let part_elements = part.elements.len().min(part_materials.len());
        println!("Processing part {}: {} elements", part_idx, part_elements);
        
        // Create node ID to index mapping for this part for O(1) lookup
        let node_lookup: HashMap<u32, usize> = part.nodes.iter()
            .enumerate()
            .map(|(idx, node)| (node.id, idx))
            .collect();
        
        // Calculate sampling step for this part
        let sampling_step = if config.element_sampling >= 1.0 { 
            1 
        } else { 
            (1.0 / config.element_sampling).max(1.0) as usize 
        };
        
        // Process each element in the part with sampling
        for (elem_idx, element) in part.elements.iter().enumerate().step_by(sampling_step) {
            if elem_idx >= part_materials.len() {
                continue;
            }
            if processed_elements % progress_interval == 0 {
                let progress = (processed_elements as f32 / total_elements as f32 * 100.0) as u32;
                println!("Processed {}% ({}/{} elements)...", progress, processed_elements, total_elements);
            }
            
            let material_value = part_materials[elem_idx];
            let element_color = material_to_color_fast(material_value, color_offset, color_scale);

            // Get tessellated faces for this element  
            let face_templates = get_element_face_templates(&element.kind);

            for face_template in face_templates {
                let mut valid_face = true;
                let mut face_positions = [[0f32; 3]; 3];
                
                for (i, &node_idx) in face_template.iter().enumerate() {
                    let node_id = element.nodes[node_idx as usize];
                    
                    // Fast O(1) node lookup using pre-built HashMap
                    let node = match node_lookup.get(&node_id).and_then(|&idx| part.nodes.get(idx)) {
                        Some(n) => n,
                        None => {
                            valid_face = false;
                            break;
                        }
                    };
                    face_positions[i] = [node.x as f32, node.y as f32, node.z as f32];
                }
                
                if valid_face {
                    for face_position in &face_positions {
                        vertices.push(face_position[0]);
                        vertices.push(face_position[1]);
                        vertices.push(face_position[2]);
                        colors.push(element_color[0]);
                        colors.push(element_color[1]);
                        colors.push(element_color[2]);
                        vertex_values.push(material_value as f32);
                        faces.push(next_vertex_id);
                        next_vertex_id += 1;
                    }
                }
            }
            
            processed_elements += 1;
        }
    }
    
    // Calculate bounds
    let bounds = calculate_bounds(&vertices);
    
    println!("Visualisation data created: {} vertices, {} faces", vertices.len(), faces.len());
    println!("Mesh bounds: X=[{:.2}, {:.2}], Y=[{:.2}, {:.2}], Z=[{:.2}, {:.2}]", 
             bounds.min_x, bounds.max_x, bounds.min_y, bounds.max_y, bounds.min_z, bounds.max_z);
    
    Ok(MeshVisualisationData {
        vertices,
        faces,
        colors,
        vertex_values,
        bounds,
        material_range,
        histogram,
    })
}

/// Static tessellation function for better performance (avoids Result allocation)
fn get_element_face_templates(kind: &ElementKind) -> &'static [[u32; 3]] {
    match kind {
        ElementKind::Tet4 | ElementKind::Tet10 => {
            // Tetrahedron: 4 triangular faces (use only corner nodes for Tet10)
            &[
                [0, 1, 2], // Use indices instead of node IDs for now
                [0, 1, 3],
                [1, 2, 3],
                [0, 2, 3],
            ]
        },
        ElementKind::Hex8 => {
            // Hexahedron: 12 triangular faces (2 per quad face)
            &[
                // Bottom face
                [0, 1, 2], [0, 2, 3],
                // Top face  
                [4, 6, 5], [4, 7, 6],
                // Front face
                [0, 4, 5], [0, 5, 1],
                // Back face
                [2, 6, 7], [2, 7, 3],
                // Left face
                [0, 3, 7], [0, 7, 4],
                // Right face
                [1, 5, 6], [1, 6, 2],
            ]
        },
        ElementKind::Wedge6 => {
            // Wedge/Prism: 8 triangular faces
            &[
                // Bottom triangular face
                [0, 1, 2],
                // Top triangular face
                [3, 5, 4],
                // Side faces
                [0, 3, 4], [0, 4, 1],
                [1, 4, 5], [1, 5, 2],
                [2, 5, 3], [2, 3, 0],
            ]
        },
    }
}

/// Find the range of material values for color mapping
fn find_material_range(material_data: &[Vec<f64>]) -> (f64, f64) {
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    
    for part_data in material_data {
        for &value in part_data {
            if value.is_finite() {
                min_val = min_val.min(value);
                max_val = max_val.max(value);
            }
        }
    }
    
    if min_val.is_infinite() || max_val.is_infinite() {
        (0.0, 1.0) // Default range
    } else {
        (min_val, max_val)
    }
}

/// Fast color conversion using precomputed parameters
fn material_to_color_fast(value: f64, offset: f64, scale: f64) -> [f32; 3] {
    if !value.is_finite() {
        return [0.5, 0.5, 0.5]; // Gray for invalid values
    }
    
    // Normalise to [0, 1] using precomputed scale and offset
    let normalised = if scale > 0.0 {
        ((value - offset) * scale).clamp(0.0, 1.0)
    } else {
        0.5
    };
    
    // Use HSV color space: blue (240°) for low values to red (0°) for high values
    let hue = (1.0 - normalised) * 240.0;
    let hsv = Hsv::new(hue as f32, 1.0f32, 1.0f32);
    let rgb: Srgb<f32> = Srgb::from_color(hsv);
    
    [rgb.red, rgb.green, rgb.blue]
}

/// Calculate bounding box of vertices
fn calculate_bounds(vertices: &[f32]) -> MeshBounds {
    if vertices.is_empty() {
        return MeshBounds {
            min_x: 0.0, max_x: 0.0,
            min_y: 0.0, max_y: 0.0,
            min_z: 0.0, max_z: 0.0,
        };
    }
    
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;

    for i in (0..vertices.len()).step_by(3) {
        let x = vertices[i];
        let y = vertices[i+1];
        let z = vertices[i+2];

        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }
    
    MeshBounds {
        min_x, max_x,
        min_y, max_y,
        min_z, max_z,
    }
}

/// Export visualisation data
pub fn export_visualisation(data: &MeshVisualisationData, export_dir: &str) -> Result<()> {
    fs::create_dir_all(export_dir)?;
    
    // Export as simple OBJ file with colors as vertex attributes
    let obj_path = std::path::Path::new(export_dir).join("mesh_visualisation.obj");
    let mut obj_content = String::new();
    
    // Write vertices with colors as comments
    for i in (0..data.vertices.len()).step_by(3) {
        obj_content.push_str(&format!(
            "v {} {} {} # color: {:.3} {:.3} {:.3}\n",
            data.vertices[i], data.vertices[i+1], data.vertices[i+2],
            data.colors[i], data.colors[i+1], data.colors[i+2]
        ));
    }
    
    // Write faces (OBJ uses 1-based indexing)
    for i in (0..data.faces.len()).step_by(3) {
        obj_content.push_str(&format!(
            "f {} {} {}\n",
            data.faces[i] + 1, data.faces[i+1] + 1, data.faces[i+2] + 1
        ));
    }
    
    fs::write(&obj_path, obj_content)?;
    
    // Export metadata as JSON
    let meta_path = std::path::Path::new(export_dir).join("metadata.json");
    let metadata = serde_json::json!({
        "vertex_count": data.vertices.len() / 3,
        "face_count": data.faces.len() / 3,
        "material_range": {
            "min": data.material_range.0,
            "max": data.material_range.1
        },
        "bounds": {
            "min_x": data.bounds.min_x,
            "max_x": data.bounds.max_x,
            "min_y": data.bounds.min_y,
            "max_y": data.bounds.max_y,
            "min_z": data.bounds.min_z,
            "max_z": data.bounds.max_z
        },
        "histogram": data.histogram
    });
    
    fs::write(&meta_path, serde_json::to_string_pretty(&metadata)?)?;
    
    Ok(())
}