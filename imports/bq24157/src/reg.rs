// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use bitfield::bitfield;

#[derive(Debug, Copy, Clone)]
pub enum Registers {
    Status = 0x00,
    Control = 0x01,
    BatteryVoltage = 0x02,
    Revision = 0x03,
    ChargerCurrent = 0x04,
    SpecialChargerVoltage = 0x05,
    SafetyLimit = 0x06,
}

bitfield! {
    pub struct Status(u8);
    impl Debug;
    pub fault, _: 2, 0;   // Charge mode: 000: Normal, 001: VBUS OVP, 010: Sleep mode
                          //              011: Bad Adaptor or VBUS<VUVLO, 100: Output OVP, 101: Thermal shutdown,
                          //              110: Timer fault, 111: No battery
                          // Boost mode:  000: Normal, 001: VBUS OVP, 010: Over load,
                          //              011: Battery voltage is too low, 100: Battery OVP, 101: Thermal shutdown,
                          //              110: Timer fault, 111-NA
    pub is_boost, _: 3;
    pub stat, _: 5, 4;    // 00: Ready, 01: Charge in progress, 10: Charge done, 11: Fault
    pub en_stat, set_en_stat: 6;
    pub otg_stat_reset_tmr, set_otg_stat_reset_tmr: 7;
}

bitfield! {
    pub struct Control(u8);
    impl Debug;

    pub opa_mode, set_opa_mode: 0;  // 0: charger, 1: boost mode
    pub hz_mode, set_hz_mode: 1;    // 0: no Hi-Z mode, 1: Hi-Z mode
    pub dis_chg, set_dis_chg: 2;    // 0: charger is enabled, 1: charger is disabled
    pub te, set_te: 3;              // 0: disable charge current termination, 1: enable

    pub v_low, set_v_low: 5, 4;     // Weak battery voltage threshold
                                    // The range of the weak battery voltage threshold is 3.4 V to 3.7 V
                                    // with an offset of 3.4 V and steps of 100 mV (default 3.7 V, using bits B4-B5)

    pub i_lim, set_i_lim: 7, 6;     // 00: USB host with 100-mA current limit (default)
                                    // 01: USB host with 500-mA current limit
                                    // 10: USB host/charger with 800-mA current limit
                                    // 11: No input current limit
}

bitfield! {
    pub struct BatteryVoltage(u8);
    impl Debug;

    pub otg_en, set_otg_en: 0;        // 1: Enable OTG Pin in HOST mode
                                      // 0: Disable OTG pin in HOST mode (default 0), not applicable
                                      // to OTG pin control of current limit at POR in default mode

    pub otg_pl, set_otg_pl: 1;        // 1: OTG Boost Enable with High level
                                      // 0: OTG Boost Enable with Low level (default 1); not applicable
                                      // to OTG pin control of current limit at POR in default mode

    pub bat_vreg, set_bat_vreg: 7, 2; // Battery regulation voltage
                                      // Charge voltage range is 3.5 V to 4.44 V with the offset of 3.5 V
                                      // and steps of 20 mV (default 3.54 V), using bits B2-B7
}

bitfield! {
    pub struct Revision(u8);
    impl Debug;

    u8;
    pub rev, _: 2, 0;           // Revision number
                                // 011: Revision 1.0; 001: Revision 1.1;
                                // 100-111: Future Revisions

    pub pn, _: 4, 3;            // Part number
                                // 01: NA, 10: bq24157, 11: NA

    pub vendor_code, _: 7, 5;   // Vendor code
}

pub(crate) const PN_BQ24157: u8 = 0b10;

bitfield! {
    pub struct ChargerCurrent(u8);
    impl Debug;

    pub v_iterm, set_v_iterm: 2, 0;                     // Termination current sense voltage
                                                        // See datasheet table 11

    pub chg_curr_sense_v, set_chr_curr_sense_v: 6, 3;   // Charge current sense voltage
                                                        // See datasheet table 12
                                                        //
                                                        // Charge current sense voltage offset is 37.4 mV
                                                        // and default charge current is 550 mA,
                                                        // if 68-mΩ sensing resistor is used and LOW_CHG=0

    pub reset, set_reset: 7;                            // 1: Reset the charger
}

bitfield! {
    pub struct SpecialChargerVoltage(u8);
    impl Debug;

    pub vsreg, set_vsreg: 2, 0;  // Special charger voltage
                                 // Offset is 4.2 V and default special charger voltage is 4.52 V

    pub cd_stat, _: 3;           // CD pin level
    pub dpm_stat, _: 4;          // DPM mode status, 1: active, 0: inactive

    pub low_chg, set_low_chg: 5; // 0: Normal charge current sense voltage at 04H (default 1)
                                 // 1: Low charge current sense voltage of 22.1 mV
}

bitfield! {
    pub struct SafetyLimit(u8);
    impl Debug;

    pub vr_max, set_vr_max: 3, 0;               // Maximum battery regulation voltage
                                                // Offset is 4.2V (default at 4.2 V) and maximum is 4.44V

    pub v_curr_sense, set_v_curr_sense: 7, 4;   // Maximum charge current sense voltage
                                                // Offset is 37.4 mV (550 mA), default at 64.6 mV (950 mA)
                                                // and the maximum is 1.55 A (105.4 mV), if 55-mΩ sensing resistor is used
}

pub const SAFETY_V_CURR_SENSE_DEFAULT: u8 = 0b0100;

#[derive(Debug, Copy, Clone)]
pub enum ChargeFault {
    Normal = 0b000,
    VbusOvp = 0b001,
    SleepMode = 0b010,
    BadAdaptor = 0b011,
    OutputOvp = 0b100,
    ThermalShutdown = 0b101,
    TimerFault = 0b110,
    NoBattery = 0b111,
}

#[derive(Debug, Copy, Clone)]
pub enum BoostFault {
    Normal = 0b000,
    VbusOvp = 0b001,
    Overload = 0b010,
    BatteryLow = 0b011,
    BatteryOvp = 0b100,
    ThermalShutdown = 0b101,
    TimerFault = 0b110,
    Reserved = 0b111,
}

#[derive(Debug, Copy, Clone)]
pub enum State {
    Ready = 0b00,
    ChargeInProgress = 0b01,
    ChargeDone = 0b10,
    Fault = 0b11,
}

impl Status {
    pub fn charge_fault(&self) -> Option<ChargeFault> {
        match self.fault() {
            0b000 => Some(ChargeFault::Normal),
            0b001 => Some(ChargeFault::VbusOvp),
            0b010 => Some(ChargeFault::SleepMode),
            0b011 => Some(ChargeFault::BadAdaptor),
            0b100 => Some(ChargeFault::OutputOvp),
            0b101 => Some(ChargeFault::ThermalShutdown),
            0b110 => Some(ChargeFault::TimerFault),
            0b111 => Some(ChargeFault::NoBattery),
            _ => None,
        }
    }

    pub fn boost_fault(&self) -> Option<BoostFault> {
        match self.fault() {
            0b000 => Some(BoostFault::Normal),
            0b001 => Some(BoostFault::VbusOvp),
            0b010 => Some(BoostFault::Overload),
            0b011 => Some(BoostFault::BatteryLow),
            0b100 => Some(BoostFault::BatteryOvp),
            0b101 => Some(BoostFault::ThermalShutdown),
            0b110 => Some(BoostFault::TimerFault),
            0b111 => Some(BoostFault::Reserved),
            _ => None,
        }
    }

    pub fn state(&self) -> Option<State> {
        match self.stat() {
            0b00 => Some(State::Ready),
            0b01 => Some(State::ChargeInProgress),
            0b10 => Some(State::ChargeDone),
            0b11 => Some(State::Fault),
            _ => None,
        }
    }
}
