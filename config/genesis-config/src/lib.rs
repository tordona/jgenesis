use bincode::{Decode, Encode};
use jgenesis_common::define_controller_inputs;
use jgenesis_common::frontend::{FrameSize, PixelAspectRatio, TimingMode};
use jgenesis_proc_macros::{EnumAll, EnumDisplay, EnumFromStr};

pub const NATIVE_M68K_DIVIDER: u64 = 7;

pub const DEFAULT_SUB_CPU_DIVIDER: u64 = 4;

pub const MODEL_1_VA2_LPF_CUTOFF: u32 = 3390;
pub const MODEL_1_VA3_LPF_CUTOFF: u32 = 2840;
pub const MODEL_2_1ST_LPF_CUTOFF: u32 = 3789;
pub const MODEL_2_2ND_LPF_CUTOFF: u32 = 6725;

pub const DEFAULT_PCM_LPF_CUTOFF: u32 = 7973;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode, EnumDisplay, EnumFromStr, EnumAll,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "clap", derive(jgenesis_proc_macros::CustomValueEnum))]
pub enum GenesisAspectRatio {
    #[default]
    Auto,
    Ntsc,
    Pal,
    SquarePixels,
    Stretched,
}

impl GenesisAspectRatio {
    #[inline]
    #[must_use]
    pub fn to_h40_pixel_aspect_ratio(self, timing_mode: TimingMode) -> Option<f64> {
        if self == Self::Auto {
            let auto_aspect = match timing_mode {
                TimingMode::Ntsc => Self::Ntsc,
                TimingMode::Pal => Self::Pal,
            };
            return auto_aspect.to_h40_pixel_aspect_ratio(timing_mode);
        }

        match self {
            Self::Ntsc => Some(32.0 / 35.0),
            Self::Pal => Some(11.0 / 10.0),
            Self::SquarePixels => Some(1.0),
            Self::Stretched => None,
            Self::Auto => unreachable!("Auto checked at start of function with early return"),
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_pixel_aspect_ratio(
        self,
        timing_mode: TimingMode,
        frame_size: FrameSize,
        adjust_for_2x_resolution: bool,
    ) -> Option<PixelAspectRatio> {
        if self == Self::Auto {
            let auto_aspect = match timing_mode {
                TimingMode::Ntsc => Self::Ntsc,
                TimingMode::Pal => Self::Pal,
            };
            return auto_aspect.to_pixel_aspect_ratio(
                timing_mode,
                frame_size,
                adjust_for_2x_resolution,
            );
        }

        let mut pixel_aspect_ratio = match (self, frame_size.width) {
            (Self::SquarePixels, _) => Some(1.0),
            (Self::Stretched, _) => None,
            (Self::Ntsc, 256..=284) => Some(8.0 / 7.0),
            (Self::Ntsc, 320..=347) => Some(32.0 / 35.0),
            (Self::Pal, 256..=284) => Some(11.0 / 8.0),
            (Self::Pal, 320..=347) => Some(11.0 / 10.0),
            (Self::Ntsc | Self::Pal, _) => {
                log::error!("unexpected Genesis frame width: {}", frame_size.width);
                None
            }
            (Self::Auto, _) => unreachable!("Auto checked at start of function with early return"),
        };

        if adjust_for_2x_resolution && frame_size.height >= 448 {
            pixel_aspect_ratio = pixel_aspect_ratio.map(|par| par * 2.0);
        }

        pixel_aspect_ratio.map(|par| PixelAspectRatio::try_from(par).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, EnumDisplay, EnumAll)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "clap", derive(jgenesis_proc_macros::CustomValueEnum))]
pub enum GenesisRegion {
    Americas,
    Japan,
    Europe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode, EnumAll, EnumDisplay)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "clap", derive(jgenesis_proc_macros::CustomValueEnum))]
pub enum Opn2BusyBehavior {
    Ym2612,
    #[default]
    Ym3438,
    AlwaysZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode, EnumDisplay, EnumAll)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "clap", derive(jgenesis_proc_macros::CustomValueEnum))]
pub enum GenesisControllerType {
    ThreeButton,
    #[default]
    SixButton,
    None,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode, EnumDisplay, EnumFromStr, EnumAll,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "clap", derive(jgenesis_proc_macros::CustomValueEnum))]
pub enum PcmInterpolation {
    #[default]
    None,
    Linear,
    CubicHermite,
    CubicHermite6Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode, EnumDisplay, EnumAll)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "clap", derive(jgenesis_proc_macros::CustomValueEnum))]
pub enum S32XVideoOut {
    #[default]
    Combined,
    GenesisOnly,
    S32XOnly,
}

define_controller_inputs! {
    buttons: GenesisButton {
        Up -> up,
        Left -> left,
        Right -> right,
        Down -> down,
        A -> a,
        B -> b,
        C -> c,
        X -> x,
        Y -> y,
        Z -> z,
        Start -> start,
        Mode -> mode,
    },
    joypad: GenesisJoypadState,
    inputs: GenesisInputs {
        players: {
            p1: Player::One,
            p2: Player::Two,
        },
    },
}
