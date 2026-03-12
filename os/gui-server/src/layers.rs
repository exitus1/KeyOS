// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use gui_server_api::{
    consts::{CONTROL_CENTER_HEIGHT_EXPANDED_PX, SCREEN_HEIGHT, SCREEN_WIDTH},
    DoubleBufferVMA, VMALocation,
};

use crate::{control_center::ControlCenterWindowState, display::MAX_LAYERS, AppWindow, Gui};

#[derive(Debug)]
pub struct Layer {
    src: SourceType,
    src_width: usize,
    #[allow(dead_code)]
    src_height: usize,
    crop_x: usize,
    crop_y: usize,
    crop_width: usize,
    crop_height: usize,
    dst_x: usize,
    dst_y: usize,
    dst_width: usize,
    dst_height: usize,
    pixel_format: LayerPixelFormat,
    alpha: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum SourceType {
    Dma(usize),
    Color { r: u8, g: u8, b: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerPixelFormat {
    Argb8888,
    #[cfg(keyos)]
    #[allow(dead_code)]
    Rgb565,
}

#[derive(Debug, Default)]
pub struct LayerStack {
    pub layers: [Option<Layer>; MAX_LAYERS],
}

impl Layer {
    pub fn new(src: VMALocation, src_width: usize, src_height: usize) -> Self {
        Self {
            src: SourceType::Dma(src.phys_addr as _),
            src_width,
            src_height,
            crop_x: 0,
            crop_y: 0,
            crop_width: src_width,
            crop_height: src_height,
            dst_x: 0,
            dst_y: 0,
            dst_width: src_width,
            dst_height: src_height,
            pixel_format: LayerPixelFormat::Argb8888,
            alpha: 255,
        }
    }

    pub fn new_double_bufs(src: &DoubleBufferVMA, src_width: usize, src_height: usize) -> Self {
        Self::new(src.disp_buf, src_width, src_height)
    }

    pub fn new_window(src: &AppWindow, src_width: usize, src_height: usize) -> Self {
        Self::new(src.blur_state.blurred_buf().unwrap_or(src.bufs.disp_buf), src_width, src_height)
    }

    pub fn new_single_color(r: u8, g: u8, b: u8, width: usize, height: usize) -> Self {
        Self {
            src: SourceType::Color { r, g, b },
            src_width: width,
            src_height: height,
            crop_x: 0,
            crop_y: 0,
            crop_width: width,
            crop_height: height,
            dst_x: 0,
            dst_y: 0,
            dst_width: width,
            dst_height: height,
            pixel_format: LayerPixelFormat::Argb8888,
            alpha: 255,
        }
    }

    pub fn with_position(self, x: usize, y: usize) -> Self { Self { dst_x: x, dst_y: y, ..self } }

    pub fn with_crop(self, x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            crop_x: x,
            crop_y: y,
            crop_width: width,
            crop_height: height,
            dst_width: width,
            dst_height: height,
            ..self
        }
    }

    pub fn with_dst_size(self, width: usize, height: usize) -> Self {
        Self { dst_width: width, dst_height: height, ..self }
    }

    pub fn with_alpha(self, alpha: u8) -> Self { Self { alpha, ..self } }

    #[allow(dead_code)]
    pub fn with_pixel_format(self, pixel_format: LayerPixelFormat) -> Self { Self { pixel_format, ..self } }

    pub fn is_scaled(&self) -> bool {
        self.crop_width != self.dst_width || self.crop_height != self.dst_height
    }

    pub fn src(&self) -> SourceType { self.src }

    pub fn src_dimensions(&self) -> (usize, usize) { (self.src_width, self.src_height) }

    pub fn crop_pos(&self) -> (usize, usize) { (self.crop_x, self.crop_y) }

    pub fn crop_dimensions(&self) -> (usize, usize) { (self.crop_width, self.crop_height) }

    pub fn pixel_format(&self) -> LayerPixelFormat { self.pixel_format }

    pub fn dst_pos(&self) -> (usize, usize) { (self.dst_x, self.dst_y) }

    pub fn dst_dimensions(&self) -> (usize, usize) { (self.dst_width, self.dst_height) }

    pub fn alpha(&self) -> u8 { self.alpha }
}

impl LayerPixelFormat {
    #[cfg(keyos)]
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            LayerPixelFormat::Argb8888 => 4,
            LayerPixelFormat::Rgb565 => 2,
        }
    }
}

impl LayerStack {
    pub fn push(&mut self, layer: Layer) {
        for (i, o) in &mut self.layers.iter_mut().enumerate() {
            match o {
                Some(o) => {
                    if o.is_scaled() && layer.is_scaled() {
                        log::error!("Cannot have two scaling layers, skipping {layer:?}");
                        return;
                    }
                }
                None => {
                    if i != 1 && i != 2 && layer.is_scaled() {
                        log::error!("Layer {i} can't have scaling. Skipping {layer:?}");
                    } else {
                        *o = Some(layer);
                        return;
                    }
                }
            }
        }
        log::error!("Too many layers; not adding {layer:?}");
    }

    pub fn layer_count(&self) -> usize { self.layers.iter().flatten().count() }
}

impl Gui {
    #[cfg(keyos)]
    pub(crate) fn boot_splash_layer() -> Layer {
        let Ok(boot_vma) = VMALocation::new_vma(xous::keyos::BOOT_SPLASH_FB as _) else {
            log::error!("Could not create buffer from Boot splash screen");
            return Layer::new_single_color(0, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
        };
        Layer::new_double_bufs(&DoubleBufferVMA::from_single(boot_vma), SCREEN_WIDTH, SCREEN_HEIGHT)
    }

    #[cfg(not(keyos))]
    pub(crate) fn boot_splash_layer() -> Layer {
        Layer::new_single_color(50, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)
    }

    pub fn update_layers(&mut self) {
        let mut layers = LayerStack::default();
        let control_center_collapsed = self.is_control_center_collapsed();
        match &self.state {
            crate::GuiState::BootSplash => {
                layers.push(Self::boot_splash_layer());
            }
            crate::GuiState::BootFade { to, progress } => {
                let Some(window) = self.windows.get(to) else {
                    log::error!("PID {to} does not have a window");
                    return;
                };
                layers.push(Self::boot_splash_layer());
                layers.push(
                    Layer::new_window(window, SCREEN_WIDTH, SCREEN_HEIGHT)
                        .with_alpha((*progress * 255 / 100) as u8),
                );
            }
            crate::GuiState::SingleWindow { pid, next_frame_animation, .. } => {
                let Some(window) = self.windows.get(pid) else {
                    log::error!("PID {pid} does not have a window");
                    return;
                };
                match next_frame_animation {
                    crate::NextFrameAnimationState::NotAnimating
                    | crate::NextFrameAnimationState::Waiting { .. } => {
                        self.add_camera_layer(window, &mut layers, 0);
                        layers.push(Layer::new_window(window, SCREEN_WIDTH, SCREEN_HEIGHT));
                    }
                    crate::NextFrameAnimationState::Animating { progress, kind } => {
                        Self::next_frame_animation_layers(
                            &mut layers,
                            Layer::new(
                                VMALocation::new_vma(self.animation_fb.as_ptr() as _).unwrap(),
                                SCREEN_WIDTH,
                                SCREEN_HEIGHT,
                            ),
                            Layer::new_window(window, SCREEN_WIDTH, SCREEN_HEIGHT),
                            *progress,
                            *kind,
                        );
                    }
                }
                self.add_keyboard_layer(window, &mut layers);
            }
            crate::GuiState::Switching { from, to, progress, animation, .. } => {
                let Some(from_window) = self.windows.get(from) else {
                    log::error!("From PID {from} does not have a window");
                    return;
                };
                let Some(to_window) = self.windows.get(to) else {
                    log::error!("To PID {to} does not have a window");
                    return;
                };
                animation.add_layers(
                    &mut layers,
                    Layer::new_window(from_window, SCREEN_WIDTH, SCREEN_HEIGHT),
                    Layer::new_window(to_window, SCREEN_WIDTH, SCREEN_HEIGHT),
                    *progress,
                );
            }
            crate::GuiState::Modal(modal_state) => {
                let Some(background) = self.windows.get(&modal_state.background_pid()) else {
                    log::error!("Modal bg PID {} does not have a window", modal_state.background_pid());
                    return;
                };
                if let Some(modal) = self.windows.get(&modal_state.modal_pid()) {
                    if modal_state.y() > 0 {
                        layers.push(Layer::new_window(background, SCREEN_WIDTH, SCREEN_HEIGHT));
                    }

                    // If we have space for it, darken the background of the modal when it's not fullscreen
                    if !modal_state.is_fullscreen()
                        && control_center_collapsed
                        && layers.layer_count() < MAX_LAYERS - 2
                    {
                        layers.push(
                            Layer::new_single_color(0, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)
                                .with_alpha(modal_state.dark_overlay_alpha()),
                        );
                    }

                    self.add_camera_layer(modal, &mut layers, modal_state.y());

                    layers.push(
                        Layer::new_window(modal, SCREEN_WIDTH, SCREEN_HEIGHT)
                            .with_position(0, modal_state.y()),
                    );

                    // Only add a keyboard if we still have one layer for it and then the control center.
                    // This should only be false if the modal uses the camera and is not full screen.
                    if layers.layer_count() < MAX_LAYERS - 1 {
                        self.add_keyboard_layer(modal, &mut layers);
                    }
                } else {
                    log::trace!(
                        "Modal PID {} does not have a window, we are probably waiting for it",
                        modal_state.modal_pid()
                    );
                    self.add_camera_layer(background, &mut layers, 0);
                    layers.push(Layer::new_window(background, SCREEN_WIDTH, SCREEN_HEIGHT));
                    self.add_keyboard_layer(background, &mut layers);
                }
            }
        };

        if self.is_control_center_visible() {
            if let Some(control_center_window) = &self.control_center_window {
                // If we have space for it, darken the background
                if !control_center_collapsed && layers.layer_count() < MAX_LAYERS - 1 {
                    layers.push(
                        Layer::new_single_color(0, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)
                            .with_alpha(control_center_window.dark_overlay_alpha()),
                    );
                }
                let crop_top = if control_center_window.state == ControlCenterWindowState::Collapsed {
                    0
                } else {
                    CONTROL_CENTER_HEIGHT_EXPANDED_PX - control_center_window.curr_height
                };
                layers.push(
                    Layer::new_double_bufs(
                        &control_center_window.bufs,
                        SCREEN_WIDTH,
                        CONTROL_CENTER_HEIGHT_EXPANDED_PX,
                    )
                    .with_crop(
                        0,
                        crop_top,
                        SCREEN_WIDTH,
                        control_center_window.curr_height,
                    ),
                );
            }
        }

        self.display.setup_layers(layers);
    }

    fn add_camera_layer(&self, window: &AppWindow, layers: &mut LayerStack, offset: usize) {
        #[cfg(feature = "recovery-os")]
        let _ = (window, layers, offset);

        #[cfg(not(feature = "recovery-os"))]
        if window.is_camera_visible() {
            if let Some(camera_window) = &self.camera_window {
                use gui_server_api::consts::{CAMERA_HEIGHT, CAMERA_MARGIN};
                #[cfg(keyos)]
                const CAMERA_PIXEL_FORMAT: LayerPixelFormat = LayerPixelFormat::Rgb565;
                #[cfg(not(keyos))]
                const CAMERA_PIXEL_FORMAT: LayerPixelFormat = LayerPixelFormat::Argb8888;

                let crop_top = CAMERA_MARGIN - window.camera_state.y_pos as usize;
                layers.push(
                    Layer::new_double_bufs(
                        &camera_window.bufs,
                        SCREEN_WIDTH,
                        CAMERA_HEIGHT + CAMERA_MARGIN * 2,
                    )
                    .with_crop(0, crop_top, SCREEN_WIDTH, SCREEN_HEIGHT)
                    .with_position(0, offset)
                    .with_pixel_format(CAMERA_PIXEL_FORMAT),
                );
            }
        }
    }

    fn add_keyboard_layer(&self, window: &AppWindow, layers: &mut LayerStack) {
        if let Some(keyboard_height) = window.keyboard_state.height() {
            if let Some(keyboard_window) = &self.keyboard_window {
                layers.push(
                    Layer::new(
                        keyboard_window.blur_state.blurred_buf().unwrap_or(keyboard_window.bufs.disp_buf),
                        SCREEN_WIDTH,
                        keyboard_height,
                    )
                    .with_position(0, SCREEN_HEIGHT - keyboard_height),
                );
            }
        }
    }
}
