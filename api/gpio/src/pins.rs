// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use num_derive::{FromPrimitive, ToPrimitive};

#[derive(
    Debug,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
    Eq,
    Hash,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum GpioPin {
    /// `CTP_RST_B` signal - Capacitive Touch Panel Controller Reset
    CtpRstB = 0,
    /// `CTP_INT_B` signal - Capacitive Touch Panel Controller Interrupt
    CtpIrqB,

    /// `LCD_RESET` signal - LCD Reset. Active low.
    LcdRstB,

    /// `MCU_WKUP` signal - Power Button
    PowerButton,

    /// `CAM_PWDN` signal - Camera power-down. Pull high to set camera to standby mode.
    CamPwdn,
    /// `CAM_LDO_PWDN` signal - Camera LDO PSU enable. Pull to ground to disable camera.
    CamLdoPwdnB,

    /// `HFB_EN` signal - Haptic feedback controller enable pin. Active high.
    HfbEn,
    /// `HFB_IN` signal - Haptic feedback controller input pin. (I2C selectable as PWM, analog or trigger).
    HfbIn,

    /// `BT_IRQ_OUT` signal - BLE->MPU IRQ line. Active low.
    BtIrqB,
    /// `BT_RESET` signal - Reset of the BLE controller. Active low.
    BtRst,
    /// `BT_WP_B` signal - Write protect of the EEPROM connected to the BLE controller. Active low.
    BtEepWpB,

    /// `OTG_ID` signal - Active low (i.e. Low means a peripheral is connected)
    UsbOtgId,
    /// `VBUS_DIV_RC` signal - VBUS has power. Active high.
    UsbVbusIrq,
    /// `USB_PC_INT` signal - USB Port Controller interrupt request. Active low.
    UsbCtrlIrqB,

    /// `LED_DRIVER_SDB` signal - LED driver shutdown. Active low.
    LedDrvPwdnB,
    /// `CHGPMP_ENA` signal - RGB driver charge pump enable. Active high.
    LedChgPmpEn,

    /// `ALS_INT_B` signal - Ambient Light Sensor Interrupt request pin. Configurable active level, driver
    /// sets it to High.
    AlsIrqB,

    /// `NFC_IRQ_IN` signal - MPU->NFC interrupt. Active low.
    NfcIntB,
    /// `NFC_IRQ_OUT` signal - NFC->MPU interrupt request. Active low.
    NfcIrqB,

    /// `BC_CD` signal - Battery Charger charge disable.
    BatChgEnB,
    /// `BC_OTG` signal - Battery Charger boost mode enable.
    BatChgOtg,
    /// `BC_STAT` signal - Battery Charger status. Active low (charge in process).
    BatChgStat,
    /// `FG_INT` signal - Fuel Gauge interrupt request. Active low.
    FuelIrqB,
    /// `WPT_EN1` signal - Wireless Power Transfer enable 1.

    /// `ACCL_INT_B` signal - Accelerometer interrupt. Active low.
    AcclIntB,

    /// `NOISE_BIAS_EN` signal - Avalanche Noise enable. Active high.
    NoiseEn,
}

#[derive(Debug, FromPrimitive, ToPrimitive, Copy, Clone)]
pub enum PinSettings {
    /// Pin is configured as a digital input (no filtering).
    Input = 0,
    /// Pin is configured as a digital output and set to the HIGH state.
    OutputHigh,
    /// Pin is configured as a digital output and set to the LOW state.
    OutputLow,
    /// Pin is configured as an open-drain output and set to the High-Z state.
    OutputOpenDrainHighZ,
    /// Pin is configured as an open-drain output and set to the LOW state.
    OutputOpenDrainLow,
    /// Pin is configured as a digital input and interrupt source on falling edge.
    InterruptFalling,
    /// Pin is configured as a digital input and interrupt source on rising edge.
    InterruptRising,
    /// Pin is configured as a digital input and interrupt source on both falling and
    /// rising edge.
    InterruptBoth,
}

impl PinSettings {
    /// Returns `true` if the pin is configured as an interrupt source with these
    /// settings.
    pub fn is_interrupt(&self) -> bool {
        matches!(
            self,
            PinSettings::InterruptFalling | PinSettings::InterruptRising | PinSettings::InterruptBoth
        )
    }

    /// Returns `true` if the pin is configured as a digital output.
    pub fn is_output(&self) -> bool {
        matches!(
            self,
            PinSettings::OutputHigh
                | PinSettings::OutputLow
                | PinSettings::OutputOpenDrainLow
                | PinSettings::OutputOpenDrainHighZ
        )
    }
}
