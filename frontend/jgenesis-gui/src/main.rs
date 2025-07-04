use clap::Parser;
use eframe::NativeOptions;
use egui::{Vec2, ViewportBuilder};
use env_logger::Env;
use jgenesis_gui::app::{App, LoadAtStartup};
use jgenesis_native_config::AppConfig;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
struct Args {
    /// Use a specific config file path instead of the default path of 'jgenesis-config.toml'
    #[arg(long = "config")]
    config_path: Option<PathBuf>,

    /// If set, the GUI will open this file immediately after starting up, and the GUI will exit
    /// when the emulator window is closed
    #[arg(long = "file-path", short = 'f')]
    startup_file_path: Option<PathBuf>,

    /// In combination with -f, attempt to load the specified save state when launching the game.
    /// This arg has no effect if -f/--file-path is not set
    #[arg(long, value_name = "SLOT")]
    load_save_state: Option<usize>,
}

impl Args {
    fn fix_appimage_relative_paths(mut self) -> Self {
        if let Some(config_path) = self.config_path {
            self.config_path = Some(jgenesis_common::fix_appimage_relative_path(config_path));
        }

        if let Some(startup_file_path) = self.startup_file_path {
            self.startup_file_path =
                Some(jgenesis_common::fix_appimage_relative_path(startup_file_path));
        }

        self
    }

    fn load_at_startup(&self) -> Option<LoadAtStartup> {
        self.startup_file_path.as_ref().map(|file_path| LoadAtStartup {
            file_path: file_path.clone(),
            load_state_slot: self.load_save_state,
        })
    }
}

// Attempt to detect if the application is running on a Steam Deck, and if it is then override
// the winit scale factor to 1. It defaults to 4.5 on the Steam Deck which results in the GUI
// being completely unusable.
#[cfg(all(unix, not(target_os = "macos")))]
fn steam_deck_dpi_hack() {
    let Ok(mut xhandle) = xrandr::XHandle::open() else {
        return;
    };
    let Ok(monitors) = xhandle.monitors() else {
        return;
    };

    if monitors.len() != 1 {
        return;
    }

    let monitor = &monitors[0];

    if monitor.width_px != 1280 || monitor.height_px != 800 || monitor.outputs.len() != 1 {
        return;
    }

    let output = &monitor.outputs[0];

    let Some(edid) = output.properties.iter().find_map(|(_, property)| match &property.value {
        xrandr::Value::Edid(edid) => Some(edid),
        _ => None,
    }) else {
        return;
    };

    // Display name part of the EDID is always here on the Steam Deck: 'ANX7530 U<LF>'
    if edid[75..87] == [0xFC, 0x00, 0x41, 0x4E, 0x58, 0x37, 0x35, 0x33, 0x30, 0x20, 0x55, 0x0A] {
        log::info!(
            "It looks like this is a Steam Deck; overriding winit scale factor to 1 as otherwise it will default to 4.5"
        );

        // SAFETY: This function is only called during initialization, before spawning any threads
        unsafe {
            std::env::set_var("WINIT_X11_SCALE_FACTOR", "1");
        }
    }
}

fn initial_gui_size(config_path: &Path) -> (f32, f32) {
    let mut gui_width = jgenesis_native_config::DEFAULT_GUI_WIDTH;
    let mut gui_height = jgenesis_native_config::DEFAULT_GUI_HEIGHT;

    if let Some(config) = fs::read_to_string(config_path)
        .ok()
        .and_then(|config_str| toml::from_str::<AppConfig>(&config_str).ok())
    {
        gui_width = f32_max(jgenesis_native_config::DEFAULT_GUI_WIDTH, config.gui_window_width);
        gui_height = f32_max(jgenesis_native_config::DEFAULT_GUI_HEIGHT, config.gui_window_height);
    }

    (gui_width, gui_height)
}

fn f32_max(value: f32, max: f32) -> f32 {
    if value < max { max } else { value }
}

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(
        Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn"),
    )
    .init();

    let args = Args::parse().fix_appimage_relative_paths();

    #[cfg(all(unix, not(target_os = "macos")))]
    steam_deck_dpi_hack();

    let config_path =
        args.config_path.clone().unwrap_or_else(jgenesis_native_config::default_config_path);
    log::info!("Using config path '{}'", config_path.display());

    if let Some(file_path) = &args.startup_file_path {
        log::info!("Will open file '{}' after starting", file_path.display());
    }

    let (gui_width, gui_height) = initial_gui_size(&config_path);

    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(gui_width, gui_height)),
        ..NativeOptions::default()
    };

    let load_at_startup = args.load_at_startup();
    eframe::run_native(
        "jgenesis",
        options,
        Box::new(|cc| Ok(Box::new(App::new(config_path, load_at_startup, cc.egui_ctx.clone())))),
    )
}
