// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use gui_server_api::{
    consts::{FPS, SCREEN_HEIGHT},
    touch::{Touch, TouchKind},
    DoubleBuffer, GuiApi, InputMessage, Key, NextFrameAnimationKind, Vsync,
};
use server::FromScalar;
#[cfg(not(feature = "recovery-os"))]
use slint::{platform::WindowAdapter, private_unstable_api::re_exports::WindowInner};
use slint::{
    platform::{software_renderer::LineBufferProvider, EventLoopProxy, PointerEventButton, WindowEvent},
    PhysicalPosition, PhysicalSize, PlatformError, SharedString,
};
use xous::envelope::Envelope;
use xous_ticktimer::{Ticktimer, TicktimerCallback};

use crate::{core::EventLoopStatus, pixel::KeyosPixel, window::KeyOsWindow, Runtime, StoredValue};

/// Width of the area on the left edge of the screen from which we detect a swipe right gesture.
const SWIPE_RIGHT_EDGE_AREA_WIDTH_PX: usize = 30; // TODO(SFT-5093): tweak setting
/// Minimum velocity (in pixels per second) required to consider a swipe right gesture valid.
const SWIPE_RIGHT_VELOCITY_THRESHOLD: f32 = 300.; // TODO(SFT-5093): tweak setting
/// Minimum swipe distance (in pixels) required to consider a swipe right gesture valid.
const SWIPE_RIGHT_DISTANCE_THRESHOLD: isize = 100; // TODO(SFT-5093): tweak setting
#[cfg(not(feature = "recovery-os"))]
const DEBUG_TOUCH_COLOR: slint::Color = slint::Color::from_argb_u8(64, 255, 0, 255);
#[cfg(not(feature = "recovery-os"))]
const DEBUG_SWIPE_COLOR: slint::Color = slint::Color::from_argb_u8(64, 0, 255, 0);

pub struct AppInput<PG: GuiAppGuiPermissions> {
    pub win: Rc<KeyOsWindow<PG>>,
    pub msg: InputMessage,
    pub envelope: Envelope,
}

impl<PG: GuiAppGuiPermissions> AppInput<PG> {
    pub fn new(win: Rc<KeyOsWindow<PG>>, msg: InputMessage, envelope: Envelope) -> Self {
        AppInput { win, msg, envelope }
    }
}

#[derive(Clone)]
pub struct AppContext<PG: GuiAppGuiPermissions, PF: GuiAppFsPermissions> {
    pub gui: Arc<GuiApi<PG>>,
    pub fs: Arc<fs::FileSystem<PF>>,

    pub router: StoredValue<crate::router::Router>,
    pub config: Rc<PlatformConfig>,

    handlers: AppHandlers<PG>,
}

impl<PG: GuiAppGuiPermissions, PF: GuiAppFsPermissions> AppContext<PG, PF> {
    pub fn new(gui: Arc<GuiApi<PG>>, fs: Arc<fs::FileSystem<PF>>) -> Self {
        Self {
            gui,
            fs,
            router: StoredValue::new(crate::router::Router::new()),
            config: Rc::new(Default::default()),
            handlers: AppHandlers::default(),
        }
    }

    pub fn set_input_handler(&self, input_handler: impl InputHandler<PG> + 'static) {
        let mut handler = self.handlers.input_handler.borrow_mut();
        let _ = handler.insert(Box::new(input_handler));
    }
}

#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub enable_swipe_back: Cell<bool>,
    pub vsync: Cell<Vsync>,
}

impl Default for PlatformConfig {
    fn default() -> Self { Self { enable_swipe_back: Cell::new(true), vsync: Cell::new(Vsync::CapFPS) } }
}

pub trait InputHandler<PG: GuiAppGuiPermissions>: FnMut(AppInput<PG>) {}
impl<T, PG: GuiAppGuiPermissions> InputHandler<PG> for T where T: FnMut(AppInput<PG>) {}

pub trait ChildrenCrashHandler: FnMut(xous::PID, i32) {}
impl<T> ChildrenCrashHandler for T where T: FnMut(xous::PID, i32) {}

pub trait InputFocusHandler: FnMut(bool) {}
impl<T> InputFocusHandler for T where T: FnMut(bool) {}

#[derive(Default, Clone)]
struct AppHandlers<PG: GuiAppGuiPermissions> {
    input_handler: Rc<RefCell<Option<Box<dyn InputHandler<PG>>>>>,
}

pub trait GuiAppFsPermissions:
    server::CheckedPermissions
    + server::MessageAllowed<fs::messages::OpenDirMessage>
    + server::MessageAllowed<fs::messages::CloseDir>
    + server::MessageAllowed<fs::messages::NextEntry>
    + server::MessageAllowed<fs::messages::MapFileMessage>
{
}

impl<P> GuiAppFsPermissions for P
where
    P: server::CheckedPermissions,
    P: server::MessageAllowed<fs::messages::OpenDirMessage>,
    P: server::MessageAllowed<fs::messages::CloseDir>,
    P: server::MessageAllowed<fs::messages::NextEntry>,
    P: server::MessageAllowed<fs::messages::MapFileMessage>,
{
}

pub trait GuiAppGuiPermissions:
    server::CheckedPermissions
    + 'static
    + server::MessageAllowed<gui_server_api::msg::SwapBuffers>
    + server::MessageAllowed<gui_server_api::msg::UpdateKeyboard>
    + server::MessageAllowed<gui_server_api::msg::HideKeyboard>
    + server::MessageAllowed<gui_server_api::msg::AnimateNextFrame>
{
}

impl<P> GuiAppGuiPermissions for P
where
    P: server::CheckedPermissions + 'static,
    P: server::MessageAllowed<gui_server_api::msg::SwapBuffers>,
    P: server::MessageAllowed<gui_server_api::msg::HideKeyboard>,
    P: server::MessageAllowed<gui_server_api::msg::UpdateKeyboard>,
    P: server::MessageAllowed<gui_server_api::msg::AnimateNextFrame>,
{
}

pub struct KeyOsPlatform<const WIDTH: usize, const HEIGHT: usize, PG: GuiAppGuiPermissions> {
    start: Instant,
    state: RefCell<KeyOsEventLoopState<WIDTH, HEIGHT, PG>>,
}

impl<const WIDTH: usize, const HEIGHT: usize, PG: GuiAppGuiPermissions> KeyOsPlatform<WIDTH, HEIGHT, PG> {
    pub fn new<PF: GuiAppFsPermissions>(
        _app_title: &'static str,
        bufs: DoubleBuffer,
        cx: AppContext<PG, PF>,
    ) -> Self {
        let window = KeyOsWindow::new(cx.gui.clone(), PhysicalSize::new(WIDTH as u32, HEIGHT as u32));

        crate::runtime::handle::global::init();
        crate::fonts::register_fonts(&cx.fs);

        Self {
            start: Instant::now(),
            state: RefCell::new(KeyOsEventLoopState {
                window,
                gui: cx.gui,

                bufs,
                visible: false,
                redraw_callback: None,
                router: cx.router,

                handlers: cx.handlers,
                config: cx.config.clone(),

                swipe_gesture_state: None,
            }),
        }
    }

    #[cfg(not(feature = "recovery-os"))]
    pub fn subscribe_to_theme_changes<PS>(&self)
    where
        PS: server::CheckedPermissions,
        PS: server::MessageAllowed<settings::messages::SubscribeDebugTouch>,
    {
        crate::spawn_local({
            let window = Rc::downgrade(&self.state.borrow().window);
            async move {
                let mut sub = crate::subscribe_scalar::<PS, _>(settings::messages::SubscribeDebugTouch);
                while let Some(debug) = sub.next().await {
                    if let Some(window) = window.upgrade() {
                        let window = WindowInner::from_pub(window.window());
                        if debug.0 {
                            window.set_debug_touch(Some(DEBUG_TOUCH_COLOR));
                            window.set_debug_swipe(Some(DEBUG_SWIPE_COLOR));
                        } else {
                            window.set_debug_touch(None);
                            window.set_debug_swipe(None);
                        }
                    }
                }
            }
        })
        .detach();
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, PG: GuiAppGuiPermissions> slint::platform::Platform
    for KeyOsPlatform<WIDTH, HEIGHT, PG>
{
    fn create_window_adapter(&self) -> Result<Rc<dyn slint::platform::WindowAdapter>, PlatformError> {
        // Since on MCUs, there can be only one window, just return a clone of self.window.
        // We'll also use the same window in the event loop.
        Ok(self.state.borrow().window.clone())
    }

    fn run_event_loop(&self) -> Result<(), PlatformError> {
        let mut state = self.state.borrow_mut();
        state.run();
        Ok(())
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
        Some(Box::new(Runtime::unsafe_handle()))
    }

    fn duration_since_start(&self) -> Duration {
        let the_beginning = self.start;
        Instant::now() - the_beginning
    }

    fn debug_log(&self, arguments: std::fmt::Arguments) {
        log::debug!("{}", arguments);
    }
}

#[allow(dead_code)]
struct KeyOsEventLoopState<const WIDTH: usize, const HEIGHT: usize, PG: GuiAppGuiPermissions> {
    window: Rc<KeyOsWindow<PG>>,
    gui: Arc<GuiApi<PG>>,

    bufs: DoubleBuffer,
    visible: bool,
    redraw_callback: Option<TicktimerCallback>,

    router: StoredValue<crate::router::Router>,

    // if this proves to be a performance issue, we can use a raw pointer instead.
    handlers: AppHandlers<PG>,
    config: Rc<PlatformConfig>,

    // Tracks the time and position of the first touch for swipe detection
    swipe_gesture_state: Option<(Instant, Touch)>,
}

impl<const WIDTH: usize, const HEIGHT: usize, PG: GuiAppGuiPermissions>
    KeyOsEventLoopState<WIDTH, HEIGHT, PG>
{
    pub fn run(&mut self) {
        let mut bufs = self.bufs;
        let mut events = Vec::new();
        let mut last_swap = 0;
        let ticktimer = Ticktimer::new().unwrap();
        loop {
            slint::platform::update_timers_and_animations();

            let work_fb = bufs.work_buf as *mut KeyosPixel;
            let work_fb = unsafe { std::slice::from_raw_parts_mut(work_fb, WIDTH * HEIGHT) };

            while let Some((event, msg)) = self.gui.try_receive_input() {
                self.process_input(event, msg, &mut events);
            }

            {
                let mut event_it = events.drain(..).peekable();
                while let Some(event) = event_it.next() {
                    // Only apply the last PointerMoved event if there are multiple consecutive ones. All
                    // other ones would just be wasted calculation, as just the move is
                    // rarely actionable, but slint does a surprisingly large amount of
                    // calculations for each of these updates.
                    if matches!(event, WindowEvent::PointerMoved { .. })
                        && event_it.peek().map_or(false, |ne| matches!(ne, WindowEvent::PointerMoved { .. }))
                    {
                        continue;
                    }
                    self.window.dispatch_event(event);
                }
            }

            let status = Runtime::unsafe_run();

            if status == EventLoopStatus::Quit {
                break;
            }

            // Draw the scene if something needs to be drawn.
            self.window.draw_if_needed(|renderer| {
                // log::info!("Client: Rendering into {:x}", bufs.work_buf);
                // let now = Instant::now();

                renderer.render_by_line(LineProvider::<WIDTH> {
                    work_fb,
                    last_swap,
                    next_timer_check: 0,
                    ticktimer: &ticktimer,
                });
                #[cfg(keyos)]
                xous::syscall::flush_cache(
                    unsafe { xous::MemoryRange::new(bufs.work_buf, WIDTH * HEIGHT * 4).unwrap() },
                    xous::CacheOperation::Clean,
                )
                .expect("clean cache");

                // let elapsed = now.elapsed();
                // log::debug!("Rendering took {elapsed:?}");

                let vsync = self.config.vsync.get();
                if let Some(new_last_swap) = self.gui.swap_buffers(vsync).expect("swap buffers") {
                    last_swap = new_last_swap;
                    bufs.swap();
                } else {
                    log::warn!("swap_buffers() was unsuccessful");
                }
            });

            let should_block = self.should_block();

            if should_block {
                if let Ok((event, msg)) = self.gui.receive_input() {
                    self.process_input(event, msg, &mut events);
                }
            }
        }
        log::info!("Closing normally (received close request)");
    }

    fn should_block(&mut self) -> bool {
        const MIN_BLOCK_DURATION: Duration = Duration::from_millis(1);

        if !self.visible {
            // If we are not visible, just block until we become visible, and disregard any active timers.
            true
        } else if self.window.has_active_animations() {
            // Never block if we are animating, animate with max framerate
            false
        } else {
            let slint_timer_expiration = slint::platform::duration_until_next_timer_update();

            match slint_timer_expiration {
                // No timers active: block
                None => true,
                // Expired timers: don't block
                Some(duration) if duration < MIN_BLOCK_DURATION => false,
                Some(callback_after) => {
                    log::debug!("Requesting callback in {callback_after:?}");
                    self.redraw_callback
                        .get_or_insert_with(|| TicktimerCallback::new(self.gui.sid()).unwrap())
                        .request(
                            callback_after.as_millis() as usize,
                            InputMessage::RedrawRequested as usize,
                            0,
                        );
                    true
                }
            }
        }
    }

    fn process_input(&mut self, event: InputMessage, msg: Envelope, events: &mut Vec<WindowEvent>) {
        match event {
            InputMessage::Touch => {
                if let Some(touch) = Touch::try_from_input_message(&msg.body) {
                    if self.config.enable_swipe_back.get() {
                        let can_go_back = self.router.with(|r| r.has_back());
                        if can_go_back && self.handle_swipe_right(&touch) {
                            // Swipe right gesture detected, don't propagate the event to GUI
                            return;
                        }
                    }

                    let button = PointerEventButton::Left;
                    let position = PhysicalPosition::new(touch.x as i32, touch.y as i32)
                        .to_logical(self.window.scale_factor());
                    events.push(match touch.kind {
                        TouchKind::Press => WindowEvent::PointerPressed { position, button },
                        TouchKind::Drag => WindowEvent::PointerMoved { position },
                        TouchKind::Release => WindowEvent::PointerReleased { position, button },
                    });
                    if matches!(touch.kind, TouchKind::Release) {
                        events.push(WindowEvent::PointerExited);
                    }
                }
            }

            InputMessage::KeyPress => events.push(self.handle_key_event_msg(true, &msg)),
            InputMessage::KeyRelease => events.push(self.handle_key_event_msg(false, &msg)),
            InputMessage::Visible => {
                log::debug!("App is now visible");
                self.visible = true;
            }
            InputMessage::Hidden => {
                log::debug!("App is hidden");
                if let Some(cb) = &self.redraw_callback {
                    log::trace!("Cancelling redraw timer");
                    cb.cancel(InputMessage::RedrawRequested as usize);
                }
                self.visible = false;
            }
            InputMessage::CloseRequested => Runtime::unsafe_quit(),
            /* do nothing but allow the event loop to redraw */
            InputMessage::RedrawRequested => (),
            _ => (),
        }
        if let Some(handler) = self.handlers.input_handler.borrow_mut().as_mut() {
            handler(AppInput::new(self.window.clone(), event, msg));
        }
    }

    fn handle_key_event_msg(&self, is_press: bool, msg: &Envelope) -> WindowEvent {
        let scalar = msg.body.scalar_message().expect("scalar message");
        let key = Key::from_scalar([scalar.arg1 as u32, scalar.arg2 as u32]);

        let key: SharedString = match key {
            Key::Char(c) => (char::from_u32(c as u32).unwrap_or('?')).into(),
            Key::Backspace => slint::platform::Key::Backspace.into(),
            Key::Delete => slint::platform::Key::Delete.into(),
            Key::CursorLeft => slint::platform::Key::LeftArrow.into(),
            Key::CursorRight => slint::platform::Key::RightArrow.into(),
        };

        if is_press {
            WindowEvent::KeyPressed { text: key }
        } else {
            WindowEvent::KeyReleased { text: key }
        }
    }

    /// Detects and handles the swipe right gesture that navigates the user back with the `Router`.
    /// Returns `true` if the touch must not be propagated to the GUI.
    fn handle_swipe_right(&mut self, touch: &Touch) -> bool {
        match touch.kind {
            TouchKind::Press if touch.x <= SWIPE_RIGHT_EDGE_AREA_WIDTH_PX => {
                log::debug!("Detected initial swipe right touch at ({}, {})", touch.x, touch.y);
                self.swipe_gesture_state = Some((Instant::now(), touch.clone()));
                false
            }

            TouchKind::Drag => {
                if self.swipe_gesture_state.is_none() {
                    if touch.x <= SWIPE_RIGHT_EDGE_AREA_WIDTH_PX {
                        log::debug!("Detected initial swipe right drag at ({}, {})", touch.x, touch.y);
                        self.swipe_gesture_state = Some((Instant::now(), touch.clone()));
                        return true;
                    }
                } else {
                    return true;
                }

                false
            }

            TouchKind::Release => {
                if let Some((first_touch_time, first_touch)) = self.swipe_gesture_state.take() {
                    let elapsed = first_touch_time.elapsed().as_secs_f32();
                    let (dx, _dy) = touch.diff(&first_touch);
                    let velocity = if elapsed != 0. { dx as f32 / elapsed } else { 0. };
                    log::debug!("Swipe right gesture velocity: {velocity} and dx: {dx}");

                    if velocity >= SWIPE_RIGHT_VELOCITY_THRESHOLD && dx >= SWIPE_RIGHT_DISTANCE_THRESHOLD {
                        log::debug!("Detected, navigating backward");
                        self.gui.animate_next_frame(NextFrameAnimationKind::SlideOutRight).ok();
                        self.router.with(|r| r.navigate_backward());
                        return true;
                    }
                }

                false
            }

            _ => {
                self.swipe_gesture_state = None;
                false
            }
        }
    }
}

struct LineProvider<'a, const WIDTH: usize> {
    work_fb: &'a mut [KeyosPixel],
    last_swap: u64,
    next_timer_check: usize,
    ticktimer: &'a Ticktimer,
}

impl<const WIDTH: usize> LineBufferProvider for LineProvider<'_, WIDTH> {
    type TargetPixel = KeyosPixel;

    fn process_line(
        &mut self,
        line: usize,
        range: std::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [KeyosPixel]),
    ) {
        if line >= self.next_timer_check {
            let time_of_line = 1000 * line / SCREEN_HEIGHT / FPS;
            // The additional milliseconds at the end is to mask inaccuracies
            let lcdc_estimated_render_tick = self.last_swap + time_of_line as u64 + 2;
            let current_tick = self.ticktimer.elapsed_ms();
            // If we would overtake the LCDC line scan, wait a bit instead.
            if lcdc_estimated_render_tick > current_tick {
                std::thread::sleep(Duration::from_millis(lcdc_estimated_render_tick - current_tick));
            }
            self.next_timer_check = line + 100;
        }
        render_fn(&mut self.work_fb[line * WIDTH..][range]);
    }
}
