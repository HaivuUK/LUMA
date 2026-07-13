pub mod params;
pub mod mesh;
pub mod volume;
pub mod integrate;
pub mod export;
pub mod visualise;

use anyhow::Result;
use std::time::Instant;
use std::sync::{Arc, atomic::{AtomicUsize}};
use log::{debug, error};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step { ParseParams, LoadMesh, LoadVolume, Integrate, GroupAndExport }

pub struct Progress {
	pub step: Step,
	pub done: usize,
	pub total: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageLevel {
	Info,    // General information messages
	Status,  // Status updates (loading, processing, etc.)
	Success, // Completion messages
	Debug,   // Debug information (when enabled)
	Error,   // Error messages
}

static PROGRESS_HOOK: once_cell::sync::Lazy<std::sync::Mutex<Option<Box<dyn Fn(Progress)+Send+Sync>>>> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));
static PROGRESS_BAR: once_cell::sync::Lazy<std::sync::Mutex<Option<Arc<indicatif::ProgressBar>>>> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));
static CURRENT_STATUS: once_cell::sync::Lazy<std::sync::Mutex<Option<String>>> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

/// Mesh transformation parameters that can override parameter file settings
#[derive(Debug, Clone, Default)]
pub struct MeshTransformation {
	pub translate_x: Option<f64>,
	pub translate_y: Option<f64>,
	pub translate_z: Option<f64>,
	pub rotate_x: Option<f64>, // degrees
	pub rotate_y: Option<f64>, // degrees
	pub rotate_z: Option<f64>, // degrees
}

pub fn set_progress_hook(f: Option<Box<dyn Fn(Progress)+Send+Sync>>) { *PROGRESS_HOOK.lock().unwrap() = f; }

pub fn set_progress_bar(pb: Option<Arc<indicatif::ProgressBar>>) { *PROGRESS_BAR.lock().unwrap() = pb; }

pub fn get_current_status() -> Option<String> { CURRENT_STATUS.lock().unwrap().clone() }

/// Centralised output function that works with progress bars
pub fn output_message(level: MessageLevel, message: &str) {
	// Try to get progress bar from the static reference
	let pb_option = PROGRESS_BAR.lock().unwrap().clone();
	
	if let Some(pb) = pb_option {
		// Format the message with appropriate emoji
		let formatted_message = match level {
			MessageLevel::Info => message.to_string(),
			MessageLevel::Status => {
				// Only format status messages in debug builds
				#[cfg(debug_assertions)]
				{ message.to_string() }
				#[cfg(not(debug_assertions))]
				{ return; } // Don't display status messages in release builds
			},
			MessageLevel::Success => {
				// Only format success messages in debug builds
				#[cfg(debug_assertions)]
				{ message.to_string() }
				#[cfg(not(debug_assertions))]
				{ return; } // Don't display success messages in release builds
			},
			MessageLevel::Debug => {
				if std::env::var("LUMA_DEBUG").is_ok() {
					format!("[DEBUG] {}", message)
				} else {
					return; // Don't display debug messages if not enabled
				}
			},
			MessageLevel::Error => message.to_string(),
		};
		
		// Store status messages for later use by progress hook
		if matches!(level, MessageLevel::Status) {
			*CURRENT_STATUS.lock().unwrap() = Some(formatted_message.clone());
		}
		
		// Update the progress bar message
		pb.set_message(formatted_message);
	} else {
		// Fallback to regular printing if no progress bar is set
		match level {
			MessageLevel::Info => println!("{}", message),
			MessageLevel::Status => {
				// Only print status messages in debug builds
				#[cfg(debug_assertions)]
				println!("{}", message);
			},
			MessageLevel::Success => {
				// Only print success messages in debug builds
				#[cfg(debug_assertions)]
				println!("{}", message);
			},
			MessageLevel::Debug => {
				debug!("{}", message);
			},
			MessageLevel::Error => error!("{}", message),
		}
	}
}

/// Convenience functions for different message types
pub fn log_info(message: &str) { output_message(MessageLevel::Info, message); }
pub fn log_status(message: &str) { output_message(MessageLevel::Status, message); }
pub fn log_success(message: &str) { output_message(MessageLevel::Success, message); }
pub fn log_debug(message: &str) { output_message(MessageLevel::Debug, message); }
pub fn log_error(message: &str) { output_message(MessageLevel::Error, message); }

fn emit(step: Step, done: usize, total: usize) {
	if let Some(cb) = &*PROGRESS_HOOK.lock().unwrap() { cb(Progress { step, done, total }); }
}

pub fn run(params_file: &str, ct_path: &str, mesh_path: &str) -> Result<String> {
	run_with_options(params_file, ct_path, mesh_path, None, None, None)
}

pub fn run_with_options(
	params_file: &str, 
	ct_path: &str, 
	mesh_path: &str, 
	vis_config: Option<visualise::VisualisationConfig>,
	transformation_override: Option<MeshTransformation>,
	histogram_overrides: Option<export::histogram::HistogramOverrides>
) -> Result<String> {
	// Check if mesh already has material assignments
	let has_materials = mesh_path.contains("-mat.") || mesh_path.contains("_mat.");
	
	if has_materials {
		log_error("Mesh appears to already have material assignments. Skipping material assignment process.");
		return Err(anyhow::anyhow!("Input mesh already has material assignments. Use a mesh without materials for processing."));
	}
	
	emit(Step::ParseParams, 0, 2); 
	let p = params::Params::parse(params_file)?;
	let histogram_options = export::histogram::HistogramOptions::from_params(&p, histogram_overrides);
	emit(Step::ParseParams, 1, 2);
	emit(Step::ParseParams, 2, 2);
	
	emit(Step::LoadMesh, 0, 3); 
	log_status(&format!("Loading mesh: {mesh_path}"));
	emit(Step::LoadMesh, 1, 3);
	let mesh = mesh::load_mesh_with_transformation(mesh_path, &p.ignore, &p, transformation_override.as_ref())?; 
	emit(Step::LoadMesh, 2, 3);
	emit(Step::LoadMesh, 3, 3);
	
	emit(Step::LoadVolume, 0, 3); 
	log_status(&format!("Loading CT volume: {ct_path}"));
	emit(Step::LoadVolume, 1, 3);
	let vol = volume::load_volume(ct_path)?; 
	emit(Step::LoadVolume, 2, 3);
	emit(Step::LoadVolume, 3, 3);
	// Integration progress
	let total_elements: usize = mesh.parts.iter().filter(|pt| !pt.ignore).map(|pt| pt.elements.len()).sum();
	log_status(&format!("Integrating {} elements...", total_elements));
	let counter = Arc::new(AtomicUsize::new(0));
	// Install transient hook for per-element progress inside parallel map
	let local_counter = counter.clone();
	integrate::set_progress_counter(Some(local_counter.clone()));
	let tic = Instant::now();
	// More frequent updates: aim for ~1000 updates max for better real-time feedback
	let interval = std::cmp::max(1, total_elements / 1000);
	let per_part = integrate::process_with_progress(&mesh, &vol, &p, move |done| {
		if done % interval == 0 || done == total_elements { emit(Step::Integrate, done, total_elements); }
	})?;
	integrate::set_progress_counter(None);
	emit(Step::Integrate, total_elements, total_elements);
	log_success(&format!("Integration done in {:.3}s", tic.elapsed().as_secs_f64()));

	let histogram_data = if histogram_options.export || histogram_options.view {
		let mut all_raw_moduli = Vec::new();
		for (pi, part) in mesh.parts.iter().enumerate() {
			if !part.ignore {
				all_raw_moduli.extend(&per_part[pi]);
			}
		}
		Some(export::histogram::build_material_histogram(&all_raw_moduli, &p))
	} else {
		None
	};

	emit(Step::GroupAndExport, 0, 2);
	log_status("Writing output file...");
	let out = if has_extension(mesh_path, "cdb") {
		export::ansys_cdb::write_cdb(&mesh, &per_part, &p, mesh_path)?
	} else if has_extension(mesh_path, "vtk") || has_extension(mesh_path, "vtu") {
		export::model_vtk::write_vtk(&mesh, &per_part, &p, mesh_path, false)?
	} else if has_extension(mesh_path, "feb") {
		export::febio_feb::write_feb(&mesh, &per_part, &p, mesh_path)?
	} else {
		export::abaqus_inp::write_abq(&mesh, &per_part, &p, mesh_path)?
	};
	emit(Step::GroupAndExport, 1, 2);
	log_success(&format!("Output written: {out}"));
	if histogram_options.export && let Some(data) = histogram_data.as_ref() {
		let (csv_path, json_path) = export::histogram::write_histogram_outputs(
			&out,
			histogram_options.export_dir.as_deref(),
			data,
		)?;
		log_success(&format!(
			"Histogram exported: {}, {}",
			csv_path.display(),
			json_path.display()
		));
	}

	// Generate visualisation if requested
	if let Some(config) = vis_config {
		log_status("Generating 3D mesh visualisation...");
		let histogram_for_view = if histogram_options.view {
			histogram_data.clone()
		} else {
			None
		};
		visualise::generate_visualisation(&mesh, &per_part, &p, &config, histogram_for_view)?;
		log_success("Visualisation complete!");
	}
	emit(Step::GroupAndExport, 2, 2);
	
	Ok(out)
}

fn has_extension(path: &str, expected: &str) -> bool {
	std::path::Path::new(path)
		.extension()
		.and_then(|ext| ext.to_str())
		.map(|ext| ext.eq_ignore_ascii_case(expected))
		.unwrap_or(false)
}

/// Generate mesh and CT visualisation without running the material assignment pipeline
/// Loads mesh and CT data independently for direct visualisation
pub fn visualise_mesh_and_ct(
	mesh_path: &str,
	ct_path: &str,
	vis_config: Option<visualise::VisualisationConfig>
) -> Result<()> {
	visualise_mesh_and_ct_with_transformation(mesh_path, ct_path, vis_config, None)
}

/// Generate mesh and CT visualisation with optional transformation
pub fn visualise_mesh_and_ct_with_transformation(
	mesh_path: &str,
	ct_path: &str,
	vis_config: Option<visualise::VisualisationConfig>,
	transformation_override: Option<MeshTransformation>
) -> Result<()> {
	let config = vis_config.unwrap_or_default();
	
	// If no transformations, use the original function
	if transformation_override.is_none() {
		return visualise::visualise_mesh_with_ct(mesh_path, ct_path, &config);
	}
	
	// Otherwise, load mesh with transformations
	let transform = transformation_override.unwrap();
	let has_transformations = transform.translate_x.is_some() || 
		transform.translate_y.is_some() || 
		transform.translate_z.is_some() ||
		transform.rotate_x.is_some() || 
		transform.rotate_y.is_some() || 
		transform.rotate_z.is_some();
		
	if !has_transformations {
		return visualise::visualise_mesh_with_ct(mesh_path, ct_path, &config);
	}
	
	log_status("Loading mesh for visualisation...");
	let mut mesh = mesh::load_mesh(mesh_path, &Vec::new())?;
	log_success(&format!("Loaded mesh with {} parts", mesh.parts.len()));
	
	log_status("Applying mesh transformations for visualisation...");
	mesh.apply_transformation(
		transform.translate_x,
		transform.translate_y,
		transform.translate_z,
		transform.rotate_x,
		transform.rotate_y,
		transform.rotate_z
	);
	
	let transforms = vec![
		transform.translate_x.map(|v| format!("translate X: {}", v)),
		transform.translate_y.map(|v| format!("translate Y: {}", v)),
		transform.translate_z.map(|v| format!("translate Z: {}", v)),
		transform.rotate_x.map(|v| format!("rotate X: {}°", v)),
		transform.rotate_y.map(|v| format!("rotate Y: {}°", v)),
		transform.rotate_z.map(|v| format!("rotate Z: {}°", v)),
	].into_iter().flatten().collect::<Vec<_>>();
	
	log_success(&format!("Applied transformations: {}", transforms.join(", ")));
	
	log_status("Loading CT volume...");
	let volume = volume::load_volume(ct_path)?;
	log_success("CT volume loaded successfully");
	
	// Create dummy material data for mesh visualisation (uniform color)
	let material_data: Vec<Vec<f64>> = mesh.parts.iter()
		.map(|part| vec![1.0; part.elements.len()])
		.collect();
		
	// Create mesh visualisation
	let mesh_viz = visualise::mesh_visualiser::create_visualisation_data(&mesh, &material_data, &config, None)?;

	// Create CT bounds and slices
	let ct_bounds = visualise::CtBounds {
		min_x: volume.x[0],
		max_x: volume.x[volume.x.len() - 1],
		min_y: volume.y[0],
		max_y: volume.y[volume.y.len() - 1],
		min_z: volume.z[0],
		max_z: volume.z[volume.z.len() - 1],
	};
	
	// Extract CT slices at middle positions
	let mid_x = volume.x.len() / 2;
	let mid_y = volume.y.len() / 2;
	let mid_z = volume.z.len() / 2;
	
	let slices = [
		visualise::extract_axial_slice(&volume, mid_z)?,
		visualise::extract_sagittal_slice(&volume, mid_x)?,
		visualise::extract_coronal_slice(&volume, mid_y)?,
	];
	
	// Create combined visualisation data
	let combined_data = visualise::MeshCtVisualisationData {
		mesh: mesh_viz,
		ct_axial: slices[0].clone(),
		ct_sagittal: slices[1].clone(),
		ct_coronal: slices[2].clone(),
		ct_bounds,
		total_slices: [volume.z.len(), volume.x.len(), volume.y.len()],
	};

	// Start web viewer
	visualise::web_viewer::start_mesh_ct_viewer_with_volume(&combined_data, &volume, &config)
}

/// Visualise a model that has already been processed with material assignments, without needing to re-run the assignment process
pub fn visualise_assigned_model(
	mesh_path: &str,
	vis_config: Option<visualise::VisualisationConfig>,
) -> Result<()> {
	let config = vis_config.unwrap_or_default();
	log_status("Loading processed mesh for visualisation...");
	let (mesh, per_part) = mesh::load_mesh_with_materials(mesh_path)?;
	log_success(&format!("Loaded mesh with {} parts and material assignments", mesh.parts.len()));

	log_status("Generating 3D mesh visualisation...");
	// Since we don't have exactly Params, we can optionally rebuild histogram view if wanted
	// For now parameter passing might require custom handling or we just pass a default.
	// Let's create a minimal Params or use the config. The visualiser needs `volume::HistogramData` or something if it was generated.
	// We can just construct a visualisation.
	// But visualise::generate_visualisation takes P: &Params. Wait, let's look at visualise mod.
	visualise::generate_visualisation_from_assigned(&mesh, &per_part, &config)
}
