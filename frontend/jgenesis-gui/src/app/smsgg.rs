mod helptext;

use crate::app::{App, OpenWindow};
use crate::emuthread::EmuThreadStatus;
use crate::widgets::OverclockSlider;
use egui::{Context, Window};
use jgenesis_common::frontend::TimingMode;
use jgenesis_native_driver::extensions::Console;
use rfd::FileDialog;
use smsgg_config::{GgAspectRatio, SmsAspectRatio, SmsGgRegion, SmsModel, Sn76489Version};
use std::num::NonZeroU32;
use std::path::PathBuf;

impl App {
    pub(super) fn render_smsgg_general_settings(&mut self, ctx: &Context) {
        const WINDOW: OpenWindow = OpenWindow::SmsGgGeneral;

        let mut open = true;
        Window::new("SMS/GG General Settings").open(&mut open).resizable(false).show(ctx, |ui| {
            let rect = ui
                .group(|ui| {
                    ui.label("Master System timing / display mode");

                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.config.smsgg.sms_timing_mode,
                            TimingMode::Ntsc,
                            "NTSC (60Hz)",
                        );
                        ui.radio_value(
                            &mut self.config.smsgg.sms_timing_mode,
                            TimingMode::Pal,
                            "PAL (50Hz)",
                        );
                    });
                })
                .response
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::TIMING_MODE);
            }

            let rect = ui
                .group(|ui| {
                    ui.label("Master System VDP version");

                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.config.smsgg.sms_model, SmsModel::Sms1, "SMS1")
                            .on_hover_text(concat!(
                                "Emulates an SMS1 quirk that is",
                                " required for the Japanese version of Ys"
                            ));

                        ui.radio_value(&mut self.config.smsgg.sms_model, SmsModel::Sms2, "SMS2");
                    });
                })
                .response
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::VDP_VERSION);
            }

            let rect = ui
                .group(|ui| {
                    ui.label("Hardware region");

                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.config.smsgg.forced_region, None, "Auto");
                        ui.radio_value(
                            &mut self.config.smsgg.forced_region,
                            Some(SmsGgRegion::International),
                            "International / Overseas",
                        );
                        ui.radio_value(
                            &mut self.config.smsgg.forced_region,
                            Some(SmsGgRegion::Domestic),
                            "Domestic (Japan)",
                        );
                    });
                })
                .response
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::REGION);
            }

            let rect = ui
                .add(OverclockSlider {
                    label: "Z80 clock divider",
                    current_value: &mut self.config.smsgg.z80_divider,
                    range: NonZeroU32::new(1).unwrap()..=NonZeroU32::new(15).unwrap(),
                    master_clock: smsgg_core::audio::NTSC_MCLK_FREQUENCY,
                    default_divider: smsgg_core::NATIVE_Z80_DIVIDER.into(),
                })
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::Z80_OVERCLOCK);
            }

            ui.add_space(5.0);
            let rect = ui
                .checkbox(&mut self.config.smsgg.boot_from_bios, "(SMS) Boot from BIOS ROM")
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::SMS_BIOS);
            }

            let rect = ui
                .horizontal(|ui| {
                    let button_label = match &self.config.smsgg.bios_path {
                        Some(bios_path) => bios_path.to_string_lossy(),
                        None => "<None>".into(),
                    };
                    if ui.button(button_label).clicked() {
                        if let Some(bios_path) = pick_sms_bios_path() {
                            self.config.smsgg.bios_path = Some(bios_path);
                        }
                    }

                    ui.label("SMS BIOS path");
                })
                .response
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::SMS_BIOS);
            }

            self.render_help_text(ui, WINDOW);
        });
        if !open {
            self.state.open_windows.remove(&WINDOW);
        }
    }

    pub(super) fn render_smsgg_video_settings(&mut self, ctx: &Context) {
        const WINDOW: OpenWindow = OpenWindow::SmsGgVideo;

        let mut open = true;
        Window::new("SMS/GG Video Settings").open(&mut open).resizable(false).show(ctx, |ui| {
            let rect = ui
                .group(|ui| {
                    ui.label("Master System aspect ratio");

                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.config.smsgg.sms_aspect_ratio,
                            SmsAspectRatio::Ntsc,
                            "NTSC",
                        )
                        .on_hover_text("8:7 pixel aspect ratio");
                        ui.radio_value(
                            &mut self.config.smsgg.sms_aspect_ratio,
                            SmsAspectRatio::Pal,
                            "PAL",
                        )
                        .on_hover_text("11:8 pixel aspect ratio");
                        ui.radio_value(
                            &mut self.config.smsgg.sms_aspect_ratio,
                            SmsAspectRatio::SquarePixels,
                            "Square pixels",
                        )
                        .on_hover_text("1:1 pixel aspect ratio");
                        ui.radio_value(
                            &mut self.config.smsgg.sms_aspect_ratio,
                            SmsAspectRatio::Stretched,
                            "Stretched",
                        )
                        .on_hover_text("Stretch image to fill the screen");
                    });
                })
                .response
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::SMS_ASPECT_RATIO);
            }

            let rect = ui
                .group(|ui| {
                    ui.label("Game Gear aspect ratio");

                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.config.smsgg.gg_aspect_ratio,
                            GgAspectRatio::GgLcd,
                            "Game Gear LCD",
                        )
                        .on_hover_text("6:5 pixel aspect ratio");
                        ui.radio_value(
                            &mut self.config.smsgg.gg_aspect_ratio,
                            GgAspectRatio::SquarePixels,
                            "Square pixels",
                        )
                        .on_hover_text("1:1 pixel aspect ratio");
                        ui.radio_value(
                            &mut self.config.smsgg.gg_aspect_ratio,
                            GgAspectRatio::Stretched,
                            "Stretched",
                        )
                        .on_hover_text("Stretch image to fill the screen");
                    });
                })
                .response
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::GG_ASPECT_RATIO);
            }

            let rect = ui
                .checkbox(
                    &mut self.config.smsgg.remove_sprite_limit,
                    "Remove sprite-per-scanline limit",
                )
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::REMOVE_SPRITE_LIMIT);
            }

            let rect = ui
                .checkbox(
                    &mut self.config.smsgg.sms_crop_vertical_border,
                    "(SMS) Crop vertical border",
                )
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::SMS_CROP_VERTICAL_BORDER);
            }

            let rect = ui
                .checkbox(&mut self.config.smsgg.sms_crop_left_border, "(SMS) Crop left border")
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::SMS_CROP_LEFT_BORDER);
            }

            let rect = ui
                .checkbox(
                    &mut self.config.smsgg.gg_use_sms_resolution,
                    "(Game Gear) Render in SMS resolution",
                )
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::GG_USE_SMS_RESOLUTION);
            }

            self.render_help_text(ui, WINDOW);
        });
        if !open {
            self.state.open_windows.remove(&WINDOW);
        }
    }

    pub(super) fn render_smsgg_audio_settings(&mut self, ctx: &Context) {
        const WINDOW: OpenWindow = OpenWindow::SmsGgAudio;

        let mut open = true;
        Window::new("SMS/GG Audio Settings").open(&mut open).resizable(false).show(ctx, |ui| {
            let rect = ui
                .group(|ui| {
                    ui.label("PSG version");

                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.config.smsgg.psg_version, None, "Auto")
                            .on_hover_text(
                                "SMS games will use SMS2 PSG, Game Gear games will use SMS1/GG PSG",
                            );
                        ui.radio_value(
                            &mut self.config.smsgg.psg_version,
                            Some(Sn76489Version::MasterSystem2),
                            "SMS2",
                        )
                        .on_hover_text("SMS2 PSG clips high volumes");
                        ui.radio_value(
                            &mut self.config.smsgg.psg_version,
                            Some(Sn76489Version::Standard),
                            "SMS1 / Game Gear",
                        )
                        .on_hover_text("SMS1 and Game Gear PSGs correctly play high volumes");
                    });
                })
                .response
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::PSG_VERSION);
            }

            let rect = ui
                .add_enabled_ui(self.emu_thread.status() != EmuThreadStatus::RunningSmsGg, |ui| {
                    ui.checkbox(
                        &mut self.config.smsgg.fm_sound_unit_enabled,
                        "Master System FM sound unit enabled",
                    );
                })
                .response
                .interact_rect;
            if ui.rect_contains_pointer(rect) {
                self.state.help_text.insert(WINDOW, helptext::SMS_FM_UNIT);
            }

            self.render_help_text(ui, WINDOW);
        });
        if !open {
            self.state.open_windows.remove(&WINDOW);
        }
    }

    pub(super) fn render_sms_bios_error(&mut self, ctx: &Context, open: &mut bool) {
        let mut path_configured = false;
        Window::new("Missing SMS BIOS").open(open).resizable(false).show(ctx, |ui| {
            ui.label("The boot from BIOS option is set but no SMS BIOS path is configured.");

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Configure now:");
                if ui.button("Configure SMS BIOS path").clicked() {
                    if let Some(bios_path) = pick_sms_bios_path() {
                        self.config.smsgg.bios_path = Some(bios_path);
                        path_configured = true;
                    }
                }
            });
        });

        if path_configured {
            *open = false;
            self.launch_emulator(self.state.current_file_path.clone(), Some(Console::MasterSystem));
        }
    }
}

fn pick_sms_bios_path() -> Option<PathBuf> {
    FileDialog::new().add_filter("bin", &["sms", "bin"]).add_filter("All Files", &["*"]).pick_file()
}
