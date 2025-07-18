use crate::config::SmsGgConfig;
use std::fs;

use crate::mainloop::save::FsSaveWriter;
use crate::mainloop::{debug, file_name_no_ext, save};
use crate::{AudioError, NativeEmulator, NativeEmulatorError, NativeEmulatorResult, extensions};

use jgenesis_native_config::common::WindowSize;
use smsgg_config::SmsGgInputs;
use smsgg_core::{SmsGgEmulator, SmsGgHardware};
use std::path::{Path, PathBuf};

pub type NativeSmsGgEmulator = NativeEmulator<SmsGgEmulator>;

impl NativeSmsGgEmulator {
    /// # Errors
    ///
    /// This method will return an error if it is unable to reload audio config.
    pub fn reload_smsgg_config(&mut self, config: Box<SmsGgConfig>) -> Result<(), AudioError> {
        log::info!("Reloading config: {config}");

        self.reload_common_config(&config.common)?;

        self.update_emulator_config(&config.emulator_config);

        self.input_mapper.update_mappings(
            config.common.axis_deadzone,
            &config.inputs.to_mapping_vec(),
            &config.common.hotkey_config.to_mapping_vec(),
        );

        Ok(())
    }
}

/// Create an emulator with the SMS/GG core with the given config.
///
/// # Errors
///
/// This function will propagate any video, audio, or disk errors encountered.
pub fn create_smsgg(config: Box<SmsGgConfig>) -> NativeEmulatorResult<NativeSmsGgEmulator> {
    log::info!("Running with config: {config}");

    let rom: Option<Vec<u8>>;
    let extension: String;
    let save_path: PathBuf;
    let save_state_path: PathBuf;
    let hardware: SmsGgHardware;
    let rom_title: String;

    let run_without_cartridge = config.boot_from_bios && config.run_without_cartridge;
    if !run_without_cartridge {
        let rom_path = Path::new(&config.common.rom_file_path);

        let rom_read_result = config.common.read_rom_file(&extensions::SMSGG)?;
        rom = Some(rom_read_result.rom);
        extension = rom_read_result.extension;

        let determined_paths = save::determine_save_paths(
            &config.common.save_path,
            &config.common.state_path,
            rom_path,
            &extension,
        )?;
        save_path = determined_paths.save_path;
        save_state_path = determined_paths.save_state_path;

        hardware = hardware_for_ext(&extension);
        rom_title = file_name_no_ext(rom_path)?;
    } else {
        let Some(bios_path) = &config.bios_path else { return Err(NativeEmulatorError::SmsNoBios) };

        rom = None;
        extension = "sms".into();

        let determined_paths = save::determine_save_paths(
            &config.common.save_path,
            &config.common.state_path,
            bios_path,
            &extension,
        )?;
        save_path = determined_paths.save_path;
        save_state_path = determined_paths.save_state_path;

        hardware = SmsGgHardware::MasterSystem;
        rom_title = "(BIOS)".into();
    }

    let mut save_writer = FsSaveWriter::new(save_path);

    let window_title = format!("smsgg - {rom_title}");

    let bios_rom = if config.boot_from_bios && hardware == SmsGgHardware::MasterSystem {
        let Some(bios_path) = &config.bios_path else { return Err(NativeEmulatorError::SmsNoBios) };
        Some(fs::read(bios_path).map_err(|source| NativeEmulatorError::SmsBiosRead {
            path: bios_path.clone(),
            source,
        })?)
    } else {
        None
    };

    let emulator_config = config.emulator_config;
    let emulator =
        SmsGgEmulator::create(rom, bios_rom, hardware, emulator_config, &mut save_writer);

    let default_window_size = match hardware {
        SmsGgHardware::MasterSystem => {
            WindowSize::new_sms(config.common.initial_window_size, emulator_config.sms_aspect_ratio)
        }
        SmsGgHardware::GameGear => WindowSize::new_game_gear(
            config.common.initial_window_size,
            emulator_config.gg_aspect_ratio,
        ),
    };

    NativeSmsGgEmulator::new(
        emulator,
        emulator_config,
        config.common,
        extension,
        default_window_size,
        &window_title,
        save_writer,
        save_state_path,
        &config.inputs.to_mapping_vec(),
        SmsGgInputs::default(),
        debug::smsgg::render_fn,
    )
}

fn hardware_for_ext(extension: &str) -> SmsGgHardware {
    match extension.to_ascii_lowercase().as_str() {
        "sms" => SmsGgHardware::MasterSystem,
        "gg" => SmsGgHardware::GameGear,
        _ => {
            log::error!("Unrecognized file extension '{extension}', defaulting to SMS mode");
            SmsGgHardware::MasterSystem
        }
    }
}
