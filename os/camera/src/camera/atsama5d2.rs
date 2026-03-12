// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::thread;
use std::time::Duration;

use atsama5d27::isc::{ClkSel, DmaBuffer, DmaControlConfig, ISCStatus, Isc};
use gpio::{GpioPin, PinSettings};
use gui_server_api::{
    consts::{CAMERA_BYTES_PER_PX, CAMERA_FB_SIZE_BYTES, CAMERA_HEIGHT, CAMERA_MARGIN, CAMERA_WIDTH},
    DoubleBufferVMA, Vsync,
};
use i2c::Peripheral;
use log::{debug, trace, warn};
use ovm7690_rs::Ovm7690;
use server::{BlockingScalar, BlockingScalarHandler, CheckedConn, ScalarHandler, ServerContext};
use utralib::utra::isc::HW_ISC_BASE;
use xous::{arch::irq::IrqNumber, MemoryFlags, MemoryRange, PID};

use crate::{error::CameraError, messages::*, GuiApi};

const ISC_MASTER_CLK_DIV: u8 = 13; // This gives around 30 fps
const ISC_MASTER_CLK_SEL: ClkSel = ClkSel::Hclock;
const ISC_ISP_CLK_DIV: u8 = 2;
const ISC_ISP_CLK_SEL: ClkSel = ClkSel::Hclock;

i2c::use_api!();
gpio::use_api!();
power_manager::use_api!();

#[derive(server::Server)]
#[name = "os/camera"]
pub struct CameraServer {
    isc_dma: DmaBuffer,
    bufs: DoubleBufferVMA,
    gui_api: GuiApi,
    is_enabled: bool,
    is_visible: bool,
    hw_state: HwState,
    is_frame_ready: bool,
    is_capture_in_progress: bool,
    frame_num: usize,
    isc: Isc,
    isc_address: u32,
    gpio: GpioApi,
    power_manager: PowerManagerApi,
    ovm: Ovm7690<I2cPeripheral>,
}

#[derive(Debug, Default, Clone, server::Permissions)]
#[server_name = "os/camera"]
#[all_permissions]
struct InternalPermissions;

struct InterruptContext {
    conn: CheckedConn<InternalPermissions>,
    isc: Isc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HwState {
    Enabled,
    DisableAfterNextFrame,
    Disabled,
}

impl CameraServer {
    pub fn new() -> Result<Self, CameraError> {
        debug!("Initializing camera");
        trace!("Allocating framebuffers");

        trace!("Registering the camera app");
        let (gui_api, bufs) =
            GuiApi::register(gui_server_api::AppKind::Camera, "Camera", CAMERA_FB_SIZE_BYTES)?;
        let bufs = bufs.into_bufs()?.into_vma()?;

        let gpio = gpio::GpioApi::default();
        trace!("Claiming GPIO pins");
        gpio.claim_pin(GpioPin::CamPwdn, PinSettings::OutputLow, false)?;
        gpio.claim_pin(GpioPin::CamLdoPwdnB, PinSettings::OutputHigh, false)?;

        trace!("Enabling image sensor interface clock");
        let power_manager = PowerManagerApi::default();
        power_manager.enable_peripheral(atsama5d27::pmc::PeripheralId::Isi)?;

        trace!("Mapping ISC");
        let mem = xous::map_memory(
            xous::MemoryAddress::new(HW_ISC_BASE),
            None,
            0x1000,
            xous::MemoryFlags::W | xous::MemoryFlags::DEV,
        )?;
        let isc_address = mem.as_ptr() as u32;
        trace!("Mapped ISC to 0x{:08x}", isc_address);
        let mut isc = Isc::with_alt_base_addr(isc_address);
        isc.setup_clocks(ISC_MASTER_CLK_DIV, ISC_MASTER_CLK_SEL, ISC_ISP_CLK_DIV, ISC_ISP_CLK_SEL);
        isc.enable_clock();
        isc.configure(false);
        isc.set_cropping_area(0, 0, CAMERA_WIDTH as u32, CAMERA_HEIGHT as u32);
        isc.enable_interrupt(ISCStatus::DDONE);
        trace!("ISC initialized");

        let dma_desc_mem = xous::map_memory(
            None,
            None,
            0x1000,
            MemoryFlags::W
                | MemoryFlags::NO_CACHE
                | MemoryFlags::DEV
                | MemoryFlags::POPULATE
                | MemoryFlags::PLAINTEXT,
        )?;

        let dma_desc1_addr = dma_desc_mem.as_ptr() as usize;
        let dma_desc1_phys_addr = xous::virt_to_phys(dma_desc1_addr)?;

        let isc_dma = DmaBuffer::new(dma_desc1_addr as u32, dma_desc1_phys_addr as u32, 0);

        trace!("Claiming I2C camera peripheral");
        let i2c = i2c::I2cApi::default().claim_peripheral(Peripheral::Camera)?;

        let mut ovm = Ovm7690::new(i2c);
        trace!("Verifying camera connection");
        ovm.verify_chip_id()?;

        let mut result = Self {
            isc_dma,
            bufs,
            gui_api,
            is_enabled: false,
            is_visible: false,
            hw_state: HwState::Disabled,
            is_frame_ready: false,
            is_capture_in_progress: false,
            frame_num: 0,
            isc,
            isc_address,
            gpio,
            power_manager,
            ovm,
        };

        trace!("Init done, disabling power and clocks");
        result.disable_hw()?;

        Ok(result)
    }

    pub fn start(&mut self, _context: &mut ServerContext<Self>) -> Result<(), CameraError> {
        let int_ctx = Box::into_raw(Box::new(InterruptContext {
            conn: CheckedConn::default(),
            isc: Isc::with_alt_base_addr(self.isc_address),
        }));
        xous::claim_interrupt(IrqNumber::Isi, handle_isc_irq, int_ctx as *mut usize)?;
        Ok(())
    }

    fn capture_frame(&mut self) {
        if self.is_capture_in_progress {
            warn!("Capture already in progress");
            return;
        }

        self.isc_dma.fb_phys_addr =
            (self.bufs.work_buf.phys_addr + CAMERA_MARGIN * CAMERA_WIDTH * CAMERA_BYTES_PER_PX) as u32;
        let dma_desc_mem_range =
            unsafe { MemoryRange::new(self.isc_dma.dma_desc_addr as usize, 4096).expect("dma range") };
        self.isc.configure_dma(
            &[self.isc_dma],
            &DmaControlConfig { descriptor_enable: true, ..Default::default() },
            || {
                xous::syscall::flush_cache(dma_desc_mem_range, xous::CacheOperation::Clean)
                    .expect("invalidate cache dma");
            },
        );

        trace!("capturing to {:08x}", self.bufs.work_buf.virt_addr);

        self.is_capture_in_progress = true;
        self.isc.start_capture();
    }

    fn reset_buffers(&mut self) {
        self.is_frame_ready = false;
        unsafe {
            self.bufs.to_double_buf_virt().fill_with(0, CAMERA_FB_SIZE_BYTES).ok();
        }
        if self.gui_api.swap_buffers(Vsync::Wait).unwrap().is_some() {
            self.bufs.swap();
        }
    }

    fn enable_hw(&mut self) -> Result<(), CameraError> {
        self.power_manager.enable_peripheral(atsama5d27::pmc::PeripheralId::Isi)?;
        self.isc.enable_clock();
        self.gpio.set_pin(GpioPin::CamPwdn, false)?;
        self.gpio.set_pin(GpioPin::CamLdoPwdnB, true)?;
        thread::sleep(Duration::from_millis(1));
        self.ovm.sw_reset()?;
        thread::sleep(Duration::from_millis(1));
        self.ovm.init()?;
        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    fn disable_hw(&mut self) -> Result<(), CameraError> {
        self.isc.disable_clock();
        PowerManagerApi::default().disable_peripheral(atsama5d27::pmc::PeripheralId::Isi)?;
        self.gpio.set_pin(GpioPin::CamLdoPwdnB, false)?;
        self.gpio.set_pin(GpioPin::CamPwdn, true)?;
        Ok(())
    }

    fn update_hw_state(&mut self) -> Result<(), CameraError> {
        log::trace!(
            "Update HW state called: enabled={:?} visible={:?} hw_state={:?}",
            self.is_enabled,
            self.is_visible,
            self.hw_state
        );
        if self.is_enabled && self.is_visible {
            if self.hw_state == HwState::Disabled {
                log::debug!("Turning ON");
                self.enable_hw()?;
            }
            self.hw_state = HwState::Enabled;
            if !self.is_capture_in_progress {
                self.capture_frame();
            }
        } else if self.hw_state != HwState::Disabled {
            log::debug!("Turning OFF after next frame");
            self.hw_state = HwState::DisableAfterNextFrame;
        }
        Ok(())
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
        if let Err(e) = self.update_hw_state() {
            log::error!("Error updating HW state: {e:?}");
        }
    }
}

impl ScalarHandler<FrameCaptured> for CameraServer {
    fn handle(&mut self, _msg: FrameCaptured, sender: xous::PID, _context: &mut ServerContext<Self>) {
        if sender != xous::current_pid().unwrap() {
            return;
        }

        self.is_capture_in_progress = false;

        if self.hw_state == HwState::DisableAfterNextFrame {
            log::debug!("Turning OFF (discarding last frame)");
            self.reset_buffers();
            if let Err(e) = self.disable_hw() {
                log::error!("Error disabling camera: {e:?}");
            }
            self.hw_state = HwState::Disabled;
            return;
        }

        self.frame_num += 1;
        trace!("Camera frame #{}, working on {:08x}", self.frame_num, self.bufs.work_buf.virt_addr);

        xous::syscall::flush_cache(
            unsafe { xous::MemoryRange::new(self.bufs.work_buf.virt_addr, CAMERA_FB_SIZE_BYTES).unwrap() },
            xous::CacheOperation::Invalidate,
        )
        .expect("invalidate cache");

        self.is_frame_ready = true;
        if self.gui_api.swap_buffers(Vsync::Wait).unwrap().is_some() {
            self.bufs.swap();
        }

        self.capture_frame();
    }
}

impl server::ScalarEventHandler<settings::global::CameraEnabled> for CameraServer {
    fn handle(
        &mut self,
        msg: settings::global::CameraEnabled,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) {
        self.set_enabled(msg.0);
    }
}

impl BlockingScalarHandler<IsReady> for CameraServer {
    fn handle(
        &mut self,
        _msg: IsReady,
        _sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> <IsReady as BlockingScalar>::Response {
        self.is_frame_ready
    }
}

impl BlockingScalarHandler<GetFrameMemoryMirror> for CameraServer {
    fn handle(
        &mut self,
        _msg: GetFrameMemoryMirror,
        sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> Option<MemoryRange> {
        let buf_range = unsafe {
            MemoryRange::new(
                self.bufs.disp_buf.virt_addr + CAMERA_MARGIN * CAMERA_WIDTH * CAMERA_BYTES_PER_PX,
                CAMERA_WIDTH * CAMERA_HEIGHT * CAMERA_BYTES_PER_PX,
            )
            .ok()?
        };
        log::debug!("Creating a mirror for PID{} in {:?} range", sender, buf_range);

        xous::mirror_memory_to_pid(buf_range, sender).ok()
    }
}

impl ScalarHandler<SetEnabled> for CameraServer {
    fn handle(&mut self, msg: SetEnabled, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        self.set_enabled(msg.0);
    }
}
impl ScalarHandler<NotifyVisible> for CameraServer {
    fn handle(&mut self, msg: NotifyVisible, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        self.is_visible = msg.0;
        if let Err(e) = self.update_hw_state() {
            log::error!("Error updating HW state: {e:?}");
        }
    }
}
impl BlockingScalarHandler<IsEnabled> for CameraServer {
    fn handle(
        &mut self,
        _msg: IsEnabled,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <IsEnabled as BlockingScalar>::Response {
        self.is_enabled
    }
}
impl BlockingScalarHandler<IsInUse> for CameraServer {
    fn handle(
        &mut self,
        _msg: IsInUse,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <IsInUse as BlockingScalar>::Response {
        self.hw_state != HwState::Disabled
    }
}

/// Handles IRQs from ISC.
/// *NOTE*: Avoid panics and prints inside an IRQ handler as they may result in forbidden syscalls.
fn handle_isc_irq(_irq_no: usize, arg: *mut usize) {
    let context = unsafe { &mut *(arg as *mut InterruptContext) };
    let status = context.isc.interrupt_status();

    // DMA transfer of the camera frame is complete
    if status.contains(ISCStatus::DDONE) {
        context.conn.send_scalar_nowait(FrameCaptured).ok();
    }
}
