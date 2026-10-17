use super::{MeshVisualisationData, MeshCtVisualisationData, VisualisationConfig};
use crate::volume::Volume;
use anyhow::{Result, anyhow};
use log::warn;
use nalgebra::point;
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone)]
struct AppState {
    mesh_data: Option<MeshVisualisationData>,
    mesh_ct_data: Option<MeshCtVisualisationData>,
    volume: Option<Volume>,
}

 #[tauri::command]
 fn get_template_params(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
     if let Some(ref data) = state.mesh_ct_data {
         let mid_material = (data.mesh.material_range.0 + data.mesh.material_range.1) / 2.0;
         return Ok(serde_json::json!({
             "vertex_count": data.mesh.vertices.len(),
             "face_count": data.mesh.faces.len(),
             "min_material": format!("{:.3}", data.mesh.material_range.0),
             "max_material": format!("{:.3}", data.mesh.material_range.1),
             "mid_material": format!("{:.3}", mid_material),
             "total_axial_slices": data.total_slices[0],
             "total_sagittal_slices": data.total_slices[1],
             "total_coronal_slices": data.total_slices[2],
         }));
     } else if let Some(ref data) = state.mesh_data {
         let mid_material = (data.material_range.0 + data.material_range.1) / 2.0;
         return Ok(serde_json::json!({
             "vertex_count": data.vertices.len(),
             "face_count": data.faces.len(),
             "min_material": format!("{:.3}", data.material_range.0),
             "max_material": format!("{:.3}", data.material_range.1),
             "mid_material": format!("{:.3}", mid_material),
         }));
     }
     Err("No data available".to_string())
 }

 #[tauri::command]
 fn get_ct_slice(plane: String, index: usize, state: State<'_, AppState>) -> Result<super::CtSliceData, String> {
     let volume = state.volume.as_ref().ok_or_else(|| "No volume data".to_string())?;
     let slice_data = match plane.as_str() {
         "axial" => super::extract_axial_slice(volume, index).map_err(|e| e.to_string()) ?,
         "sagittal" => super::extract_sagittal_slice(volume, index).map_err(|e| e.to_string())?,
         "coronal" => super::extract_coronal_slice(volume, index).map_err(|e| e.to_string())?,
         _ => return Err(format!("Invalid plane: {}", plane)),
     };
     Ok(slice_data)
 }

#[derive(serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RoiMeanOptions {
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
    z_start: f64,
    z_end: f64,
    x_center: f64,
    y_center: f64,
    radius: f64,
    points: Vec<[f64; 2]>,
}

impl Default for RoiMeanOptions {
    fn default() -> Self {
        Self {
            x_start: 0.0,
            x_end: 0.0,
            y_start: 0.0,
            y_end: 0.0,
            z_start: 0.0,
            z_end: 0.0,
            x_center: 0.0,
            y_center: 0.0,
            radius: 0.0,
            points: Vec::new(),
        }
    }
}

#[tauri::command]
fn get_ct_roi_mean(
    roi_type: String,
    options: RoiMeanOptions,
    state: State<'_, AppState>,
) -> Result<f64, String> {
    if roi_type == "nodal" && options.points.len() < 3 {
        return Err("At least 3 points are required to define a polygon".to_string());
    }

    let volume = state.volume.as_ref().ok_or_else(|| "No volume data".to_string())?;

    let nx = volume.x.len();
    let ny = volume.y.len();
    let nz = volume.z.len();
    if nx < 2 || ny < 2 || nz < 2 {
        return Err("Volume has insufficient data".to_string());
    }

    let origin_x = volume.x[0];
    let origin_y = volume.y[0];
    let origin_z = volume.z[0];
    let spacing_x = volume.x[1] - volume.x[0];
    let spacing_y = volume.y[1] - volume.y[0];
    let spacing_z = volume.z[1] - volume.z[0];

    let (mut x_start, mut x_end, mut y_start, mut y_end) = (0.0, 0.0, 0.0, 0.0);

    if roi_type == "nodal" {
        (x_start, x_end) = (f64::INFINITY, f64::NEG_INFINITY);
        (y_start, y_end) = (f64::INFINITY, f64::NEG_INFINITY);
        for point in &options.points {
            x_start = x_start.min(point[0]);
            x_end = x_end.max(point[0]);
            y_start = y_start.min(point[1]);
            y_end = y_end.max(point[1]);
        }
    } else {
        x_start = options.x_start;
        x_end = options.x_end;
        y_start = options.y_start;
        y_end = options.y_end;
    }

    let idx_x_start = map_to_index(x_start, origin_x, spacing_x, nx);
    let idx_x_end = map_to_index(x_end, origin_x, spacing_x, nx);

    let idx_y_start = map_to_index(y_start, origin_y, spacing_y, ny);
    let idx_y_end = map_to_index(y_end, origin_y, spacing_y, ny);

    let idx_z_start = map_to_index(options.z_start, origin_z, spacing_z, nz);
    let idx_z_end = map_to_index(options.z_end, origin_z, spacing_z, nz);

    let (x0, x1) = (idx_x_start.min(idx_x_end), idx_x_start.max(idx_x_end));
    let (y0, y1) = (idx_y_start.min(idx_y_end), idx_y_start.max(idx_y_end));
    let (z0, z1) = (idx_z_start.min(idx_z_end), idx_z_start.max(idx_z_end));

    let stride_xy = nx * ny;

    let mut sum = 0.0f64;
    let mut count = 0usize;

    match roi_type.as_str() {
        "box" => (sum, count) = get_ct_roi_mean_box(z0, z1, y0, y1, x0, x1, nx, stride_xy, volume, sum, count),
        "cylinder" => (sum, count) = get_ct_roi_mean_cylinder(z0, z1, nx, ny, options.x_center, options.y_center, options.radius, stride_xy, volume, sum, count),
        "nodal" => (sum, count) = get_ct_roi_mean_nodal(x0, x1, y0, y1, z0, z1, options.points, nx, stride_xy, volume, sum, count),
        _ => return Err(format!("Invalid ROI type: {}", roi_type)),
    }

    if count == 0 {
        return Err("ROI contains no voxels".to_string());
    }

    Ok(sum / count as f64)
}

#[inline]
fn map_to_index(world: f64, origin: f64, spacing: f64, dim: usize) -> usize {
    (((world - origin) / spacing).round() as isize).clamp(0, (dim as isize) - 1) as usize
}

#[inline]
fn get_ct_roi_mean_box(
    z0: usize,
    z1: usize,
    y0: usize,
    y1: usize,
    x0: usize,
    x1: usize,
    nx: usize,
    stride_xy: usize,
    volume: &Volume,
    mut sum: f64,
    mut count: usize
) -> (f64, usize) {
    for z in z0..=z1 {
        for y in y0..=y1 {
            let row_offset = y * nx + z * stride_xy;
            for x in x0..=x1 {
                let index = row_offset + x;
                if let Some(value) = volume.scalars.get(index) {
                    sum += *value as f64;
                    count += 1;
                }
            }
        }
    }
    (sum, count)
}

#[inline]
fn get_ct_roi_mean_cylinder(
    z0: usize,
    z1: usize,
    nx: usize,
    ny: usize,
    x_center: f64,
    y_center: f64,
    radius: f64,
    stride_xy: usize,
    volume: &Volume,
    mut sum: f64,
    mut count: usize
 ) -> (f64, usize) {
    for z in z0..=z1 {
        for y in 0..ny {
            let row_offset = y * nx + z * stride_xy;
            for x in 0..nx {
                let world_x = volume.x[x];
                let world_y = volume.y[y];
                let dx = world_x - x_center;
                let dy = world_y - y_center;
                if dx * dx + dy * dy <= radius * radius {
                    let index = row_offset + x;
                    if let Some(value) = volume.scalars.get(index) {
                        sum += *value as f64;
                        count += 1;
                    }
                }
            }
        }
    }
    (sum, count)
}

#[inline]
fn get_ct_roi_mean_nodal(
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
    z0: usize,
    z1: usize,
    points: Vec<[f64; 2]>,
    nx: usize,
    stride_xy: usize,
    volume: &Volume,
    mut sum: f64,
    mut count: usize
) -> (f64, usize) {
    let mut inside_polygon: Vec<(usize, usize)>= Vec::new();
    for y in y0..=y1 {
        let wy = volume.y[y];
        for x in x0..=x1 {
            let wx = volume.x[x];
            let mut inside = false;
            let n = &points.len();
            let mut j = n - 1;
            for i in 0..*n {
                let (xi, yi) = (points[i][0], points[i][1]);
                let (xj, yj) = (points[j][0], points[j][1]);
                if ((yi > wy) != (yj > wy)) {
                    let x_intersect = (xj - xi) * (wy - yi) / (yj - yi) + xi;
                    if wx < x_intersect {
                        inside = !inside;
                    }
                }
                j = i;
            }
            if inside {
                inside_polygon.push((x, y));
            }
        }
    }

    if inside_polygon.is_empty() {
        warn!("ROI polygon contains no voxels");
    }

    for z in z0..=z1 {
        let z_offset = z * stride_xy;
        for &(x, y) in &inside_polygon {
            let index = z_offset + y * nx + x;
            if let Some(value) = volume.scalars.get(index) {
                sum += *value as f64;
                count += 1;
            }
        }
    }
    (sum, count)
}

#[tauri::command]
fn get_view_mode(state: State<'_, AppState>) -> Result<String, String> {
    if state.mesh_ct_data.is_some() {
        Ok("mesh_ct".to_string())
    } else if state.mesh_data.is_some() {
        Ok("mesh".to_string())
    } else {
        Err("No data available".to_string())
    }
}

 #[tauri::command]
fn get_mesh_metadata(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let data = state.mesh_data.as_ref().ok_or_else(|| "No mesh data".to_string())?;
    Ok(serde_json::json!({
        "bounds": data.bounds,
        "material_range": data.material_range,
        "histogram": data.histogram,
        "filename": data.filename,
    }))
}

  /// Binary IPC command for mesh vertices — returns raw f32 bytes via Tauri v2 tauri::ipc::Response.
   #[tauri::command]
fn get_mesh_vertices(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let data = state.mesh_data.as_ref().ok_or_else(|| "No mesh data".to_string())?;
    let bytes = bytemuck::cast_slice::<f32, u8>(data.vertices.as_slice()).to_vec();
    Ok(tauri::ipc::Response::new(bytes))
}

  /// Binary IPC command for mesh faces — returns raw u32 bytes via Tauri v2 tauri::ipc::Response.
   #[tauri::command]
fn get_mesh_faces(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let data = state.mesh_data.as_ref().ok_or_else(|| "No mesh data".to_string())?;
    let bytes = bytemuck::cast_slice::<u32, u8>(data.faces.as_slice()).to_vec();
    Ok(tauri::ipc::Response::new(bytes))
}

  /// Binary IPC command for mesh colors — returns raw f32 bytes via Tauri v2 tauri::ipc::Response.
   #[tauri::command]
fn get_mesh_colors(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let data = state.mesh_data.as_ref().ok_or_else(|| "No mesh data".to_string())?;
    let bytes = bytemuck::cast_slice::<f32, u8>(data.colors.as_slice()).to_vec();
    Ok(tauri::ipc::Response::new(bytes))
}

  /// Binary IPC command for mesh vertex values — returns raw f32 bytes via Tauri v2 tauri::ipc::Response.
   #[tauri::command]
fn get_mesh_vertex_values(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let data = state.mesh_data.as_ref().ok_or_else(|| "No mesh data".to_string())?;
    let bytes = bytemuck::cast_slice::<f32, u8>(data.vertex_values.as_slice()).to_vec();
    Ok(tauri::ipc::Response::new(bytes))
}

// Mirror for mesh_ct (delegating to mesh_ct_data.mesh)
 #[tauri::command]
fn get_mesh_ct_metadata(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let data = state.mesh_ct_data.as_ref().ok_or_else(|| "No mesh_ct data".to_string())?;
    Ok(serde_json::json!({
        "ct_bounds": data.ct_bounds,
        "total_slices": data.total_slices,
        "mesh_metadata": {
            "bounds": data.mesh.bounds,
            "material_range": data.mesh.material_range,
            "histogram": data.mesh.histogram
        }
    }))
}

  /// Binary IPC command for mesh_ct vertices — returns raw f32 bytes via Tauri v2 tauri::ipc::Response.
   #[tauri::command]
fn get_mesh_ct_vertices(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let data = state.mesh_ct_data.as_ref().ok_or_else(|| "No mesh_ct data".to_string())?;
    let bytes = bytemuck::cast_slice::<f32, u8>(data.mesh.vertices.as_slice()).to_vec();
    Ok(tauri::ipc::Response::new(bytes))
}

  /// Binary IPC command for mesh_ct faces — returns raw u32 bytes via Tauri v2 tauri::ipc::Response.
   #[tauri::command]
fn get_mesh_ct_faces(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let data = state.mesh_ct_data.as_ref().ok_or_else(|| "No mesh_ct data".to_string())?;
    let bytes = bytemuck::cast_slice::<u32, u8>(data.mesh.faces.as_slice()).to_vec();
    Ok(tauri::ipc::Response::new(bytes))
}

  /// Binary IPC command for mesh_ct colors — returns raw f32 bytes via Tauri v2 tauri::ipc::Response.
   #[tauri::command]
fn get_mesh_ct_colors(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let data = state.mesh_ct_data.as_ref().ok_or_else(|| "No mesh_ct data".to_string())?;
    let bytes = bytemuck::cast_slice::<f32, u8>(data.mesh.colors.as_slice()).to_vec();
    Ok(tauri::ipc::Response::new(bytes))
}

  /// Binary IPC command for mesh_ct vertex values — returns raw f32 bytes via Tauri v2 tauri::ipc::Response.
   #[tauri::command]
fn get_mesh_ct_vertex_values(state: State<'_, AppState>) -> Result<tauri::ipc::Response, String> {
    let data = state.mesh_ct_data.as_ref().ok_or_else(|| "No mesh_ct data".to_string())?;
    let bytes = bytemuck::cast_slice::<f32, u8>(data.mesh.vertex_values.as_slice()).to_vec();
    Ok(tauri::ipc::Response::new(bytes))
}

fn build_tauri_app(app_state: AppState, config: &VisualisationConfig) -> Result<()> {
    let width = config.viewport_resolution.0 as f64 * 2.0;
    let height = config.viewport_resolution.1 as f64 * 2.0;
    
    tauri::Builder::default()
        .manage(app_state)
         .invoke_handler(tauri::generate_handler![
             get_template_params,
             get_view_mode,
             get_ct_slice,
             get_ct_roi_mean,
             get_mesh_metadata,
             get_mesh_vertices,
             get_mesh_faces,
             get_mesh_colors,
             get_mesh_vertex_values,
             get_mesh_ct_metadata,
             get_mesh_ct_vertices,
             get_mesh_ct_faces,
             get_mesh_ct_colors,
             get_mesh_ct_vertex_values
         ])
        .setup(move |app| {
            let _webview = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into())
            )
            .title("LUMA Visualisation")
            .inner_size(width, height)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow!("Failed to run internal viewer: {}", e))
}

pub fn start_viewer(data: &MeshVisualisationData, config: &VisualisationConfig) -> Result<()> {
    crate::log_success("Starting 3D Mesh Visualisation...");
    let state = AppState {
        mesh_data: Some(data.clone()),
        mesh_ct_data: None,
        volume: None,
    };
    build_tauri_app(state, config)
}

pub fn start_mesh_ct_viewer(data: &MeshCtVisualisationData, config: &VisualisationConfig) -> Result<()> {
    crate::log_success("Starting Mesh and CT Visualisation...");
    let state = AppState {
        mesh_data: None,
        mesh_ct_data: Some(data.clone()),
        volume: None,
    };
    build_tauri_app(state, config)
}

pub fn start_mesh_ct_viewer_with_volume(data: &MeshCtVisualisationData, volume: &Volume, config: &VisualisationConfig) -> Result<()> {
    crate::log_success("Starting Mesh and CT Visualisation with Volume...");
    let state = AppState {
        mesh_data: None,
        mesh_ct_data: Some(data.clone()),
        volume: Some(volume.clone()),
    };
    build_tauri_app(state, config)
}
