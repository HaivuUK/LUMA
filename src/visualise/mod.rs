pub mod mesh_visualiser;
pub mod web_viewer;

use crate::mesh::Mesh;
use crate::params::Params;
use crate::volume::Volume;
use crate::export::histogram::HistogramData;
use anyhow::Result;

/// Configuration for visualisation generation
#[derive(Debug, Clone)]
pub struct VisualisationConfig {
    /// Resolution of each viewport
    pub viewport_resolution: (u32, u32),
    /// Whether to launch web viewer automatically
    pub auto_open: bool,
    /// Export directory for generated images
    pub export_dir: Option<String>,
    /// Element sampling ratio (1.0 = all elements, 0.5 = every other element, etc.)
    pub element_sampling: f32,
}

impl Default for VisualisationConfig {
    fn default() -> Self {
        Self {
            viewport_resolution: (512, 512),
            auto_open: true,
            export_dir: None,
            element_sampling: 1.0,
        }
    }
}

/// Generate 3D visualisation of material distribution on processed mesh
pub fn generate_visualisation(
    mesh: &Mesh,
    material_data: &[Vec<f64>],
    _params: &Params,
    config: &VisualisationConfig,
    histogram: Option<HistogramData>,
) -> Result<()> {
    crate::log_status("Generating 3D mesh visualisation...");
    
    // Create mesh visualisation data with material assignments
    let viz_data = mesh_visualiser::create_visualisation_data(mesh, material_data, config, histogram)?;

    // Start web viewer if requested

    {
        if config.auto_open {
            web_viewer::start_viewer(&viz_data, config)?;
        }
    }
    
    // Export data if directory is specified
    if let Some(export_dir) = &config.export_dir {
        mesh_visualiser::export_visualisation(&viz_data, export_dir)?;
        crate::log_success(&format!("Visualisation data exported to: {}", export_dir));
    }
    
    Ok(())
}

/// Generate 3D visualisation of material distribution on a model that already has material assignments
pub fn generate_visualisation_from_assigned(
    mesh: &Mesh,
    material_data: &[Vec<f64>],
    config: &VisualisationConfig,
) -> Result<()> {
    crate::log_status("Generating 3D mesh visualisation...");

    // Build the histogram from the assigned materials directly
    let mut all_assigned_moduli = Vec::new();
    for (pi, part) in mesh.parts.iter().enumerate() {
        if !part.ignore && pi < material_data.len() {
            all_assigned_moduli.extend(&material_data[pi]);
        }
    }
    let histogram = if !all_assigned_moduli.is_empty() {
        Some(crate::export::histogram::build_assigned_material_histogram(&all_assigned_moduli))
    } else {
        None
    };

    let viz_data = mesh_visualiser::create_visualisation_data(mesh, material_data, config, histogram)?;

    if config.auto_open {
        web_viewer::start_viewer(&viz_data, config)?;
    }

    if let Some(export_dir) = &config.export_dir {
        mesh_visualiser::export_visualisation(&viz_data, export_dir)?;
        crate::log_success(&format!("Visualisation data exported to: {}", export_dir));
    }

    Ok(())
}

/// Generate combined mesh and CT visualisation without running the full pipeline
/// This function loads mesh and CT data independently for direct visualisation
pub fn visualise_mesh_with_ct(
    mesh_path: &str,
    ct_path: &str,
    config: &VisualisationConfig,
) -> Result<()> {
    crate::log_status("Loading mesh for visualisation...");
    
    // Load mesh without material processing
    let mesh = crate::mesh::load_mesh(mesh_path, &Vec::new())?;
    crate::log_success(&format!("Loaded mesh with {} parts", mesh.parts.len()));
    
    crate::log_status("Loading CT volume...");
    let volume = crate::volume::load_volume(ct_path)?;
    crate::log_success("CT volume loaded successfully");
    
    // Create dummy material data for mesh visualisation (uniform color)
    let material_data: Vec<Vec<f64>> = mesh.parts.iter()
        .map(|part| vec![1.0; part.elements.len()])
        .collect();
    
    // Create mesh visualisation data
    let mesh_viz = mesh_visualiser::create_visualisation_data(&mesh, &material_data, config, None)?;

    // Extract CT slices at middle positions
    let ct_bounds = CtBounds {
        min_x: volume.x[0],
        max_x: volume.x[volume.x.len() - 1],
        min_y: volume.y[0],
        max_y: volume.y[volume.y.len() - 1],
        min_z: volume.z[0],
        max_z: volume.z[volume.z.len() - 1],
    };
    
    println!("CT bounds: X=[{:.2}, {:.2}], Y=[{:.2}, {:.2}], Z=[{:.2}, {:.2}]", 
             ct_bounds.min_x, ct_bounds.max_x, ct_bounds.min_y, ct_bounds.max_y, ct_bounds.min_z, ct_bounds.max_z);
    println!("Mesh bounds: X=[{:.2}, {:.2}], Y=[{:.2}, {:.2}], Z=[{:.2}, {:.2}]", 
             mesh_viz.bounds.min_x, mesh_viz.bounds.max_x, mesh_viz.bounds.min_y, mesh_viz.bounds.max_y, mesh_viz.bounds.min_z, mesh_viz.bounds.max_z);
    
    // Extract middle slices for initial display
    let mid_z_idx = volume.z.len() / 2;
    let mid_y_idx = volume.y.len() / 2;
    let mid_x_idx = volume.x.len() / 2;
    
    let ct_axial = extract_axial_slice(&volume, mid_z_idx)?;
    let ct_sagittal = extract_sagittal_slice(&volume, mid_x_idx)?;
    let ct_coronal = extract_coronal_slice(&volume, mid_y_idx)?;
    
    // Create combined visualisation data
    let combined_data = MeshCtVisualisationData {
        mesh: mesh_viz,
        ct_axial,
        ct_sagittal,
        ct_coronal,
        ct_bounds,
        total_slices: [volume.z.len(), volume.x.len(), volume.y.len()],
    };
    
    // Start web viewer with combined data

    {
        if config.auto_open {
            web_viewer::start_mesh_ct_viewer_with_volume(&combined_data, &volume, config)?;
        }
    }
    

    {
        let _ = combined_data; // Suppress unused variable warning when visualisation is disabled
        let _ = volume;
        crate::log_error("Visualisation feature not enabled. Rebuild with --features visualisation");
    }
    
    crate::log_success("Mesh and CT visualisation started");
    Ok(())
}

/// Extract an axial slice from CT volume (constant Z)
pub fn extract_axial_slice(volume: &Volume, z_idx: usize) -> Result<CtSliceData> {
    if z_idx >= volume.z.len() {
        return Err(anyhow::anyhow!("Z index {} out of bounds", z_idx));
    }
    
    let width = volume.x.len();
    let height = volume.y.len();
    let (min_val, max_val) = volume.value_range();
    let mut data = Vec::with_capacity(width * height);

    for y_idx in 0..height {
        for x_idx in 0..width {
            let scalar_idx = x_idx + y_idx * width + z_idx * width * height;
            let value = volume.scalars[scalar_idx];
            data.push(value);
        }
    }

    Ok(CtSliceData {
        width,
        height,
        data,
        value_range: (min_val, max_val),
        physical_bounds: PlaneInfo {
            origin: [volume.x[0], volume.y[0], volume.z[z_idx]],
            u_axis: [1.0, 0.0, 0.0],
            v_axis: [0.0, 1.0, 0.0],
            spacing: [
                if volume.x.len() > 1 { volume.x[1] - volume.x[0] } else { 1.0 },
                if volume.y.len() > 1 { volume.y[1] - volume.y[0] } else { 1.0 }
            ],
        },
    })
}

/// Extract a sagittal slice from CT volume (constant X)
pub fn extract_sagittal_slice(volume: &Volume, x_idx: usize) -> Result<CtSliceData> {
    if x_idx >= volume.x.len() {
        return Err(anyhow::anyhow!("X index {} out of bounds", x_idx));
    }
    
    let width = volume.z.len();  // Z becomes the width (horizontal)
    let height = volume.y.len(); // Y becomes the height (vertical)
    let (min_val, max_val) = volume.value_range();
    let mut data = Vec::with_capacity(width * height);

    for y_idx in 0..height {
        for z_idx in 0..width {
            let scalar_idx = x_idx + y_idx * volume.x.len() + z_idx * volume.x.len() * volume.y.len();
            let value = volume.scalars[scalar_idx];
            data.push(value);
        }
    }

    Ok(CtSliceData {
        width,
        height,
        data,
        value_range: (min_val, max_val),
        physical_bounds: PlaneInfo {
            origin: [volume.x[x_idx], volume.y[0], volume.z[0]],
            u_axis: [0.0, 0.0, 1.0],
            v_axis: [0.0, 1.0, 0.0],
            spacing: [
                if volume.z.len() > 1 { volume.z[1] - volume.z[0] } else { 1.0 },
                if volume.y.len() > 1 { volume.y[1] - volume.y[0] } else { 1.0 }
            ],
        },
    })
}

/// Extract a coronal slice from CT volume (constant Y)
pub fn extract_coronal_slice(volume: &Volume, y_idx: usize) -> Result<CtSliceData> {
    if y_idx >= volume.y.len() {
        return Err(anyhow::anyhow!("Y index {} out of bounds", y_idx));
    }
    
    let width = volume.x.len();  // X becomes the width (horizontal)
    let height = volume.z.len(); // Z becomes the height (vertical)
    let (min_val, max_val) = volume.value_range();
    let mut data = Vec::with_capacity(width * height);

    for z_idx in 0..height {
        for x_idx in 0..width {
            let scalar_idx = x_idx + y_idx * volume.x.len() + z_idx * volume.x.len() * volume.y.len();
            let value = volume.scalars[scalar_idx];
            data.push(value);
        }
    }

    Ok(CtSliceData {
        width,
        height,
        data,
        value_range: (min_val, max_val),
        physical_bounds: PlaneInfo {
            origin: [volume.x[0], volume.y[y_idx], volume.z[0]],
            u_axis: [1.0, 0.0, 0.0],
            v_axis: [0.0, 0.0, 1.0],
            spacing: [
                if volume.x.len() > 1 { volume.x[1] - volume.x[0] } else { 1.0 },
                if volume.z.len() > 1 { volume.z[1] - volume.z[0] } else { 1.0 }
            ],
        },
    })
}

/// Mesh visualisation data for web viewer
#[derive(Debug, Clone, serde::Serialize)]
pub struct MeshVisualisationData {
    pub vertices: Vec<f32>,
    pub faces: Vec<u32>,
    pub colors: Vec<f32>,
    pub vertex_values: Vec<f32>,
    pub bounds: MeshBounds,
    pub material_range: (f64, f64),
    pub histogram: Option<HistogramData>,
}

/// CT plane slice data for visualisation
#[derive(Debug, Clone, serde::Serialize)]
pub struct CtSliceData {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
    pub value_range: (f32, f32),
    pub physical_bounds: PlaneInfo,
}

/// Physical information about a CT plane
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaneInfo {
    pub origin: [f64; 3],
    pub u_axis: [f64; 3],
    pub v_axis: [f64; 3],
    pub spacing: [f64; 2],
}

/// Combined mesh and CT visualisation data
#[derive(Debug, Clone, serde::Serialize)]
pub struct MeshCtVisualisationData {
    pub mesh: MeshVisualisationData,
    pub ct_axial: CtSliceData,
    pub ct_sagittal: CtSliceData,
    pub ct_coronal: CtSliceData,
    pub ct_bounds: CtBounds,
    pub total_slices: [usize; 3], // [axial, sagittal, coronal] 
}

/// CT volume bounds
#[derive(Debug, Clone, serde::Serialize)]
pub struct CtBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MeshBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
}
