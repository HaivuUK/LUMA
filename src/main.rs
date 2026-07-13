use clap::{Parser, ValueEnum};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

/// LUMA - Rust tool for bone material assignment
/// Local Unit Modulus Assignment (LUMA)

#[derive(ValueEnum, Clone, Debug)]
enum VizMode {
	Align,			// Align mesh and CT
	Material,		// Show material assignments
	Processed,		// Show processed model with materials and transformations
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(allow_negative_numbers = true)]
struct Args {
	/// Parameter file (.toml)
	#[arg(short, long)]
	params: Option<String>,
	/// CT data path (.vtk, .nii, .nrrd or DICOM directory)
	#[arg(short, long)]
	ct: Option<String>,
	/// Mesh (.inp, .cdb, .feb, .vtk, or .vtu)
	#[arg(short, long)]
	mesh: String,
	/// Enable 3D visualisation
	#[arg(long = "visualise", num_args = 0..=1, value_name = "MODE")]
	visualise: Option<Option<VizMode>>,
	/// Viewport resolution for visualisation
	#[arg(long, default_value = "512")]
	viz_resolution: u32,
	/// Export visualisation data to directory
	#[arg(long)]
	viz_export: Option<String>,
	/// Translate mesh: X Y Z (mm)
	#[arg(long, value_names = ["X", "Y", "Z"], num_args = 3)]
	trans: Option<Vec<f64>>,
	/// Rotate mesh: X Y Z (degrees)
	#[arg(long, value_names = ["X", "Y", "Z"], num_args = 3)]
	rot: Option<Vec<f64>>,
	/// Export a material histogram (CSV + JSON)
	#[arg(long)]
	histogram: bool,
	/// Output directory for histogram exports
	#[arg(long)]
	histogram_dir: Option<String>,
}

fn resolve_visualise_selection(args: &Args) -> Result<Option<VizMode>> {
	match &args.visualise {
		Some(Some(mode)) => Ok(Some(mode.clone())),
		Some(None) => match (&args.ct, &args.params) {
			(Some(_), Some(_)) => Ok(Some(VizMode::Processed)),
			(_, Some(_)) => Ok(Some(VizMode::Material)),
			(Some(_), None) => Ok(Some(VizMode::Align)),
			(None, None) => Ok(Some(VizMode::Material)),
		},
		None => Ok(None),
	}
}

fn main() -> Result<()> {
	// Initialise logger - only show debug/trace in debug builds or with explicit env var
	let log_level = if cfg!(debug_assertions) || std::env::var("RUST_LOG").is_ok() {
		log::LevelFilter::Debug
	} else {
		log::LevelFilter::Info
	};
	
	env_logger::Builder::from_default_env()
		.filter_level(log_level)
		.init();
	
	let args = Args::parse();
	
	println!("************** LUMA **************");
	
	// Create a progress bar with indicatif
	let pb = Arc::new(ProgressBar::new(100));
	pb.set_style(
		ProgressStyle::with_template(
			"{bar:30.cyan/blue} {pos:>3}% {msg}"
		)?
		.progress_chars("█▉▊▋▌▍▎▏ ")
	);
	
	// Set the progress bar for centralised output  
	// luma::set_progress_bar(Some(pb.clone())); // Function not accessible from binary
	
	// Register progress hook using indicatif
	let pb_clone = pb.clone();
	luma::set_progress_hook(Some(Box::new(move |p| {
		use luma::Step;
		
		// Calculate overall progress as percentage across all steps
		let (step_weight, step_name) = match p.step {
			Step::ParseParams => (5.0, "Parsing parameters"),
			Step::LoadMesh => (10.0, "Loading mesh"),
			Step::LoadVolume => (15.0, "Loading CT volume"),
			Step::Integrate => (60.0, "Integrating elements"), // Longest step gets most weight
			Step::GroupAndExport => (10.0, "Exporting results"),
		};
		
		let base_progress = match p.step {
			Step::ParseParams => 0.0,
			Step::LoadMesh => 5.0,
			Step::LoadVolume => 15.0,
			Step::Integrate => 30.0,
			Step::GroupAndExport => 90.0,
		};
		
		if p.total == 0 { return; }
		
		let step_progress = (p.done as f64 / p.total as f64) * step_weight;
		let overall_progress = (base_progress + step_progress).min(100.0) as u64;
		
		// Update progress bar
		pb_clone.set_position(overall_progress);
		
		// Set message with more details for integration step
		if matches!(p.step, Step::Integrate) && p.total > 0 {
			pb_clone.set_message(format!("{} ({}/{})", step_name, p.done, p.total));
		} else {
			pb_clone.set_message(step_name.to_string());
		}
		
		// Finish progress bar when complete
		if p.done == p.total && matches!(p.step, Step::GroupAndExport) {
			pb_clone.finish_with_message("Complete!");
		}
	}))); 
	
	// Create transformation override from command line arguments
	let transformation_override = if args.trans.is_some() || args.rot.is_some() {
		let (tx, ty, tz) = if let Some(trans) = &args.trans {
			(Some(trans[0]), Some(trans[1]), Some(trans[2]))
		} else {
			(None, None, None)
		};
		
		let (rx, ry, rz) = if let Some(rot) = &args.rot {
			(Some(rot[0]), Some(rot[1]), Some(rot[2]))
		} else {
			(None, None, None)
		};
		
		Some(luma::MeshTransformation {
			translate_x: tx,
			translate_y: ty,
			translate_z: tz,
			rotate_x: rx,
			rotate_y: ry,
			rotate_z: rz,
		})
	} else {
		None
	};
	
	let histogram_overrides = if args.histogram || args.histogram_dir.is_some() {
		Some(luma::export::histogram::HistogramOverrides {
			export: if args.histogram { Some(true) } else { None },
			view: None,
			export_dir: args.histogram_dir.clone(),
		})
	} else {
		None
	};

	let visualise_selection = resolve_visualise_selection(&args)?;
	let viz_config = if visualise_selection.is_some() {
		Some(luma::visualise::VisualisationConfig {
			viewport_resolution: (args.viz_resolution, args.viz_resolution),
			auto_open: true,
			export_dir: args.viz_export.clone(),
			element_sampling: 1.0,
		})
	} else {
		None
	};

	match visualise_selection {
		Some(VizMode::Align) => {
			let config = viz_config.unwrap();

			let final_transformation = if transformation_override.is_some() {
				transformation_override
			} else if let Some(params_path) = &args.params {
				match luma::params::Params::parse(params_path) {
					Ok(params) => {
						let has_params_transforms = params.mesh_translate_x.is_some()
							|| params.mesh_translate_y.is_some()
							|| params.mesh_translate_z.is_some()
							|| params.mesh_rotate_x.is_some()
							|| params.mesh_rotate_y.is_some()
							|| params.mesh_rotate_z.is_some();

						if has_params_transforms {
							Some(luma::MeshTransformation {
								translate_x: params.mesh_translate_x,
								translate_y: params.mesh_translate_y,
								translate_z: params.mesh_translate_z,
								rotate_x: params.mesh_rotate_x,
								rotate_y: params.mesh_rotate_y,
								rotate_z: params.mesh_rotate_z,
							})
						} else {
							None
						}
					}
					Err(e) => {
						pb.println(format!("Warning: Could not load parameter file for transformations: {}", e));
						None
					}
				}
			} else {
				None
			};

			let ct_path = match &args.ct {
				Some(path) => path,
				None => {
					pb.println("Error: --ct is required for --visualise align");
					pb.finish_with_message("Failed!");
					return Err(anyhow::anyhow!("Missing required CT data"));
				}
			};

			match luma::visualise_mesh_and_ct_with_transformation(&args.mesh, ct_path, Some(config), final_transformation) {
				Ok(_) => {
					pb.finish_with_message("Visualisation complete!");
					Ok(())
				}
				Err(e) => {
					pb.println(format!("Error: {e:?}"));
					pb.finish_with_message("Failed!");
					Err(e)
				}
			}
		}
		Some(VizMode::Material) => {
			let config = viz_config.unwrap();
			match luma::visualise_assigned_model(&args.mesh, Some(config)) {
				Ok(_) => {
					pb.finish_with_message("Visualisation complete!");
					Ok(())
				}
				Err(e) => {
					pb.println(format!("Error: {e:?}"));
					pb.finish_with_message("Failed!");
					Err(e)
				}
			}
		}
		Some(VizMode::Processed) => {
			let ct_path = match &args.ct {
				Some(path) => path,
				None => {
					pb.println("Error: --ct is required for --visualise processed");
					pb.finish_with_message("Failed!");
					return Err(anyhow::anyhow!("Missing required CT data"));
				}
			};

			let params_path = match &args.params {
				Some(path) => path,
				None => {
					pb.println("Error: --params is required for --visualise processed");
					pb.finish_with_message("Failed!");
					return Err(anyhow::anyhow!("Missing required parameter file"));
				}
			};

			match luma::run_with_options(params_path, ct_path, &args.mesh, viz_config, transformation_override, histogram_overrides) {
				Ok(_out) => Ok(()),
				Err(e) => {
					pb.println(format!("Error: {e:?}"));
					pb.finish_with_message("Failed!");
					Err(e)
				}
			}
		}
		None => {
			let ct_path = match &args.ct {
				Some(path) => path,
				None => {
					pb.println("Error: --ct is required when not using --visualise");
					pb.finish_with_message("Failed!");
					return Err(anyhow::anyhow!("Missing required CT data"));
				}
			};

			let params_path = match &args.params {
				Some(path) => path,
				None => {
					pb.println("Error: --params is required when not using --visualise");
					pb.finish_with_message("Failed!");
					return Err(anyhow::anyhow!("Missing required parameter file"));
				}
			};

			match luma::run_with_options(params_path, ct_path, &args.mesh, viz_config, transformation_override, histogram_overrides) {
				Ok(_out) => Ok(()),
				Err(e) => {
					pb.println(format!("Error: {e:?}"));
					pb.finish_with_message("Failed!");
					Err(e)
				}
			}
		}
	}
}
