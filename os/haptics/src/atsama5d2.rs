// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use drv2605::Effect;
use haptics::HapticPattern;
use {
    drv2605::Drv2605,
    gpio::{GpioPin, PinSettings},
    i2c::Peripheral,
};

i2c::use_api!();
gpio::use_api!();

pub struct Implementation {
    hfb: Drv2605<I2cPeripheral>,
}

impl Implementation {
    pub fn init() -> Self {
        // Claim and activate haptic feedback driver's "enable" pin
        log::debug!("Claiming haptic feedback controller EN signal");
        let gpio_api = GpioApi::default();
        gpio_api.claim_pin(GpioPin::HfbEn, PinSettings::OutputHigh, false).unwrap();

        // Claim haptic feedback I2C interface
        log::debug!("Claiming haptic feedback controller I2C peripheral");
        let i2c_api = I2cApi::default();
        let i2c_periph = i2c_api.claim_peripheral(Peripheral::HapticDriver).unwrap();

        // Connect to and initialize the haptic feedback driver chip
        log::debug!("Initializing DRV2605");
        let mut hfb = Drv2605::new(i2c_periph);
        hfb.init_open_loop_erm().unwrap();

        log::debug!("Haptics controller initialized");

        Self { hfb }
    }

    fn vibrate_fallible(&mut self, haptic_pattern: HapticPattern) -> Result<(), i2c::I2cError> {
        match haptic_pattern {
            HapticPattern::Click => self.hfb.set_single_effect(Effect::SharpClick60)?,
            HapticPattern::StrongClick => self.hfb.set_single_effect(Effect::StrongClick100)?,
            HapticPattern::DoubleClick => self.hfb.set_single_effect(Effect::DoubleClick100)?,
            HapticPattern::StrongClick100 => self.hfb.set_single_effect(Effect::StrongClick100)?,
            HapticPattern::StrongClick60 => self.hfb.set_single_effect(Effect::StrongClick60)?,
            HapticPattern::StrongClick30 => self.hfb.set_single_effect(Effect::StrongClick30)?,
            HapticPattern::SharpClick100 => self.hfb.set_single_effect(Effect::SharpClick100)?,
            HapticPattern::SharpClick60 => self.hfb.set_single_effect(Effect::SharpClick60)?,
            HapticPattern::SharpClick30 => self.hfb.set_single_effect(Effect::SharpClick30)?,
            HapticPattern::SoftBump100 => self.hfb.set_single_effect(Effect::SoftBump100)?,
            HapticPattern::SoftBump60 => self.hfb.set_single_effect(Effect::SoftBump60)?,
            HapticPattern::SoftBump30 => self.hfb.set_single_effect(Effect::SoftBump30)?,
            HapticPattern::DoubleClick100 => self.hfb.set_single_effect(Effect::DoubleClick100)?,
            HapticPattern::DoubleClick60 => self.hfb.set_single_effect(Effect::DoubleClick60)?,
            HapticPattern::TripleClick100 => self.hfb.set_single_effect(Effect::TripleClick100)?,
            HapticPattern::SoftFuzz60 => self.hfb.set_single_effect(Effect::SoftFuzz60)?,
            HapticPattern::StrongBuzz100 => self.hfb.set_single_effect(Effect::StrongBuzz100)?,
            HapticPattern::Alert750ms => self.hfb.set_single_effect(Effect::Alert750ms)?,
            HapticPattern::Alert1000ms => self.hfb.set_single_effect(Effect::Alert1000ms)?,
            HapticPattern::StrongClickOne100 => self.hfb.set_single_effect(Effect::StrongClickOne100)?,
            HapticPattern::StrongClickTwo80 => self.hfb.set_single_effect(Effect::StrongClickTwo80)?,
            HapticPattern::StrongClickThree60 => self.hfb.set_single_effect(Effect::StrongClickThree60)?,
            HapticPattern::StrongClickFour30 => self.hfb.set_single_effect(Effect::StrongClickFour30)?,
            HapticPattern::MediumClickOne100 => self.hfb.set_single_effect(Effect::MediumClickOne100)?,
            HapticPattern::MediumClickTwo80 => self.hfb.set_single_effect(Effect::MediumClickTwo80)?,
            HapticPattern::MediumClickThree60 => self.hfb.set_single_effect(Effect::MediumClickThree60)?,
            HapticPattern::SharpTickOne100 => self.hfb.set_single_effect(Effect::SharpTickOne100)?,
            HapticPattern::SharpTickTwo80 => self.hfb.set_single_effect(Effect::SharpTickTwo80)?,
            HapticPattern::SharpTickThree60 => self.hfb.set_single_effect(Effect::SharpTickThree60)?,
            HapticPattern::ShortDoubleClickStrongOne100 => {
                self.hfb.set_single_effect(Effect::ShortDoubleClickStrongOne100)?
            }
            HapticPattern::ShortDoubleClickStrongTwo80 => {
                self.hfb.set_single_effect(Effect::ShortDoubleClickStrongTwo80)?
            }
            HapticPattern::ShortDoubleClickStrongThree60 => {
                self.hfb.set_single_effect(Effect::ShortDoubleClickStrongThree60)?
            }
            HapticPattern::ShortDoubleClickStrongFour30 => {
                self.hfb.set_single_effect(Effect::ShortDoubleClickStrongFour30)?
            }
            HapticPattern::ShortDoubleClickMediumOne100 => {
                self.hfb.set_single_effect(Effect::ShortDoubleClickMediumOne100)?
            }
            HapticPattern::ShortDoubleClickMediumTwo80 => {
                self.hfb.set_single_effect(Effect::ShortDoubleClickMediumTwo80)?
            }
            HapticPattern::ShortDoubleClickMediumThree60 => {
                self.hfb.set_single_effect(Effect::ShortDoubleClickMediumThree60)?
            }
            HapticPattern::ShortDoubleSharpTickOne100 => {
                self.hfb.set_single_effect(Effect::ShortDoubleSharpTickOne100)?
            }
            HapticPattern::ShortDoubleSharpTickTwo80 => {
                self.hfb.set_single_effect(Effect::ShortDoubleSharpTickTwo80)?
            }
            HapticPattern::ShortDoubleSharpTickThree60 => {
                self.hfb.set_single_effect(Effect::ShortDoubleSharpTickThree60)?
            }
            HapticPattern::LongDoubleSharpClickStrongOne100 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpClickStrongOne100)?
            }
            HapticPattern::LongDoubleSharpClickStrongTwo80 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpClickStrongTwo80)?
            }
            HapticPattern::LongDoubleSharpClickStrongThree60 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpClickStrongThree60)?
            }
            HapticPattern::LongDoubleSharpClickStrongFour30 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpClickStrongFour30)?
            }
            HapticPattern::LongDoubleSharpClickMediumOne100 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpClickMediumOne100)?
            }
            HapticPattern::LongDoubleSharpClickMediumTwo80 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpClickMediumTwo80)?
            }
            HapticPattern::LongDoubleSharpClickMediumThree60 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpClickMediumThree60)?
            }
            HapticPattern::LongDoubleSharpTickOne100 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpTickOne100)?
            }
            HapticPattern::LongDoubleSharpTickTwo80 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpTickTwo80)?
            }
            HapticPattern::LongDoubleSharpTickThree60 => {
                self.hfb.set_single_effect(Effect::LongDoubleSharpTickThree60)?
            }
            HapticPattern::BuzzOne100 => self.hfb.set_single_effect(Effect::BuzzOne100)?,
            HapticPattern::BuzzTwo80 => self.hfb.set_single_effect(Effect::BuzzTwo80)?,
            HapticPattern::BuzzThree60 => self.hfb.set_single_effect(Effect::BuzzThree60)?,
            HapticPattern::BuzzFour40 => self.hfb.set_single_effect(Effect::BuzzFour40)?,
            HapticPattern::BuzzFive20 => self.hfb.set_single_effect(Effect::BuzzFive20)?,
            HapticPattern::PulsingStrongOne100 => self.hfb.set_single_effect(Effect::PulsingStrongOne100)?,
            HapticPattern::PulsingStrongTwo60 => self.hfb.set_single_effect(Effect::PulsingStrongTwo60)?,
            HapticPattern::PulsingMediumOne100 => self.hfb.set_single_effect(Effect::PulsingMediumOne100)?,
            HapticPattern::PulsingMediumTwo60 => self.hfb.set_single_effect(Effect::PulsingMediumTwo60)?,
            HapticPattern::PulsingSharpOne100 => self.hfb.set_single_effect(Effect::PulsingSharpOne100)?,
            HapticPattern::PulsingSharpTwo60 => self.hfb.set_single_effect(Effect::PulsingSharpTwo60)?,
            HapticPattern::TransitionClickOne100 => {
                self.hfb.set_single_effect(Effect::TransitionClickOne100)?
            }
            HapticPattern::TransitionClickTwo80 => {
                self.hfb.set_single_effect(Effect::TransitionClickTwo80)?
            }
            HapticPattern::TransitionClickThree60 => {
                self.hfb.set_single_effect(Effect::TransitionClickThree60)?
            }
            HapticPattern::TransitionClickFour40 => {
                self.hfb.set_single_effect(Effect::TransitionClickFour40)?
            }
            HapticPattern::TransitionClickFive20 => {
                self.hfb.set_single_effect(Effect::TransitionClickFive20)?
            }
            HapticPattern::TransitionClickSix10 => {
                self.hfb.set_single_effect(Effect::TransitionClickSix10)?
            }
            HapticPattern::TransitionHumOne100 => self.hfb.set_single_effect(Effect::TransitionHumOne100)?,
            HapticPattern::TransitionHumTwo80 => self.hfb.set_single_effect(Effect::TransitionHumTwo80)?,
            HapticPattern::TransitionHumThree60 => {
                self.hfb.set_single_effect(Effect::TransitionHumThree60)?
            }
            HapticPattern::TransitionHumFour40 => self.hfb.set_single_effect(Effect::TransitionHumFour40)?,
            HapticPattern::TransitionHumFive20 => self.hfb.set_single_effect(Effect::TransitionHumFive20)?,
            HapticPattern::TransitionHumSix10 => self.hfb.set_single_effect(Effect::TransitionHumSix10)?,
            HapticPattern::TransitionRampDownLongSmoothOne100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownLongSmoothOne100to0)?
            }
            HapticPattern::TransitionRampDownLongSmoothTwo100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownLongSmoothTwo100to0)?
            }
            HapticPattern::TransitionRampDownMediumSmoothOne100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownMediumSmoothOne100to0)?
            }
            HapticPattern::TransitionRampDownMediumSmoothTwo100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownMediumSmoothTwo100to0)?
            }
            HapticPattern::TransitionRampDownShortSmoothOne100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownShortSmoothOne100to0)?
            }
            HapticPattern::TransitionRampDownShortSmoothTwo100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownShortSmoothTwo100to0)?
            }
            HapticPattern::TransitionRampDownLongSharpOne100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownLongSharpOne100to0)?
            }
            HapticPattern::TransitionRampDownLongSharpTwo100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownLongSharpTwo100to0)?
            }
            HapticPattern::TransitionRampDownMediumSharpOne100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownMediumSharpOne100to0)?
            }
            HapticPattern::TransitionRampDownMediumSharpTwo100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownMediumSharpTwo100to0)?
            }
            HapticPattern::TransitionRampDownShortSharpOne100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownShortSharpOne100to0)?
            }
            HapticPattern::TransitionRampDownShortSharpTwo100to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownShortSharpTwo100to0)?
            }
            HapticPattern::TransitionRampUpLongSmoothOne0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpLongSmoothOne0to100)?
            }
            HapticPattern::TransitionRampUpLongSmoothTwo0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpLongSmoothTwo0to100)?
            }
            HapticPattern::TransitionRampUpMediumSmoothOne0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpMediumSmoothOne0to100)?
            }
            HapticPattern::TransitionRampUpMediumSmoothTwo0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpMediumSmoothTwo0to100)?
            }
            HapticPattern::TransitionRampUpShortSmoothOne0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpShortSmoothOne0to100)?
            }
            HapticPattern::TransitionRampUpShortSmoothTwo0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpShortSmoothTwo0to100)?
            }
            HapticPattern::TransitionRampUpLongSharpOne0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpLongSharpOne0to100)?
            }
            HapticPattern::TransitionRampUpLongSharpTwo0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpLongSharpTwo0to100)?
            }
            HapticPattern::TransitionRampUpMediumSharpOne0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpMediumSharpOne0to100)?
            }
            HapticPattern::TransitionRampUpMediumSharpTwo0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpMediumSharpTwo0to100)?
            }
            HapticPattern::TransitionRampUpShortSharpOne0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpShortSharpOne0to100)?
            }
            HapticPattern::TransitionRampUpShortSharpTwo0to100 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpShortSharpTwo0to100)?
            }
            HapticPattern::TransitionRampDownLongSmoothOne50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownLongSmoothOne50to0)?
            }
            HapticPattern::TransitionRampDownLongSmoothTwo50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownLongSmoothTwo50to0)?
            }
            HapticPattern::TransitionRampDownMediumSmoothOne50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownMediumSmoothOne50to0)?
            }
            HapticPattern::TransitionRampDownMediumSmoothTwo50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownMediumSmoothTwo50to0)?
            }
            HapticPattern::TransitionRampDownShortSmoothOne50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownShortSmoothOne50to0)?
            }
            HapticPattern::TransitionRampDownShortSmoothTwo50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownShortSmoothTwo50to0)?
            }
            HapticPattern::TransitionRampDownLongSharpOne50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownLongSharpOne50to0)?
            }
            HapticPattern::TransitionRampDownLongSharpTwo50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownLongSharpTwo50to0)?
            }
            HapticPattern::TransitionRampDownMediumSharpOne50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownMediumSharpOne50to0)?
            }
            HapticPattern::TransitionRampDownMediumSharpTwo50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownMediumSharpTwo50to0)?
            }
            HapticPattern::TransitionRampDownShortSharpOne50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownShortSharpOne50to0)?
            }
            HapticPattern::TransitionRampDownShortSharpTwo50to0 => {
                self.hfb.set_single_effect(Effect::TransitionRampDownShortSharpTwo50to0)?
            }
            HapticPattern::TransitionRampUpLongSmoothOne0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpLongSmoothOne0to50)?
            }
            HapticPattern::TransitionRampUpLongSmoothTwo0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpLongSmoothTwo0to50)?
            }
            HapticPattern::TransitionRampUpMediumSmoothOne0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpMediumSmoothOne0to50)?
            }
            HapticPattern::TransitionRampUpMediumSmoothTwo0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpMediumSmoothTwo0to50)?
            }
            HapticPattern::TransitionRampUpShortSmoothOne0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpShortSmoothOne0to50)?
            }
            HapticPattern::TransitionRampUpShortSmoothTwo0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpShortSmoothTwo0to50)?
            }
            HapticPattern::TransitionRampUpLongSharpOne0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpLongSharpOne0to50)?
            }
            HapticPattern::TransitionRampUpLongSharpTwo0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpLongSharpTwo0to50)?
            }
            HapticPattern::TransitionRampUpMediumSharpOne0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpMediumSharpOne0to50)?
            }
            HapticPattern::TransitionRampUpMediumSharpTwo0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpMediumSharpTwo0to50)?
            }
            HapticPattern::TransitionRampUpShortSharpOne0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpShortSharpOne0to50)?
            }
            HapticPattern::TransitionRampUpShortSharpTwo0to50 => {
                self.hfb.set_single_effect(Effect::TransitionRampUpShortSharpTwo0to50)?
            }
            HapticPattern::LongBuzzForProgrammaticStopping100 => {
                self.hfb.set_single_effect(Effect::LongBuzzForProgrammaticStopping100)?
            }
            HapticPattern::SmoothHumOne50 => self.hfb.set_single_effect(Effect::SmoothHumOne50)?,
            HapticPattern::SmoothHumTwo40 => self.hfb.set_single_effect(Effect::SmoothHumTwo40)?,
            HapticPattern::SmoothHumThree30 => self.hfb.set_single_effect(Effect::SmoothHumThree30)?,
            HapticPattern::SmoothHumFour20 => self.hfb.set_single_effect(Effect::SmoothHumFour20)?,
            HapticPattern::SmoothHumFive10 => self.hfb.set_single_effect(Effect::SmoothHumFive10)?,
        }

        self.hfb.set_go(true)?;

        Ok(())
    }

    pub fn vibrate(&mut self, haptic_pattern: HapticPattern) {
        if let Err(e) = self.vibrate_fallible(haptic_pattern) {
            log::error!("Error running pattern {haptic_pattern:?}: {e:?}");
        }
    }
}
