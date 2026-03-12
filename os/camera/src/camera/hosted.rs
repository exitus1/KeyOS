// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use image::{ImageBuffer, RgbaImage};
use nokhwa::{nokhwa_initialize, utils::RequestedFormat, Camera};
use server::{ArchiveHandler, BlockingScalar, BlockingScalarHandler, ScalarHandler, ServerContext};
use xous::{MemoryRange, PID};
use {
    gui_server_api::{
        consts::{CAMERA_BYTES_PER_PX, CAMERA_FB_SIZE_BYTES, CAMERA_HEIGHT, CAMERA_MARGIN, CAMERA_WIDTH},
        DoubleBuffer, DoubleBufferRegistration, Vsync,
    },
    log::debug,
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use crate::{error::CameraError, messages::*, GuiApi};

static IS_FRAME_READY: AtomicBool = AtomicBool::new(false);
static IS_ENABLED: AtomicBool = AtomicBool::new(false);

const PHYSICAL_CAMERA_WIDTH: usize = 640;

#[derive(server::Server)]
#[name = "os/camera"]
pub struct CameraServer {
    gui_api: GuiApi,
    bufs: DoubleBuffer,
    frame: Arc<Mutex<RgbaImage>>,
    is_visible: bool,
    frame_buffer: DoubleBufferRegistration,
}

#[derive(Debug, Default, Clone, server::Permissions)]
#[server_name = "os/camera"]
#[all_permissions]
struct InternalPermissions;

impl CameraServer {
    pub fn new() -> Result<Self, CameraError> {
        debug!("Initializing camera");
        let (gui_api, framebuffer) =
            GuiApi::register(gui_server_api::AppKind::Camera, "Camera", CAMERA_FB_SIZE_BYTES)?;
        let bufs = framebuffer.clone().into_bufs()?;
        let work_fb =
            unsafe { core::slice::from_raw_parts_mut(bufs.work_buf as *mut u32, CAMERA_FB_SIZE_BYTES / 4) };
        work_fb.fill(0xff000000);
        let back_fb =
            unsafe { core::slice::from_raw_parts_mut(bufs.disp_buf as *mut u32, CAMERA_FB_SIZE_BYTES / 4) };
        back_fb.fill(0xff000000);

        Ok(Self { gui_api, bufs, frame: Default::default(), is_visible: false, frame_buffer: framebuffer })
    }

    pub fn start(&mut self, _context: &mut ServerContext<CameraServer>) -> Result<(), CameraError> {
        debug!("Running camera app");

        nokhwa_initialize(|allowed| log::info!("Nokhwa initialized: allowed={allowed:?}"));
        let frame = self.frame.clone();
        std::thread::spawn(move || match Self::camera_thread(frame) {
            Ok(_) => log::warn!("Camera thread exited with Ok(())"),
            Err(e) => log::error!("Camera thread exited with an error: {e:?}"),
        });

        Ok(())
    }

    fn camera_thread(frame: Arc<Mutex<RgbaImage>>) -> Result<(), CameraError> {
        let connection = server::CheckedConn::<InternalPermissions>::default();
        let mut camera = Camera::new(
            nokhwa::utils::CameraIndex::Index(0),
            RequestedFormat::new::<nokhwa::pixel_format::RgbAFormat>(
                nokhwa::utils::RequestedFormatType::HighestResolution(nokhwa::utils::Resolution {
                    width_x: PHYSICAL_CAMERA_WIDTH as _,
                    height_y: CAMERA_HEIGHT as _,
                }),
            ),
        )?;
        let mut started = false;

        loop {
            if IS_ENABLED.load(Ordering::Relaxed) {
                if !started {
                    camera.open_stream()?;
                    started = true;
                }
                let new_frame = camera.frame()?.decode_image::<nokhwa::pixel_format::RgbAFormat>()?;
                *frame.lock().unwrap() = new_frame;
                connection.try_send_scalar(FrameCaptured)?;
            } else {
                if started {
                    camera.stop_stream()?;
                    started = false;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }

    pub fn handle_frame(&mut self) -> Result<(), CameraError> {
        // Copy the result into the GUI buffer
        let work_fb = (self.bufs.work_buf + CAMERA_MARGIN * CAMERA_WIDTH * CAMERA_BYTES_PER_PX) as *mut u8;
        let work_fb = unsafe { core::slice::from_raw_parts_mut(work_fb, CAMERA_WIDTH * CAMERA_HEIGHT * 4) };
        let mut work_image = ImageBuffer::from_raw(CAMERA_WIDTH as _, CAMERA_HEIGHT as _, work_fb).unwrap();

        image::imageops::replace(&mut work_image, &*self.frame.lock().unwrap(), -100, 0);

        self.bufs.swap();
        IS_FRAME_READY.store(true, Ordering::Relaxed);
        self.gui_api.swap_buffers(Vsync::Wait)?;
        Ok(())
    }
}

impl server::ScalarEventHandler<settings::global::CameraEnabled> for CameraServer {
    fn handle(
        &mut self,
        msg: settings::global::CameraEnabled,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) {
        IS_ENABLED.store(msg.0, Ordering::Relaxed);
    }
}

impl ScalarHandler<FrameCaptured> for CameraServer {
    fn handle(&mut self, _msg: FrameCaptured, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        self.handle_frame().expect("Error during handling frame");
    }
}

impl BlockingScalarHandler<IsReady> for CameraServer {
    fn handle(
        &mut self,
        _msg: IsReady,
        _sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> <IsReady as BlockingScalar>::Response {
        IS_FRAME_READY.load(Ordering::Relaxed)
    }
}

impl BlockingScalarHandler<GetFrameMemoryMirror> for CameraServer {
    fn handle(
        &mut self,
        _msg: GetFrameMemoryMirror,
        _sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> Option<MemoryRange> {
        None
    }
}

impl ScalarHandler<SetEnabled> for CameraServer {
    fn handle(&mut self, msg: SetEnabled, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        IS_ENABLED.store(msg.0, Ordering::Relaxed);
    }
}
impl ScalarHandler<NotifyVisible> for CameraServer {
    fn handle(&mut self, msg: NotifyVisible, _sender: xous::PID, _context: &mut ServerContext<Self>) {
        self.is_visible = msg.0;
    }
}
impl BlockingScalarHandler<IsEnabled> for CameraServer {
    fn handle(
        &mut self,
        _msg: IsEnabled,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <IsEnabled as BlockingScalar>::Response {
        IS_ENABLED.load(Ordering::Relaxed)
    }
}
impl BlockingScalarHandler<IsInUse> for CameraServer {
    fn handle(
        &mut self,
        _msg: IsInUse,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <IsInUse as BlockingScalar>::Response {
        IS_ENABLED.load(Ordering::Relaxed) && self.is_visible
    }
}

impl ArchiveHandler<GetFrameBufId> for CameraServer {
    fn handle(
        &mut self,
        _msg: GetFrameBufId,
        _sender: xous::PID,
        _context: &mut ServerContext<Self>,
    ) -> <GetFrameBufId as server::Archive>::Response {
        self.frame_buffer.disp_buf_id.clone()
    }
}
