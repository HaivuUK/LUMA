//! Bevy and egui viewer, that builds a full GUI
//!
//! Callable with (--gui) and as standalone windows for the CLI
//! Submodules are split by responsibility and glob imported into this module for convenience

mod egui_root;

use egui_root::*;

use crate::volume::Volume;
use anyhow::Result;
use bevy::prelude::*;
use bevy::winit::{WINIT_WINDOWS};
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::ecs::system::NonSendMarker;
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui,
};
use winit::window::Icon;

#[derive(Resource)]
pub struct LumaGuiApp {
    pub is_standalone: bool, // GUI vs CLI mode
}

impl LumaGuiApp {
    pub fn new_gui() -> Self {
        Self {
            is_standalone: false,
        }
    }

    pub fn new_standalone() -> Self {
        let mut app = Self::new_gui();
        app.is_standalone = true;
        app
    }
}

pub fn start_gui() -> Result<()> {
    run_bevy_app(LumaGuiApp::new_gui(), (1280, 720), "LUMA - Bone Material Assignment Tool")
}

static ICON_PNG: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/icon.png"));

fn set_window_icon(_main_thread: NonSendMarker,) {
    let image = match image::load_from_memory(ICON_PNG) {
        Ok(img) => img.into_rgba8(),
        Err(e) => {
            log::warn!("Failed to decode window icon: {}", e);
            return;
        }
    };

    let (width, height) = image.dimensions();

    let icon = Icon::from_rgba(image.into_raw(), width, height)
        .expect("Failed to create icon");

    WINIT_WINDOWS.with_borrow(|windows|{
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    })
}

fn run_bevy_app(app_state: LumaGuiApp, window_size: (u32, u32), title: &str) -> Result<()> {
    let _ = env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .try_init();

    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(0x22, 0x22, 0x22)))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 700.0,
            affects_lightmapped_meshes: true,
        })
        .insert_resource(app_state)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: title.to_string(),
                resolution: WindowResolution::new(window_size.0, window_size.1),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
        .add_systems(EguiPrimaryContextPass, render_egui_ui)
        .add_systems(Startup, set_window_icon)
        .run();

    Ok(())
}