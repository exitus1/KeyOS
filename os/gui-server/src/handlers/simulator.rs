// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use gui_server_api::msg::{
    GetDeviceFrame, GetScreenFrame, SetScaleFactor, SimulatePowerButton, SimulateTouch,
};
use server::{LendMutHandler, ScalarHandler, ServerContext};
use xous::PID;

use crate::display::PlatformDisplay;
use crate::{get_frame, Gui};

impl ScalarHandler<SetScaleFactor> for Gui {
    fn handle(&mut self, msg: SetScaleFactor, _sender: PID, _context: &mut ServerContext<Self>) {
        PlatformDisplay::set_scale_factor(msg.0)
    }
}

impl LendMutHandler<GetDeviceFrame> for Gui {
    fn handle(
        &mut self,
        GetDeviceFrame(mut mem): GetDeviceFrame,
        _sender: PID,
        _context: &mut ServerContext<Self>,
    ) {
        get_frame(true, &mut mem)
    }
}

impl LendMutHandler<GetScreenFrame> for Gui {
    fn handle(
        &mut self,
        GetScreenFrame(mut mem): GetScreenFrame,
        _sender: PID,
        _context: &mut ServerContext<Self>,
    ) {
        get_frame(false, &mut mem)
    }
}

impl ScalarHandler<SimulateTouch> for Gui {
    fn handle(
        &mut self,
        SimulateTouch(pos): SimulateTouch,
        _sender: PID,
        _context: &mut ServerContext<Self>,
    ) {
        self.touch_dispatch(pos);
    }
}

impl ScalarHandler<SimulatePowerButton> for Gui {
    fn handle(
        &mut self,
        SimulatePowerButton(is_pressed): SimulatePowerButton,
        _sender: PID,
        _context: &mut ServerContext<Self>,
    ) {
        self.handle_power_button(is_pressed);
    }
}
