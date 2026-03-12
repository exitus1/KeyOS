// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        colors::DarkPalette,
        display::DISPLAY,
        fonts::{ICON_FONT, SOURCE_CODE_PRO_FONT},
        theme::UISize,
    },
    arrayvec::ArrayVec,
    core::str,
    embedded_graphics::{
        mono_font::{MonoFont, MonoTextStyle},
        pixelcolor::Rgb888,
        prelude::{Point, Size, *},
        primitives::{
            CornerRadiiBuilder, PrimitiveStyleBuilder, Rectangle, RoundedRectangle, StrokeAlignment,
        },
        text::Text as EGText,
        Drawable,
    },
    enum_dispatch::enum_dispatch,
};

// An arbitrary limit on the number of the lines in the error message to be displayed
pub const MAX_MESSAGE_LINES: usize = 32;

// Maximum number of menu items per menu
pub const MAX_MENU_ITEMS: usize = 4;

const BORDER_W: i32 = 2;

pub fn text_width(text: &str, font: &MonoFont) -> i32 { text.len() as i32 * font.character_size.width as i32 }

// Primitives for drawing the UI
fn draw_text(x: i32, y: i32, text: &str, font: &MonoFont, color: Rgb888) {
    if let Some(display) = unsafe { (*core::ptr::addr_of_mut!(DISPLAY)).as_mut() } {
        EGText::new(text, Point::new(x, y), MonoTextStyle::new(font, color)).draw(display).ok();
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_table_cell(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    left_icon: Option<&str>,
    label: &str,
    right_icon: Option<&str>,
    background_color: Rgb888,
    background_color_pressed: Rgb888,
    outline_color: Rgb888,
    is_pressed: bool,
    has_round_top: bool,
    has_round_bottom: bool,
) {
    if let Some(display) = unsafe { (*core::ptr::addr_of_mut!(DISPLAY)).as_mut() } {
        // Determine fill color based on press state
        let fill_color = if is_pressed { background_color_pressed } else { background_color };

        // Draw the background
        let style = PrimitiveStyleBuilder::new()
            .fill_color(fill_color)
            .stroke_width(2)
            .stroke_alignment(StrokeAlignment::Inside)
            .stroke_color(outline_color)
            .build();

        let round_size = Size::new(UISize::SZ4 as u32, UISize::SZ4 as u32);
        let square_size = Size::new(0, 0);

        let radii = CornerRadiiBuilder::new()
            .top_left(if has_round_top { round_size } else { square_size })
            .top_right(if has_round_top { round_size } else { square_size })
            .bottom_right(if has_round_bottom { round_size } else { square_size })
            .bottom_left(if has_round_bottom { round_size } else { square_size })
            .build();

        RoundedRectangle::new(
            Rectangle::new(Point::new(x, y), Size::new(width as u32, height as u32)),
            radii,
        )
        .into_styled(style)
        .draw(display)
        .ok();

        // Prepare for drawing icons and label
        let icon_font = ICON_FONT;
        let label_font = SOURCE_CODE_PRO_FONT;
        let icon_height = icon_font.character_size.height as i32;
        let label_height = label_font.character_size.height as i32;
        let label_offset_y = 3;

        let mut curr_x = x + UISize::SZ4 + BORDER_W;

        // Draw the left icon if available
        if let Some(left_icon) = left_icon {
            let left_icon_width = text_width(left_icon, &icon_font);
            draw_text(
                curr_x,
                y + (height - icon_height) / 2,
                left_icon,
                if left_icon == "<" {
                    // Only use label_font for "<"
                    &label_font
                } else {
                    &icon_font
                },
                DarkPalette::CONTENT_PRIMARY,
            );
            curr_x += left_icon_width + UISize::SZ4;
        }

        // Draw the label
        draw_text(
            curr_x,
            y + label_offset_y + (height - label_height) / 2,
            label,
            &label_font,
            DarkPalette::CONTENT_PRIMARY,
        );

        // Draw the right icon if provided
        if let Some(right_icon) = right_icon {
            let right_icon_width = text_width(right_icon, &icon_font);
            draw_text(
                x + width - UISize::SZ4 - 1 - right_icon_width,
                y + (height - icon_height) / 2,
                right_icon,
                if right_icon == ">" {
                    // Only use label_font for ">"
                    &label_font
                } else {
                    &icon_font
                },
                DarkPalette::CONTENT_PRIMARY,
            );
        }
    }
}

fn draw_line(start_x: i32, start_y: i32, end_x: i32, end_y: i32, color: Rgb888, stroke_width: u32) {
    if let Some(display) = unsafe { (*core::ptr::addr_of_mut!(DISPLAY)).as_mut() } {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(stroke_width)
            .stroke_alignment(StrokeAlignment::Center) // This is the default, but making it explicit
            .build();

        embedded_graphics::primitives::Line::new(Point::new(start_x, start_y), Point::new(end_x, end_y))
            .into_styled(style)
            .draw(display)
            .ok();
    }
}
#[derive(Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Bounds {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self { Self { x, y, width, height } }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

//=============================================================================
// Component Trait
//=============================================================================
#[enum_dispatch]
pub trait Component {
    fn render(&self);

    // TODO: Consider returning a bool to indicate if the touch was handled
    fn bounds(&self) -> &Bounds;
    fn mut_bounds(&mut self) -> &mut Bounds;

    fn on_press(&mut self, _x: i32, _y: i32) {}
    fn on_release(&mut self, _x: i32, _y: i32) {}
    fn on_drag(&mut self, _x: i32, _y: i32) {}
}

//=============================================================================
// Base Components
//=============================================================================

#[enum_dispatch(Component)]
pub enum DynComponent<T: Component> {
    Text,
    Line,
    TextMessage,
    Button,
    Menu,
    CustomComponent(CustomComponent<T>),
}

pub struct CustomComponent<T: Component>(pub T);

impl<T: Component> Component for CustomComponent<T> {
    fn render(&self) { self.0.render() }

    fn bounds(&self) -> &Bounds { self.0.bounds() }

    fn mut_bounds(&mut self) -> &mut Bounds { self.0.mut_bounds() }

    fn on_press(&mut self, x: i32, y: i32) { self.0.on_press(x, y); }

    fn on_release(&mut self, x: i32, y: i32) { self.0.on_release(x, y); }

    fn on_drag(&mut self, x: i32, y: i32) { self.0.on_drag(x, y); }
}

//=============================================================================
// Page Component
//=============================================================================
pub struct Page<T: Component> {
    bounds: Bounds,
    components: ArrayVec<T, 32>,
    captured_touch_index: Option<usize>,
}

impl<T: Component> Page<T> {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            bounds: Bounds::new(0, 0, width, height),
            components: ArrayVec::new(),
            captured_touch_index: None,
        }
    }

    pub fn add_component(&mut self, component: impl Into<T>) { self.components.push(component.into()); }

    pub fn pop_component(&mut self) -> Option<T> { self.components.pop() }

    pub fn render(&self) {
        // log_with_u32("Num components on page: ", self.num_components as u32);
        self.components.iter().for_each(|component| component.render());
    }

    pub fn bounds(&self) -> &Bounds { &self.bounds }

    pub fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }

    pub fn on_press(&mut self, x: i32, y: i32) {
        if let Some((i, component)) =
            self.components.iter_mut().enumerate().find(|(_, component)| component.bounds().contains(x, y))
        {
            self.captured_touch_index = Some(i);
            component.on_press(x, y);
        }
    }

    pub fn on_release(&mut self, x: i32, y: i32) {
        if let Some(i) = self.captured_touch_index {
            self.components[i].on_release(x, y);
            self.captured_touch_index = None;
        }
    }

    pub fn on_drag(&mut self, x: i32, y: i32) {
        if let Some(i) = self.captured_touch_index {
            self.components[i].on_drag(x, y);
        }
    }
}

//=============================================================================
// MenuItem
//=============================================================================
pub struct MenuItem {
    pub bounds: Bounds,
    pub left_icon: Option<&'static str>,
    pub title: &'static str,
    pub right_icon: Option<&'static str>,
    pub on_select: fn(),
    pub is_pressed: bool,
    pub has_round_top: bool,
    pub has_round_bottom: bool,
}

impl MenuItem {
    pub fn new(
        left_icon: Option<&'static str>,
        title: &'static str,
        right_icon: Option<&'static str>,
        on_select: fn(),
    ) -> Self {
        Self {
            // Position and size will be set by the Menu
            bounds: Bounds::new(0, 0, 0, 0),
            left_icon,
            title,
            right_icon,
            on_select,
            is_pressed: false,
            has_round_top: false,
            has_round_bottom: false,
        }
    }

    fn render(&self) {
        draw_table_cell(
            self.bounds().x,
            self.bounds().y,
            self.bounds().width,
            self.bounds().height,
            self.left_icon,
            self.title,
            self.right_icon,
            DarkPalette::BLACK,
            DarkPalette::BACKGROUND_SELECTED,
            DarkPalette::WHITE,
            self.is_pressed,
            self.has_round_top,
            self.has_round_bottom,
        );
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }
}

//=============================================================================
// Menu
//=============================================================================

const MENU_ITEM_HEIGHT: i32 = 72;

pub struct Menu {
    bounds: Bounds,
    items: ArrayVec<MenuItem, MAX_MENU_ITEMS>,
    next_y: i32,
    pressed_index: Option<usize>,
}

impl Menu {
    pub fn new(x: i32, y: i32, width: i32) -> Self {
        Self {
            // Height is updated when items are added
            bounds: Bounds::new(x, y, width, 0),
            items: ArrayVec::new(),
            next_y: y,
            pressed_index: None,
        }
    }

    pub fn add_item(&mut self, mut item: MenuItem) {
        // Set the bounds of the new item
        let bounds = self.bounds();
        let item_bounds = item.mut_bounds();
        item_bounds.x = bounds.x;
        item_bounds.y = self.next_y;
        item_bounds.width = bounds.width;
        item_bounds.height = MENU_ITEM_HEIGHT;

        // Add item to the list
        self.items.push(item);

        // Update layout
        self.next_y += MENU_ITEM_HEIGHT - BORDER_W;
        self.update_layout();
    }

    fn update_layout(&mut self) {
        let num_items = self.items.len();

        // Adjust each item's layout
        for (index, item) in self.items.iter_mut().enumerate() {
            item.has_round_top = index == 0;
            item.has_round_bottom = index == num_items - 1;
        }

        // Update the menu's total height
        self.mut_bounds().height = MENU_ITEM_HEIGHT * num_items as i32;
    }
}

impl Component for Menu {
    fn render(&self) {
        // log_with_u32("Num menu items: ", self.num_items as u32);
        for item in self.items.iter() {
            item.render();
        }
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }

    fn on_press(&mut self, x: i32, y: i32) {
        for (index, item) in self.items.iter_mut().enumerate() {
            if item.bounds().contains(x, y) {
                // Add logic to select menu item
                item.is_pressed = true;
                self.pressed_index = Some(index);
                return;
            }
        }
    }

    fn on_release(&mut self, _x: i32, _y: i32) {
        if let Some(index) = self.pressed_index {
            (self.items[index].on_select)();
            self.items[index].is_pressed = false;
            self.pressed_index = None;
        }
    }
}

//=============================================================================
// Button
//=============================================================================

pub const BUTTON_HEIGHT: i32 = 64;

#[derive(Clone, PartialEq)]
pub enum ButtonType {
    Primary,
    Secondary,
}

pub struct Button {
    bounds: Bounds,
    icon: Option<&'static str>,
    label: Option<&'static str>,
    button_type: ButtonType,
    on_click: fn(),
    is_pressed: bool,
}

impl Button {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        icon: Option<&'static str>,
        label: Option<&'static str>,
        button_type: ButtonType,
        on_click: fn(),
    ) -> Self {
        Self {
            bounds: Bounds::new(x, y, width, BUTTON_HEIGHT),
            icon,
            label,
            button_type,
            on_click,
            is_pressed: false,
        }
    }
}

impl Component for Button {
    fn render(&self) {
        if let Some(display) = unsafe { (*core::ptr::addr_of_mut!(DISPLAY)).as_mut() } {
            let bounds = self.bounds();
            let button_rect = Rectangle::new(
                Point::new(bounds.x, bounds.y),
                Size::new(bounds.width as u32, bounds.height as u32),
            );

            // Draw the button background
            let fill_color = if self.is_pressed {
                DarkPalette::BACKGROUND_BRAND_PRESSED
            } else {
                match self.button_type {
                    ButtonType::Primary => DarkPalette::BACKGROUND_BRAND,
                    ButtonType::Secondary => DarkPalette::BACKGROUND_DISABLED,
                }
            };

            let style = PrimitiveStyleBuilder::new().fill_color(fill_color).build();
            RoundedRectangle::with_equal_corners(
                button_rect,
                Size::new(UISize::SZ6 as u32, UISize::SZ6 as u32),
            )
            .into_styled(style)
            .draw(display)
            .ok();

            // Draw the icon and label
            let label_font = SOURCE_CODE_PRO_FONT;
            let icon_font = ICON_FONT;

            let label_width = self.label.as_ref().map_or(0, |label| text_width(label, &label_font));
            let icon_width = self.icon.as_ref().map_or(0, |icon| text_width(icon, &icon_font));
            let separation = if icon_width > 0 && label_width > 0 { UISize::SZ4 } else { 0 };
            let content_width = icon_width + separation + label_width;
            let mut curr_x = bounds.x + (bounds.width - content_width) / 2;
            let offset_y = 3;

            // Draw the icon, if given
            if let Some(icon) = &self.icon {
                let (font, y_offset) = if *icon == "<" {
                    // Only use label_font for "<"
                    (&label_font, 4)
                } else {
                    (&icon_font, 0)
                };
                let icon_y = bounds.y + (bounds.height - font.character_size.height as i32 + y_offset) / 2;
                draw_text(curr_x, icon_y, icon, font, DarkPalette::WHITE);
                curr_x += icon_width + separation;
            }

            // Draw the label, if given
            if let Some(label) = &self.label {
                let label_y =
                    bounds.y + ((bounds.height - label_font.character_size.height as i32) / 2) + offset_y;
                draw_text(curr_x, label_y, label, &label_font, DarkPalette::WHITE);
            }
        }
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }

    fn on_press(&mut self, _x: i32, _y: i32) { self.is_pressed = true; }

    fn on_release(&mut self, _x: i32, _y: i32) {
        self.is_pressed = false;
        (self.on_click)();
    }
}

//=============================================================================
// Text
//=============================================================================
pub struct Text {
    bounds: Bounds,
    text: &'static str,
    is_centered: bool,
    color: Rgb888,
}

impl Text {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        text: &'static str,
        is_centered: bool,
        color: Rgb888,
    ) -> Self {
        let font = SOURCE_CODE_PRO_FONT;
        let actual_height = if height == 0 { font.character_size.height as i32 } else { height };
        Self { bounds: Bounds::new(x, y, width, actual_height), text, is_centered, color }
    }
}

impl Component for Text {
    fn render(&self) {
        let font = SOURCE_CODE_PRO_FONT;
        let height = font.character_size.height as i32;

        let bounds = self.bounds();

        // Set x to get text centered if required
        let text_x = if self.is_centered {
            bounds.x + (bounds.width / 2) - (text_width(self.text, &font) / 2)
        } else {
            bounds.x
        };

        // Center vertically
        let text_y = bounds.y + (bounds.height / 2) - (height / 2);

        // Safely access the display and render the text
        if let Some(display) = unsafe { (*core::ptr::addr_of_mut!(DISPLAY)).as_mut() } {
            EGText::new(self.text, Point::new(text_x, text_y), MonoTextStyle::new(&font, self.color))
                .draw(display)
                .ok();
        }
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }
}

pub struct TextMessage {
    bounds: Bounds,
    lines: ArrayVec<&'static str, MAX_MESSAGE_LINES>,
    is_centered: bool,
    scroll_offset: i32,
    line_height: i32,
    is_scrolling: bool,
    last_touch_y: i32,
}

impl TextMessage {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        lines: ArrayVec<&'static str, MAX_MESSAGE_LINES>,
        is_centered: bool,
    ) -> Self {
        let font = SOURCE_CODE_PRO_FONT;
        let line_height = font.character_size.height as i32 + 1;
        Self {
            bounds: Bounds::new(x, y, width, 0),
            lines,
            is_centered,
            scroll_offset: 0,
            line_height,
            is_scrolling: false,
            last_touch_y: 0,
        }
    }
}

impl Component for TextMessage {
    fn render(&self) {
        let font = SOURCE_CODE_PRO_FONT;
        let total_content_height = self.lines.len() as i32 * self.line_height;
        let available_height = self.bounds.height;

        // Calculate visible line range based on scroll offset
        let start_line = (-self.scroll_offset / self.line_height).max(0) as usize;
        let end_line = ((available_height - self.scroll_offset) / self.line_height) as usize + 1;
        let end_line = end_line.min(self.lines.len());

        for (line_idx, line) in self.lines.iter().enumerate().skip(start_line).take(end_line - start_line) {
            let line_y =
                self.bounds.y + UISize::SZ4 + self.scroll_offset + (line_idx as i32 * self.line_height);

            if line_y + self.line_height > self.bounds.y && line_y < self.bounds.y + available_height {
                let text_x = if self.is_centered {
                    self.bounds.x + (self.bounds.width / 2) - (text_width(line, &font) / 2)
                } else {
                    self.bounds.x
                };

                draw_text(text_x, line_y, line, &font, DarkPalette::WHITE);
            }
        }

        // Draw scroll indicator if content is scrollable
        if total_content_height > available_height {
            let scroll_bar_height = (available_height * available_height) / total_content_height;
            let scroll_bar_y =
                self.bounds.y + ((-self.scroll_offset * available_height) / total_content_height);
            let scroll_bar_x = self.bounds.x + self.bounds.width - 4;

            // Draw scroll bar background
            draw_line(
                scroll_bar_x,
                self.bounds.y,
                scroll_bar_x,
                self.bounds.y + available_height,
                DarkPalette::BORDER_TERTIARY,
                2,
            );

            // Draw scroll bar thumb
            let thumb_height = scroll_bar_height.max(10);
            let thumb_y =
                scroll_bar_y.max(self.bounds.y).min(self.bounds.y + available_height - thumb_height);
            draw_line(scroll_bar_x, thumb_y, scroll_bar_x, thumb_y + thumb_height, DarkPalette::WHITE, 2);
        }
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }

    fn on_press(&mut self, x: i32, y: i32) {
        // Always allow scrolling to start when touching within bounds
        // The bounds checking ensures we only respond to touches in our area
        if self.bounds.contains(x, y) {
            self.is_scrolling = true;
            self.last_touch_y = y;
        }
    }

    fn on_release(&mut self, _x: i32, _y: i32) { self.is_scrolling = false; }

    fn on_drag(&mut self, _x: i32, y: i32) {
        if self.is_scrolling {
            let delta_y = y - self.last_touch_y;
            self.last_touch_y = y;

            // Update scroll offset
            self.scroll_offset += delta_y;

            // Calculate and clamp scroll bounds
            let total_content_height = self.lines.len() as i32 * self.line_height;
            let available_height = self.bounds.height;
            let max_scroll = (total_content_height - available_height).max(0);

            // Ensure scroll_offset stays within valid bounds
            // Negative values scroll up (showing content from higher up)
            // 0 shows the top of the content
            // -max_scroll shows the bottom of the content
            self.scroll_offset = self.scroll_offset.clamp(-max_scroll, 0);
        }
    }
}

pub struct Line {
    bounds: Bounds,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    color: Rgb888,
    stroke_width: u32,
}

impl Line {
    pub fn new(start_x: i32, start_y: i32, end_x: i32, end_y: i32, color: Rgb888, stroke_width: u32) -> Self {
        // Calculate bounds that encompass the line
        let x = start_x.min(end_x);
        let y = start_y.min(end_y);
        let width = (start_x - end_x).abs();
        let height = (start_y - end_y).abs();

        Self { bounds: Bounds::new(x, y, width, height), start_x, start_y, end_x, end_y, color, stroke_width }
    }
}

impl Component for Line {
    fn render(&self) {
        draw_line(self.start_x, self.start_y, self.end_x, self.end_y, self.color, self.stroke_width);
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn mut_bounds(&mut self) -> &mut Bounds { &mut self.bounds }
}
