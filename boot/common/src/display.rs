// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        get_pit, FB_BASE_ADDR_0, FB_BASE_ADDR_1, FB_BASE_ADDR_ACTIVE, FB_BASE_ADDR_OFFSCREEN,
        FB_BASE_DMA_ADDR, HEIGHT, WIDTH,
    },
    atsama5d27::{
        display::FramebufDisplay,
        lcdc::{LayerConfig, Lcdc, LcdcLayerId},
        lcdspi::LcdSpi,
        pio::{Direction, Func, Pio, PioB, PioC, PioPort},
        pmc::{PeripheralId, Pmc},
        spi::{ChipSelect, Spi},
    },
    embedded_graphics::{
        draw_target::DrawTarget,
        geometry::{Dimensions, Point},
        pixelcolor::{raw::RawU32, PixelColor, Rgb888},
        prelude::RgbColor,
        primitives::Rectangle,
        Pixel,
    },
    keyos::MASTER_CLOCK_SPEED,
};

pub static mut DISPLAY: Option<FramebufDisplay> = None;

pub fn init_display() {
    unsafe {
        FB_BASE_ADDR_ACTIVE = FB_BASE_ADDR_0;
        FB_BASE_ADDR_OFFSCREEN = FB_BASE_ADDR_1;
        let fb = core::slice::from_raw_parts_mut(FB_BASE_ADDR_OFFSCREEN as *mut u32, WIDTH * HEIGHT);
        DISPLAY = Some(FramebufDisplay::new(fb, WIDTH, HEIGHT));
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Argb8888(pub u32);

impl Argb8888 {
    pub const fn new(a: u8, r: u8, g: u8, b: u8) -> Self { Self(u32::from_le_bytes([a, r, g, b])) }

    pub fn into_inner(self) -> u32 { self.0 }
}

impl PixelColor for Argb8888 {
    type Raw = RawU32;
}

impl RgbColor for Argb8888 {
    const BLACK: Self = Self(0xff000000);
    const BLUE: Self = Self(0xff0000ff);
    const CYAN: Self = Self(Self::GREEN.0 | Self::BLUE.0);
    const GREEN: Self = Self(0xff00ff00);
    const MAGENTA: Self = Self(Self::RED.0 | Self::BLUE.0);
    const MAX_B: u8 = 255;
    const MAX_G: u8 = 255;
    const MAX_R: u8 = 255;
    const RED: Self = Self(0xffff0000);
    const WHITE: Self = Self(0xffffffff);
    const YELLOW: Self = Self(Self::RED.0 | Self::GREEN.0);

    fn r(&self) -> u8 { self.0.to_le_bytes()[1] }

    fn g(&self) -> u8 { self.0.to_le_bytes()[2] }

    fn b(&self) -> u8 { self.0.to_le_bytes()[3] }
}

impl From<Rgb888> for Argb8888 {
    fn from(color: Rgb888) -> Self { Argb8888::new(255, color.r(), color.g(), color.b()) }
}

impl From<Argb8888> for Rgb888 {
    fn from(val: Argb8888) -> Self { Rgb888::new(val.r(), val.g(), val.b()) }
}

pub struct ArgbDisplay {
    fb: &'static mut [u32],
    w: usize,
    h: usize,
}

#[allow(dead_code)]
impl ArgbDisplay {
    pub fn new(fb: &'static mut [u32], w: usize, h: usize) -> ArgbDisplay { ArgbDisplay { fb, w, h } }

    pub fn width(&self) -> usize { self.w }

    pub fn height(&self) -> usize { self.h }
}

impl Dimensions for ArgbDisplay {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::with_corners(Point::new(0, 0), Point::new(self.w as i32, self.h as i32))
    }
}

impl DrawTarget for ArgbDisplay {
    type Color = Argb8888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            let x = point.x as usize;
            let y = point.y as usize;

            // Ensure the point is within bounds
            if x < self.w && y < self.h {
                // Calculate the index for the framebuffer
                let index = self.w * y + x;

                // Write the color value into the framebuffer
                self.fb[index] = color.into_inner();
            }
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        // Fill the framebuffer with the given color
        let color_value = color.into_inner();
        self.fb.fill(color_value);
        Ok(())
    }
}

fn turn_on_lcd() {
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Spi0);
    pmc.enable_system_clock_lcdc();

    let mut pit = get_pit();

    // PB1: reset LCD panel
    let mut rst = Pio::pb1();
    rst.set_func(Func::Gpio);
    rst.set_direction(Direction::Output);
    rst.set(false);
    pit.busy_wait_ms(MASTER_CLOCK_SPEED, 1);
    rst.set(true);
    pit.busy_wait_ms(MASTER_CLOCK_SPEED, 5);

    let mosi = Pio::pa15();
    mosi.set_func(Func::A); // SPI0_MOSI
    let sck = Pio::pa14();
    sck.set_func(Func::A); // SPI0_SPCK
    let cs2 = Pio::pa19();
    cs2.set_func(Func::A); // SPI0_NPCS2
}

fn finalize_lcd() {
    // PB11 - PB31
    PioB::configure_pins_by_mask(None, 0xFFFFF800, Func::A, None);
    PioB::clear_all(None);

    // PC0 - PC8
    PioC::configure_pins_by_mask(None, 0x1ff, Func::A, None);

    let mut lcd_spi = LcdSpi::new(Spi::spi0(), ChipSelect::Cs2, MASTER_CLOCK_SPEED, get_pit());
    lcd_spi.run_init_sequence();
}

pub fn lcd_sleep() {
    let mut lcd_spi = LcdSpi::new(Spi::spi0(), ChipSelect::Cs2, MASTER_CLOCK_SPEED, get_pit());
    lcd_spi.send_command(0x10);
}

pub fn lcd_wake() {
    let mut lcd_spi = LcdSpi::new(Spi::spi0(), ChipSelect::Cs2, MASTER_CLOCK_SPEED, get_pit());
    lcd_spi.send_command(0x11);
}

pub fn init_lcdc(extra_setup: impl FnOnce(&mut Lcdc)) {
    unsafe { core::slice::from_raw_parts_mut(FB_BASE_ADDR_0 as *mut u32, WIDTH * HEIGHT) }.fill(0);
    turn_on_lcd();
    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Lcdc);
    let mut lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    lcdc.init(
        &[LayerConfig::new(LcdcLayerId::Base, FB_BASE_ADDR_0, FB_BASE_DMA_ADDR, FB_BASE_DMA_ADDR)],
        || (),
    );

    lcdc.wait_for_sync_in_progress();
    lcdc.set_lcdc_clk_source(false);
    lcdc.wait_for_sync_in_progress();
    lcdc.set_clock_divider(8);

    extra_setup(&mut lcdc);

    finalize_lcd();
}

pub fn swap_buffers() {
    unsafe {
        if FB_BASE_ADDR_ACTIVE == FB_BASE_ADDR_0 {
            FB_BASE_ADDR_ACTIVE = FB_BASE_ADDR_1;
            FB_BASE_ADDR_OFFSCREEN = FB_BASE_ADDR_0;
        } else {
            FB_BASE_ADDR_ACTIVE = FB_BASE_ADDR_0;
            FB_BASE_ADDR_OFFSCREEN = FB_BASE_ADDR_1;
        }

        let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
        lcdc.update_layer(
            &LayerConfig::new(LcdcLayerId::Base, FB_BASE_ADDR_ACTIVE, FB_BASE_DMA_ADDR, FB_BASE_DMA_ADDR),
            || (),
        );

        // Update the display object with the new offscreen buffer
        let fb = core::slice::from_raw_parts_mut(FB_BASE_ADDR_OFFSCREEN as *mut u32, WIDTH * HEIGHT);
        DISPLAY = Some(FramebufDisplay::new(fb, WIDTH, HEIGHT));
    }
}

pub fn backlight_fade_out() {
    let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);

    let mut pit = get_pit();
    let current_bl = lcdc.pwm_compare_value();
    for bl in (current_bl..0xff).step_by(16) {
        lcdc.set_pwm_compare_value(bl);
        pit.busy_wait_ms(MASTER_CLOCK_SPEED, 10);
    }
}

pub fn backlight_dim(level: u8) {
    let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);

    let mut pit = get_pit();
    let current_bl = lcdc.pwm_compare_value();
    for bl in (current_bl..level).step_by(16) {
        lcdc.set_pwm_compare_value(bl);
        pit.busy_wait_ms(MASTER_CLOCK_SPEED, 10);
    }
}

pub fn backlight_set(level: u8) {
    let lcdc = Lcdc::new(WIDTH as u16, HEIGHT as u16);
    lcdc.set_pwm_compare_value(level);
}
