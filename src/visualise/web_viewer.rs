use super::{MeshVisualisationData, MeshCtVisualisationData, VisualisationConfig};
use crate::volume::Volume;
use anyhow::{Result, anyhow};
use log::warn;
use nalgebra::point;

#[derive(Clone)]
struct AppState {
    mesh_data: Option<MeshVisualisationData>,
    mesh_ct_data: Option<MeshCtVisualisationData>,
    volume: Option<Volume>,
}

pub fn start_viewer(data: &MeshVisualisationData, config: &VisualisationConfig) -> Result<()> {
    crate::log_success("Starting 3D Mesh Visualisation...");
    let state = AppState {
        mesh_data: Some(data.clone()),
        mesh_ct_data: None,
        volume: None,
    };
    Ok(())
}

pub fn start_mesh_ct_viewer(data: &MeshCtVisualisationData, config: &VisualisationConfig) -> Result<()> {
    crate::log_success("Starting Mesh and CT Visualisation...");
    let state = AppState {
        mesh_data: None,
        mesh_ct_data: Some(data.clone()),
        volume: None,
    };
    Ok(())
}

pub fn start_mesh_ct_viewer_with_volume(data: &MeshCtVisualisationData, volume: &Volume, config: &VisualisationConfig) -> Result<()> {
    crate::log_success("Starting Mesh and CT Visualisation with Volume...");
    let state = AppState {
        mesh_data: None,
        mesh_ct_data: Some(data.clone()),
        volume: Some(volume.clone()),
    };
    Ok(())
}
