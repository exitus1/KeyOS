// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
pub mod messages;

use std::time::Duration;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use server::{AsScalar, CheckedConn, CheckedPermissions, FromScalar, MessageAllowed};

use crate::messages::*;

#[macro_export]
macro_rules! use_api {
    () => {
        mod haptics_permissions {
            use haptics::messages::*;
            #[derive(Clone, Default, server::Permissions)]
            #[server_name = "os/haptics"]
            pub struct HapticsPermissions;
        }
        type HapticsApi = haptics::HapticsApi<haptics_permissions::HapticsPermissions>;
    };
}

#[derive(Debug, Copy, Clone, ToPrimitive, FromPrimitive)]
pub enum HapticPattern {
    Click,
    StrongClick,
    DoubleClick,
    StrongClick100,
    StrongClick60,
    StrongClick30,
    SharpClick100,
    SharpClick60,
    SharpClick30,
    SoftBump100,
    SoftBump60,
    SoftBump30,
    DoubleClick100,
    DoubleClick60,
    TripleClick100,
    SoftFuzz60,
    StrongBuzz100,
    Alert750ms,
    Alert1000ms,
    StrongClickOne100,
    StrongClickTwo80,
    StrongClickThree60,
    StrongClickFour30,
    MediumClickOne100,
    MediumClickTwo80,
    MediumClickThree60,
    SharpTickOne100,
    SharpTickTwo80,
    SharpTickThree60,
    ShortDoubleClickStrongOne100,
    ShortDoubleClickStrongTwo80,
    ShortDoubleClickStrongThree60,
    ShortDoubleClickStrongFour30,
    ShortDoubleClickMediumOne100,
    ShortDoubleClickMediumTwo80,
    ShortDoubleClickMediumThree60,
    ShortDoubleSharpTickOne100,
    ShortDoubleSharpTickTwo80,
    ShortDoubleSharpTickThree60,
    LongDoubleSharpClickStrongOne100,
    LongDoubleSharpClickStrongTwo80,
    LongDoubleSharpClickStrongThree60,
    LongDoubleSharpClickStrongFour30,
    LongDoubleSharpClickMediumOne100,
    LongDoubleSharpClickMediumTwo80,
    LongDoubleSharpClickMediumThree60,
    LongDoubleSharpTickOne100,
    LongDoubleSharpTickTwo80,
    LongDoubleSharpTickThree60,
    BuzzOne100,
    BuzzTwo80,
    BuzzThree60,
    BuzzFour40,
    BuzzFive20,
    PulsingStrongOne100,
    PulsingStrongTwo60,
    PulsingMediumOne100,
    PulsingMediumTwo60,
    PulsingSharpOne100,
    PulsingSharpTwo60,
    TransitionClickOne100,
    TransitionClickTwo80,
    TransitionClickThree60,
    TransitionClickFour40,
    TransitionClickFive20,
    TransitionClickSix10,
    TransitionHumOne100,
    TransitionHumTwo80,
    TransitionHumThree60,
    TransitionHumFour40,
    TransitionHumFive20,
    TransitionHumSix10,
    TransitionRampDownLongSmoothOne100to0,
    TransitionRampDownLongSmoothTwo100to0,
    TransitionRampDownMediumSmoothOne100to0,
    TransitionRampDownMediumSmoothTwo100to0,
    TransitionRampDownShortSmoothOne100to0,
    TransitionRampDownShortSmoothTwo100to0,
    TransitionRampDownLongSharpOne100to0,
    TransitionRampDownLongSharpTwo100to0,
    TransitionRampDownMediumSharpOne100to0,
    TransitionRampDownMediumSharpTwo100to0,
    TransitionRampDownShortSharpOne100to0,
    TransitionRampDownShortSharpTwo100to0,
    TransitionRampUpLongSmoothOne0to100,
    TransitionRampUpLongSmoothTwo0to100,
    TransitionRampUpMediumSmoothOne0to100,
    TransitionRampUpMediumSmoothTwo0to100,
    TransitionRampUpShortSmoothOne0to100,
    TransitionRampUpShortSmoothTwo0to100,
    TransitionRampUpLongSharpOne0to100,
    TransitionRampUpLongSharpTwo0to100,
    TransitionRampUpMediumSharpOne0to100,
    TransitionRampUpMediumSharpTwo0to100,
    TransitionRampUpShortSharpOne0to100,
    TransitionRampUpShortSharpTwo0to100,
    TransitionRampDownLongSmoothOne50to0,
    TransitionRampDownLongSmoothTwo50to0,
    TransitionRampDownMediumSmoothOne50to0,
    TransitionRampDownMediumSmoothTwo50to0,
    TransitionRampDownShortSmoothOne50to0,
    TransitionRampDownShortSmoothTwo50to0,
    TransitionRampDownLongSharpOne50to0,
    TransitionRampDownLongSharpTwo50to0,
    TransitionRampDownMediumSharpOne50to0,
    TransitionRampDownMediumSharpTwo50to0,
    TransitionRampDownShortSharpOne50to0,
    TransitionRampDownShortSharpTwo50to0,
    TransitionRampUpLongSmoothOne0to50,
    TransitionRampUpLongSmoothTwo0to50,
    TransitionRampUpMediumSmoothOne0to50,
    TransitionRampUpMediumSmoothTwo0to50,
    TransitionRampUpShortSmoothOne0to50,
    TransitionRampUpShortSmoothTwo0to50,
    TransitionRampUpLongSharpOne0to50,
    TransitionRampUpLongSharpTwo0to50,
    TransitionRampUpMediumSharpOne0to50,
    TransitionRampUpMediumSharpTwo0to50,
    TransitionRampUpShortSharpOne0to50,
    TransitionRampUpShortSharpTwo0to50,
    LongBuzzForProgrammaticStopping100,
    SmoothHumOne50,
    SmoothHumTwo40,
    SmoothHumThree30,
    SmoothHumFour20,
    SmoothHumFive10,
}

impl HapticPattern {
    pub fn from_string(s: &str) -> Option<HapticPattern> {
        match s {
            "Click" => Some(HapticPattern::Click),
            "StrongClick" => Some(HapticPattern::StrongClick),
            "DoubleClick" => Some(HapticPattern::DoubleClick),
            "StrongClick100" => Some(HapticPattern::StrongClick100),
            "StrongClick60" => Some(HapticPattern::StrongClick60),
            "StrongClick30" => Some(HapticPattern::StrongClick30),
            "SharpClick100" => Some(HapticPattern::SharpClick100),
            "SharpClick60" => Some(HapticPattern::SharpClick60),
            "SharpClick30" => Some(HapticPattern::SharpClick30),
            "SoftBump100" => Some(HapticPattern::SoftBump100),
            "SoftBump60" => Some(HapticPattern::SoftBump60),
            "SoftBump30" => Some(HapticPattern::SoftBump30),
            "DoubleClick100" => Some(HapticPattern::DoubleClick100),
            "DoubleClick60" => Some(HapticPattern::DoubleClick60),
            "TripleClick100" => Some(HapticPattern::TripleClick100),
            "SoftFuzz60" => Some(HapticPattern::SoftFuzz60),
            "StrongBuzz100" => Some(HapticPattern::StrongBuzz100),
            "Alert750ms" => Some(HapticPattern::Alert750ms),
            "Alert1000ms" => Some(HapticPattern::Alert1000ms),
            "StrongClickOne100" => Some(HapticPattern::StrongClickOne100),
            "StrongClickTwo80" => Some(HapticPattern::StrongClickTwo80),
            "StrongClickThree60" => Some(HapticPattern::StrongClickThree60),
            "StrongClickFour30" => Some(HapticPattern::StrongClickFour30),
            "MediumClickOne100" => Some(HapticPattern::MediumClickOne100),
            "MediumClickTwo80" => Some(HapticPattern::MediumClickTwo80),
            "MediumClickThree60" => Some(HapticPattern::MediumClickThree60),
            "SharpTickOne100" => Some(HapticPattern::SharpTickOne100),
            "SharpTickTwo80" => Some(HapticPattern::SharpTickTwo80),
            "SharpTickThree60" => Some(HapticPattern::SharpTickThree60),
            "ShortDoubleClickStrongOne100" => Some(HapticPattern::ShortDoubleClickStrongOne100),
            "ShortDoubleClickStrongTwo80" => Some(HapticPattern::ShortDoubleClickStrongTwo80),
            "ShortDoubleClickStrongThree60" => Some(HapticPattern::ShortDoubleClickStrongThree60),
            "ShortDoubleClickStrongFour30" => Some(HapticPattern::ShortDoubleClickStrongFour30),
            "ShortDoubleClickMediumOne100" => Some(HapticPattern::ShortDoubleClickMediumOne100),
            "ShortDoubleClickMediumTwo80" => Some(HapticPattern::ShortDoubleClickMediumTwo80),
            "ShortDoubleClickMediumThree60" => Some(HapticPattern::ShortDoubleClickMediumThree60),
            "ShortDoubleSharpTickOne100" => Some(HapticPattern::ShortDoubleSharpTickOne100),
            "ShortDoubleSharpTickTwo80" => Some(HapticPattern::ShortDoubleSharpTickTwo80),
            "ShortDoubleSharpTickThree60" => Some(HapticPattern::ShortDoubleSharpTickThree60),
            "LongDoubleSharpClickStrongOne100" => Some(HapticPattern::LongDoubleSharpClickStrongOne100),
            "LongDoubleSharpClickStrongTwo80" => Some(HapticPattern::LongDoubleSharpClickStrongTwo80),
            "LongDoubleSharpClickStrongThree60" => Some(HapticPattern::LongDoubleSharpClickStrongThree60),
            "LongDoubleSharpClickStrongFour30" => Some(HapticPattern::LongDoubleSharpClickStrongFour30),
            "LongDoubleSharpClickMediumOne100" => Some(HapticPattern::LongDoubleSharpClickMediumOne100),
            "LongDoubleSharpClickMediumTwo80" => Some(HapticPattern::LongDoubleSharpClickMediumTwo80),
            "LongDoubleSharpClickMediumThree60" => Some(HapticPattern::LongDoubleSharpClickMediumThree60),
            "LongDoubleSharpTickOne100" => Some(HapticPattern::LongDoubleSharpTickOne100),
            "LongDoubleSharpTickTwo80" => Some(HapticPattern::LongDoubleSharpTickTwo80),
            "LongDoubleSharpTickThree60" => Some(HapticPattern::LongDoubleSharpTickThree60),
            "BuzzOne100" => Some(HapticPattern::BuzzOne100),
            "BuzzTwo80" => Some(HapticPattern::BuzzTwo80),
            "BuzzThree60" => Some(HapticPattern::BuzzThree60),
            "BuzzFour40" => Some(HapticPattern::BuzzFour40),
            "BuzzFive20" => Some(HapticPattern::BuzzFive20),
            "PulsingStrongOne100" => Some(HapticPattern::PulsingStrongOne100),
            "PulsingStrongTwo60" => Some(HapticPattern::PulsingStrongTwo60),
            "PulsingMediumOne100" => Some(HapticPattern::PulsingMediumOne100),
            "PulsingMediumTwo60" => Some(HapticPattern::PulsingMediumTwo60),
            "PulsingSharpOne100" => Some(HapticPattern::PulsingSharpOne100),
            "PulsingSharpTwo60" => Some(HapticPattern::PulsingSharpTwo60),
            "TransitionClickOne100" => Some(HapticPattern::TransitionClickOne100),
            "TransitionClickTwo80" => Some(HapticPattern::TransitionClickTwo80),
            "TransitionClickThree60" => Some(HapticPattern::TransitionClickThree60),
            "TransitionClickFour40" => Some(HapticPattern::TransitionClickFour40),
            "TransitionClickFive20" => Some(HapticPattern::TransitionClickFive20),
            "TransitionClickSix10" => Some(HapticPattern::TransitionClickSix10),
            "TransitionHumOne100" => Some(HapticPattern::TransitionHumOne100),
            "TransitionHumTwo80" => Some(HapticPattern::TransitionHumTwo80),
            "TransitionHumThree60" => Some(HapticPattern::TransitionHumThree60),
            "TransitionHumFour40" => Some(HapticPattern::TransitionHumFour40),
            "TransitionHumFive20" => Some(HapticPattern::TransitionHumFive20),
            "TransitionHumSix10" => Some(HapticPattern::TransitionHumSix10),
            "TransitionRampDownLongSmoothOne100to0" => {
                Some(HapticPattern::TransitionRampDownLongSmoothOne100to0)
            }
            "TransitionRampDownLongSmoothTwo100to0" => {
                Some(HapticPattern::TransitionRampDownLongSmoothTwo100to0)
            }
            "TransitionRampDownMediumSmoothOne100to0" => {
                Some(HapticPattern::TransitionRampDownMediumSmoothOne100to0)
            }
            "TransitionRampDownMediumSmoothTwo100to0" => {
                Some(HapticPattern::TransitionRampDownMediumSmoothTwo100to0)
            }
            "TransitionRampDownShortSmoothOne100to0" => {
                Some(HapticPattern::TransitionRampDownShortSmoothOne100to0)
            }
            "TransitionRampDownShortSmoothTwo100to0" => {
                Some(HapticPattern::TransitionRampDownShortSmoothTwo100to0)
            }
            "TransitionRampDownLongSharpOne100to0" => {
                Some(HapticPattern::TransitionRampDownLongSharpOne100to0)
            }
            "TransitionRampDownLongSharpTwo100to0" => {
                Some(HapticPattern::TransitionRampDownLongSharpTwo100to0)
            }
            "TransitionRampDownMediumSharpOne100to0" => {
                Some(HapticPattern::TransitionRampDownMediumSharpOne100to0)
            }
            "TransitionRampDownMediumSharpTwo100to0" => {
                Some(HapticPattern::TransitionRampDownMediumSharpTwo100to0)
            }
            "TransitionRampDownShortSharpOne100to0" => {
                Some(HapticPattern::TransitionRampDownShortSharpOne100to0)
            }
            "TransitionRampDownShortSharpTwo100to0" => {
                Some(HapticPattern::TransitionRampDownShortSharpTwo100to0)
            }
            "TransitionRampUpLongSmoothOne0to100" => Some(HapticPattern::TransitionRampUpLongSmoothOne0to100),
            "TransitionRampUpLongSmoothTwo0to100" => Some(HapticPattern::TransitionRampUpLongSmoothTwo0to100),
            "TransitionRampUpMediumSmoothOne0to100" => {
                Some(HapticPattern::TransitionRampUpMediumSmoothOne0to100)
            }
            "TransitionRampUpMediumSmoothTwo0to100" => {
                Some(HapticPattern::TransitionRampUpMediumSmoothTwo0to100)
            }
            "TransitionRampUpShortSmoothOne0to100" => {
                Some(HapticPattern::TransitionRampUpShortSmoothOne0to100)
            }
            "TransitionRampUpShortSmoothTwo0to100" => {
                Some(HapticPattern::TransitionRampUpShortSmoothTwo0to100)
            }
            "TransitionRampUpLongSharpOne0to100" => Some(HapticPattern::TransitionRampUpLongSharpOne0to100),
            "TransitionRampUpLongSharpTwo0to100" => Some(HapticPattern::TransitionRampUpLongSharpTwo0to100),
            "TransitionRampUpMediumSharpOne0to100" => {
                Some(HapticPattern::TransitionRampUpMediumSharpOne0to100)
            }
            "TransitionRampUpMediumSharpTwo0to100" => {
                Some(HapticPattern::TransitionRampUpMediumSharpTwo0to100)
            }
            "TransitionRampUpShortSharpOne0to100" => Some(HapticPattern::TransitionRampUpShortSharpOne0to100),
            "TransitionRampUpShortSharpTwo0to100" => Some(HapticPattern::TransitionRampUpShortSharpTwo0to100),
            "TransitionRampDownLongSmoothOne50to0" => {
                Some(HapticPattern::TransitionRampDownLongSmoothOne50to0)
            }
            "TransitionRampDownLongSmoothTwo50to0" => {
                Some(HapticPattern::TransitionRampDownLongSmoothTwo50to0)
            }
            "TransitionRampDownMediumSmoothOne50to0" => {
                Some(HapticPattern::TransitionRampDownMediumSmoothOne50to0)
            }
            "TransitionRampDownMediumSmoothTwo50to0" => {
                Some(HapticPattern::TransitionRampDownMediumSmoothTwo50to0)
            }
            "TransitionRampDownShortSmoothOne50to0" => {
                Some(HapticPattern::TransitionRampDownShortSmoothOne50to0)
            }
            "TransitionRampDownShortSmoothTwo50to0" => {
                Some(HapticPattern::TransitionRampDownShortSmoothTwo50to0)
            }
            "TransitionRampDownLongSharpOne50to0" => Some(HapticPattern::TransitionRampDownLongSharpOne50to0),
            "TransitionRampDownLongSharpTwo50to0" => Some(HapticPattern::TransitionRampDownLongSharpTwo50to0),
            "TransitionRampDownMediumSharpOne50to0" => {
                Some(HapticPattern::TransitionRampDownMediumSharpOne50to0)
            }
            "TransitionRampDownMediumSharpTwo50to0" => {
                Some(HapticPattern::TransitionRampDownMediumSharpTwo50to0)
            }
            "TransitionRampDownShortSharpOne50to0" => {
                Some(HapticPattern::TransitionRampDownShortSharpOne50to0)
            }
            "TransitionRampDownShortSharpTwo50to0" => {
                Some(HapticPattern::TransitionRampDownShortSharpTwo50to0)
            }
            "TransitionRampUpLongSmoothOne0to50" => Some(HapticPattern::TransitionRampUpLongSmoothOne0to50),
            "TransitionRampUpLongSmoothTwo0to50" => Some(HapticPattern::TransitionRampUpLongSmoothTwo0to50),
            "TransitionRampUpMediumSmoothOne0to50" => {
                Some(HapticPattern::TransitionRampUpMediumSmoothOne0to50)
            }
            "TransitionRampUpMediumSmoothTwo0to50" => {
                Some(HapticPattern::TransitionRampUpMediumSmoothTwo0to50)
            }
            "TransitionRampUpShortSmoothOne0to50" => Some(HapticPattern::TransitionRampUpShortSmoothOne0to50),
            "TransitionRampUpShortSmoothTwo0to50" => Some(HapticPattern::TransitionRampUpShortSmoothTwo0to50),
            "TransitionRampUpLongSharpOne0to50" => Some(HapticPattern::TransitionRampUpLongSharpOne0to50),
            "TransitionRampUpLongSharpTwo0to50" => Some(HapticPattern::TransitionRampUpLongSharpTwo0to50),
            "TransitionRampUpMediumSharpOne0to50" => Some(HapticPattern::TransitionRampUpMediumSharpOne0to50),
            "TransitionRampUpMediumSharpTwo0to50" => Some(HapticPattern::TransitionRampUpMediumSharpTwo0to50),
            "TransitionRampUpShortSharpOne0to50" => Some(HapticPattern::TransitionRampUpShortSharpOne0to50),
            "TransitionRampUpShortSharpTwo0to50" => Some(HapticPattern::TransitionRampUpShortSharpTwo0to50),
            "LongBuzzForProgrammaticStopping100" => Some(HapticPattern::LongBuzzForProgrammaticStopping100),
            "SmoothHumOne50" => Some(HapticPattern::SmoothHumOne50),
            "SmoothHumTwo40" => Some(HapticPattern::SmoothHumTwo40),
            "SmoothHumThree30" => Some(HapticPattern::SmoothHumThree30),
            "SmoothHumFour20" => Some(HapticPattern::SmoothHumFour20),
            "SmoothHumFive10" => Some(HapticPattern::SmoothHumFive10),
            _ => None, // Return None if the string doesn't match any known pattern
        }
    }
}

#[derive(Default)]
pub struct HapticsApi<P: CheckedPermissions> {
    conn: CheckedConn<P>,
}

impl<P: CheckedPermissions> HapticsApi<P> {
    pub fn try_new_with_timeout(timeout: Duration) -> Option<Self> {
        Some(Self { conn: CheckedConn::try_connect_with_timeout(timeout)? })
    }

    pub fn click(&self)
    where
        P: MessageAllowed<Vibrate>,
    {
        self.vibrate(HapticPattern::Click);
    }

    pub fn triple_click(&self)
    where
        P: MessageAllowed<Vibrate>,
    {
        self.vibrate(HapticPattern::TripleClick100);
    }

    pub fn strong_click(&self)
    where
        P: MessageAllowed<Vibrate>,
    {
        self.vibrate(HapticPattern::StrongClick);
    }

    pub fn double_click(&self)
    where
        P: MessageAllowed<Vibrate>,
    {
        self.vibrate(HapticPattern::DoubleClick);
    }

    pub fn vibrate(&self, haptic_pattern: HapticPattern)
    where
        P: MessageAllowed<Vibrate>,
    {
        self.conn.try_send_scalar(Vibrate(haptic_pattern)).ok();
    }
}

impl AsScalar<1> for HapticPattern {
    fn as_scalar(&self) -> [u32; 1] { [self.to_u32().unwrap()] }
}

impl FromScalar<1> for HapticPattern {
    fn from_scalar([value]: [u32; 1]) -> Self { Self::from_u32(value).unwrap_or(HapticPattern::Click) }
}
