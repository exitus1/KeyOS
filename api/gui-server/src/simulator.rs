// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{consts, msg, touch, GuiServerError};

#[derive(Default)]
pub struct SimulatorApi<P: server::CheckedPermissions>(server::CheckedConn<P>);

impl<P: server::CheckedPermissions> SimulatorApi<P> {
    pub fn device_frame(&self, entire_device: bool) -> Result<Vec<u8>, GuiServerError>
    where
        P: server::MessageAllowed<msg::GetDeviceFrame>,
        P: server::MessageAllowed<msg::GetScreenFrame>,
    {
        let mem;
        if entire_device {
            mem = xous::map_memory(
                None,
                None,
                consts::DEVICE_WIDTH as usize * consts::DEVICE_HEIGHT as usize * 4,
                xous::MemoryFlags::W,
            )?;
            self.0.lend_mut(msg::GetDeviceFrame(mem));
        } else {
            mem = xous::map_memory(
                None,
                None,
                consts::SCREEN_WIDTH * consts::SCREEN_HEIGHT * 4,
                xous::MemoryFlags::W,
            )?;
            self.0.lend_mut(msg::GetScreenFrame(mem));
        };

        let vec = mem.as_slice().to_vec();
        xous::unmap_memory(mem)?; // memory was copied into the Vec

        Ok(vec)
    }

    pub fn set_scale_factor(&self, scale_factor: f32) -> Result<(), GuiServerError>
    where
        P: server::MessageAllowed<msg::SetScaleFactor>,
    {
        self.0.try_send_scalar(msg::SetScaleFactor((scale_factor * 256.0) as usize))?;
        Ok(())
    }

    pub fn simulate_touch(&self, touch: touch::Touch) -> Result<(), GuiServerError>
    where
        P: server::MessageAllowed<msg::SimulateTouch>,
    {
        self.0.try_send_scalar(msg::SimulateTouch(touch))?;

        Ok(())
    }

    pub fn simulate_power_button(&self, is_pressed: bool) -> Result<(), GuiServerError>
    where
        P: server::MessageAllowed<msg::SimulatePowerButton>,
    {
        self.0.try_send_scalar(msg::SimulatePowerButton(is_pressed))?;

        Ok(())
    }
}
