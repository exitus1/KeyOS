// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        boot_screen::set_continue_boot,
        gui::Page,
        securam::set_os_arguments,
        select_recovery_image,
        verify::{get_bootloader_version_and_date, load_os_version_info},
    },
    boot_common::{
        colors::DarkPalette,
        display::backlight_fade_out,
        gui::{Button, ButtonType, Menu, MenuItem, Text, BUTTON_HEIGHT},
        shutdown,
        theme::UISize,
        HEIGHT, WIDTH,
    },
    securam_manager::OsArguments,
};

pub struct MenuPage {
    page: Page,
}

impl MenuPage {
    pub fn new() -> Self {
        let mut page = Page::new(WIDTH as i32, HEIGHT as i32);

        // Initial positions for components
        let curr_x = UISize::SZ10;
        let mut curr_y = UISize::SZ10;
        let main_width = page.bounds().width - 2 * UISize::SZ10;

        // Add the title
        page.add_component(Text::new(
            curr_x,
            curr_y,
            main_width,
            BUTTON_HEIGHT,
            "BOOT MENU",
            true,
            DarkPalette::WHITE,
        ));
        curr_y += BUTTON_HEIGHT + UISize::SZ6;

        // Create and add the menu
        let mut menu = Menu::new(curr_x, curr_y, main_width);

        // Add menu items
        menu.add_item(MenuItem::new(Some("i"), "System Information", Some(">"), || {
            let (bootloader_version, bootloader_build_date) = get_bootloader_version_and_date();
            set_os_arguments(&OsArguments::SystemInfoMode { bootloader_version, bootloader_build_date }).ok();
            select_recovery_image();
            load_os_version_info();
            set_continue_boot(true);
        }));

        menu.add_item(MenuItem::new(Some("S"), "Recovery", Some(">"), || {
            let (bootloader_version, bootloader_build_date) = get_bootloader_version_and_date();
            set_os_arguments(&OsArguments::RecoveryMode { bootloader_version, bootloader_build_date }).ok();
            select_recovery_image();
            load_os_version_info();
            set_continue_boot(true);
        }));

        #[cfg(not(feature = "production"))]
        menu.add_item(MenuItem::new(None, "(Debug) SAM-BA mode", Some(">"), boot_common::enter_sam_ba_mode));

        page.add_component(menu);

        // Add the buttons
        curr_y = page.bounds().height - UISize::SZ10 - BUTTON_HEIGHT - UISize::SZ4 - BUTTON_HEIGHT;

        page.add_component(Button::new(
            curr_x,
            curr_y,
            main_width,
            Some("P"),
            Some("Start KeyOS"),
            ButtonType::Primary,
            || {
                set_continue_boot(true);
            },
        ));
        curr_y += BUTTON_HEIGHT + UISize::SZ4;

        page.add_component(Button::new(
            curr_x,
            curr_y,
            main_width,
            Some("p"),
            Some("Shut Down"),
            ButtonType::Primary,
            || {
                backlight_fade_out();
                shutdown();
            },
        ));

        // Return the MenuPage instance
        Self { page }
    }

    pub fn render(&self) { self.page.render(); }

    pub fn on_press(&mut self, x: i32, y: i32) { self.page.on_press(x, y); }

    pub fn on_release(&mut self, x: i32, y: i32) { self.page.on_release(x, y); }
}
