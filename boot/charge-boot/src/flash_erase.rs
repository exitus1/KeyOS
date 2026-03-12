// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use atsama5d27::sdmmc::{SDCmd, SDCmdInner, SDRespType, Sdmmc};

pub fn erase_flash_blocks() {
    let mut sdmmc = Sdmmc::sdmmc0();
    sdmmc.send_command(SDCmd::Sd(SDCmdInner::EraseGroupStart), SDRespType::R1, 0, None).ok();
    sdmmc.send_command(SDCmd::Sd(SDCmdInner::EraseGroupEnd), SDRespType::R1, 16, None).ok();

    sdmmc.send_command(SDCmd::Sd(SDCmdInner::Erase), SDRespType::R1B, 0, None).ok();
}
