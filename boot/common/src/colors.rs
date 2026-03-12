// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(dead_code)]

use {crate::display::Argb8888, embedded_graphics::pixelcolor::Rgb888};

pub struct UIColors;

impl UIColors {
    // #8ad2df
    pub const BLUE_100: Rgb888 = Rgb888::new(176, 225, 233);
    // #54bdd0
    pub const BLUE_200: Rgb888 = Rgb888::new(138, 210, 223);
    // #33b1c7
    pub const BLUE_300: Rgb888 = Rgb888::new(84, 189, 208);
    // #009db9
    pub const BLUE_400: Rgb888 = Rgb888::new(51, 177, 199);
    // #b0e1e9
    pub const BLUE_50: Rgb888 = Rgb888::new(230, 245, 248);
    // #008fa8
    pub const BLUE_500: Rgb888 = Rgb888::new(0, 157, 185);
    // #006f83
    pub const BLUE_600: Rgb888 = Rgb888::new(0, 143, 168);
    // #005666
    pub const BLUE_700: Rgb888 = Rgb888::new(0, 111, 131);
    // #00424e
    pub const BLUE_800: Rgb888 = Rgb888::new(0, 86, 102);
    // #f1eeec

    // Blue
    pub const BLUE_900: Rgb888 = Rgb888::new(0, 66, 78);
    // #bfafa9
    pub const BRONZE_100: Rgb888 = Rgb888::new(212, 201, 197);
    // #a18a82
    pub const BRONZE_200: Rgb888 = Rgb888::new(191, 175, 169);
    // #8f736a
    pub const BRONZE_300: Rgb888 = Rgb888::new(161, 138, 130);
    // #735045
    pub const BRONZE_400: Rgb888 = Rgb888::new(143, 115, 106);
    // #d4c9c5
    pub const BRONZE_50: Rgb888 = Rgb888::new(241, 238, 236);
    // #69493f
    pub const BRONZE_500: Rgb888 = Rgb888::new(115, 80, 69);
    // #523931
    pub const BRONZE_600: Rgb888 = Rgb888::new(105, 73, 63);
    // #3f2c26
    pub const BRONZE_700: Rgb888 = Rgb888::new(82, 57, 49);
    // #30221d
    pub const BRONZE_800: Rgb888 = Rgb888::new(63, 44, 38);
    // #fbf3f1

    // Bronze
    pub const BRONZE_900: Rgb888 = Rgb888::new(48, 34, 29);
    // #e2c0b5
    pub const DARK_COPPER_100: Rgb888 = Rgb888::new(235, 212, 205);
    // #d4a394
    pub const DARK_COPPER_200: Rgb888 = Rgb888::new(226, 192, 181);
    // #cc917f
    pub const DARK_COPPER_300: Rgb888 = Rgb888::new(212, 163, 148);
    // #bf755f
    pub const DARK_COPPER_400: Rgb888 = Rgb888::new(204, 145, 127);
    // #ebd4cd
    pub const DARK_COPPER_50: Rgb888 = Rgb888::new(251, 243, 241);
    // #ae6a56
    pub const DARK_COPPER_500: Rgb888 = Rgb888::new(191, 117, 95);
    // #885343
    pub const DARK_COPPER_600: Rgb888 = Rgb888::new(174, 106, 86);
    // #694034
    pub const DARK_COPPER_700: Rgb888 = Rgb888::new(136, 83, 67);
    // #503128
    pub const DARK_COPPER_800: Rgb888 = Rgb888::new(105, 64, 52);
    // #f6f6f6

    // Dark Copper
    pub const DARK_COPPER_900: Rgb888 = Rgb888::new(80, 49, 40);
    // #9fcec6
    pub const GREEN_100: Rgb888 = Rgb888::new(190, 222, 217);
    // #73b7ac
    pub const GREEN_200: Rgb888 = Rgb888::new(159, 206, 198);
    // #58a99c
    pub const GREEN_300: Rgb888 = Rgb888::new(115, 183, 172);
    // #2e9483
    pub const GREEN_400: Rgb888 = Rgb888::new(88, 169, 156);
    // #beded9
    pub const GREEN_50: Rgb888 = Rgb888::new(234, 244, 243);
    // #2a8777
    pub const GREEN_500: Rgb888 = Rgb888::new(46, 148, 131);
    // #21695d
    pub const GREEN_600: Rgb888 = Rgb888::new(42, 135, 119);
    // #195148
    pub const GREEN_700: Rgb888 = Rgb888::new(33, 105, 93);
    // #133e37
    pub const GREEN_800: Rgb888 = Rgb888::new(25, 81, 72);
    // #e6f2f2

    // Green
    pub const GREEN_900: Rgb888 = Rgb888::new(19, 62, 55);
    // #eccabc
    pub const LIGHT_COPPER_100: Rgb888 = Rgb888::new(242, 219, 210);
    // #e4b19e
    pub const LIGHT_COPPER_200: Rgb888 = Rgb888::new(236, 202, 188);
    // #dea28b
    pub const LIGHT_COPPER_300: Rgb888 = Rgb888::new(228, 177, 158);
    // #d68b6e
    pub const LIGHT_COPPER_400: Rgb888 = Rgb888::new(222, 162, 139);
    // #f2dbd2
    pub const LIGHT_COPPER_50: Rgb888 = Rgb888::new(251, 243, 241);
    // #c37e64
    pub const LIGHT_COPPER_500: Rgb888 = Rgb888::new(214, 139, 110);
    // #98634e
    pub const LIGHT_COPPER_600: Rgb888 = Rgb888::new(195, 126, 100);
    // #764c3d
    pub const LIGHT_COPPER_700: Rgb888 = Rgb888::new(152, 99, 78);
    // #523a2e
    pub const LIGHT_COPPER_800: Rgb888 = Rgb888::new(118, 76, 61);
    // #fbf3f1

    // Light Copper
    pub const LIGHT_COPPER_900: Rgb888 = Rgb888::new(82, 58, 46);
    // #d5d4d5
    pub const NEUTRAL_100: Rgb888 = Rgb888::new(227, 226, 226);
    // #c2c1c1
    pub const NEUTRAL_200: Rgb888 = Rgb888::new(213, 212, 213);
    // #b6b5b5
    pub const NEUTRAL_300: Rgb888 = Rgb888::new(194, 193, 193);
    // #a4a2a3
    pub const NEUTRAL_400: Rgb888 = Rgb888::new(182, 181, 181);
    // #e3e2e2
    pub const NEUTRAL_50: Rgb888 = Rgb888::new(246, 246, 246);
    // #959394
    pub const NEUTRAL_500: Rgb888 = Rgb888::new(164, 162, 163);
    // #747374
    pub const NEUTRAL_600: Rgb888 = Rgb888::new(149, 147, 148);
    // #5a595a
    pub const NEUTRAL_700: Rgb888 = Rgb888::new(116, 115, 116);
    // #454444
    pub const NEUTRAL_800: Rgb888 = Rgb888::new(90, 89, 90);
    // #231f20
    pub const NEUTRAL_900: Rgb888 = Rgb888::new(69, 68, 68);
    // #00000000

    // Neutral colors
    pub const NEUTRAL_950: Rgb888 = Rgb888::new(35, 31, 32);
    // #8ac2c2
    pub const PINE_100: Rgb888 = Rgb888::new(176, 214, 214);
    // #54a6a6
    pub const PINE_200: Rgb888 = Rgb888::new(138, 194, 194);
    // #339595
    pub const PINE_300: Rgb888 = Rgb888::new(84, 166, 166);
    // #007a7a
    pub const PINE_400: Rgb888 = Rgb888::new(51, 149, 149);
    // #b0d6d6
    pub const PINE_50: Rgb888 = Rgb888::new(230, 242, 242);
    // #006f6f
    pub const PINE_500: Rgb888 = Rgb888::new(0, 122, 122);
    // #005757
    pub const PINE_600: Rgb888 = Rgb888::new(0, 111, 111);
    // #004343
    pub const PINE_700: Rgb888 = Rgb888::new(0, 87, 87);
    // #003333
    pub const PINE_800: Rgb888 = Rgb888::new(0, 67, 67);
    // #e6f6f7

    // Pine
    pub const PINE_900: Rgb888 = Rgb888::new(0, 51, 51);
    // #ffffff
    pub const PRIMARY_BLACK: Rgb888 = Rgb888::new(0, 0, 0);
    // Primary colors
    pub const PRIMARY_WHITE: Rgb888 = Rgb888::new(255, 255, 255);
    // #d2b2ff
    pub const PURPLE_100: Rgb888 = Rgb888::new(225, 203, 255);
    // #be8eff
    pub const PURPLE_200: Rgb888 = Rgb888::new(210, 178, 255);
    // #b179ff
    pub const PURPLE_300: Rgb888 = Rgb888::new(190, 142, 255);
    // #9e57ff
    pub const PURPLE_400: Rgb888 = Rgb888::new(177, 121, 255);
    // #e1cbff
    pub const PURPLE_50: Rgb888 = Rgb888::new(245, 238, 255);
    // #904fe8
    pub const PURPLE_500: Rgb888 = Rgb888::new(158, 87, 255);
    // #703eb5
    pub const PURPLE_600: Rgb888 = Rgb888::new(144, 79, 232);
    // #57308c
    pub const PURPLE_700: Rgb888 = Rgb888::new(112, 62, 181);
    // #42256b
    pub const PURPLE_800: Rgb888 = Rgb888::new(87, 48, 140);
    // #eaf4f3

    // Purple
    pub const PURPLE_900: Rgb888 = Rgb888::new(66, 37, 107);
    // #ffa1a1
    pub const RED_100: Rgb888 = Rgb888::new(255, 192, 192);
    // #ff7676
    pub const RED_200: Rgb888 = Rgb888::new(255, 161, 161);
    // #ff5c5c
    pub const RED_300: Rgb888 = Rgb888::new(255, 118, 118);
    // #ff3333
    pub const RED_400: Rgb888 = Rgb888::new(255, 92, 92);
    // #ffc0c0
    pub const RED_50: Rgb888 = Rgb888::new(255, 235, 235);
    // #e82e2e
    pub const RED_500: Rgb888 = Rgb888::new(255, 51, 51);
    // #b52424
    pub const RED_600: Rgb888 = Rgb888::new(232, 46, 46);
    // #8c1c1c
    pub const RED_700: Rgb888 = Rgb888::new(181, 36, 36);
    // #6b1515
    pub const RED_800: Rgb888 = Rgb888::new(140, 28, 28);
    // #f5eeff

    // Red
    pub const RED_900: Rgb888 = Rgb888::new(107, 21, 21);
    // #8ad6dc
    pub const TEAL_100: Rgb888 = Rgb888::new(176, 227, 231);
    // #54c3cb
    pub const TEAL_200: Rgb888 = Rgb888::new(138, 214, 220);
    // #33b7c1
    pub const TEAL_300: Rgb888 = Rgb888::new(84, 195, 203);
    // #00a5b2
    pub const TEAL_400: Rgb888 = Rgb888::new(51, 183, 193);
    // #b0e3e7
    pub const TEAL_50: Rgb888 = Rgb888::new(230, 246, 247);
    // #0096a2
    pub const TEAL_500: Rgb888 = Rgb888::new(0, 165, 178);
    // #00757e
    pub const TEAL_600: Rgb888 = Rgb888::new(0, 150, 162);
    // #005b62
    pub const TEAL_700: Rgb888 = Rgb888::new(0, 117, 126);
    // #00454b
    pub const TEAL_800: Rgb888 = Rgb888::new(0, 91, 98);
    // #e6f5f8

    // Teal
    pub const TEAL_900: Rgb888 = Rgb888::new(0, 69, 75);
    // #00ffffff
    pub const TRANSPARENT_BLACK: Argb8888 = Argb8888::new(0, 0, 0, 0);
    // #000000

    pub const TRANSPARENT_WHITE: Argb8888 = Argb8888::new(0, 255, 255, 255); // #ffebeb
}

pub struct DarkPalette;

impl DarkPalette {
    pub const BACKGROUND_BRAND: Rgb888 = UIColors::BLUE_400;
    pub const BACKGROUND_BRAND_HOVER: Rgb888 = UIColors::BLUE_300;
    pub const BACKGROUND_BRAND_PRESSED: Rgb888 = UIColors::BLUE_200;
    pub const BACKGROUND_DISABLED: Rgb888 = UIColors::NEUTRAL_900;
    pub const BACKGROUND_HOVER: Rgb888 = UIColors::NEUTRAL_900;
    pub const BACKGROUND_INFO: Rgb888 = UIColors::BLUE_400;
    pub const BACKGROUND_INFO_SUBTLE: Rgb888 = UIColors::BLUE_800;
    pub const BACKGROUND_INVERSE: Rgb888 = UIColors::PRIMARY_WHITE;
    pub const BACKGROUND_NEGATIVE: Rgb888 = UIColors::RED_500;
    pub const BACKGROUND_NEGATIVE_SUBTLE: Rgb888 = UIColors::RED_900;
    pub const BACKGROUND_NOTICE: Rgb888 = UIColors::LIGHT_COPPER_500;
    pub const BACKGROUND_NOTICE_SUBTLE: Rgb888 = UIColors::LIGHT_COPPER_800;
    pub const BACKGROUND_POSITIVE: Rgb888 = UIColors::GREEN_500;
    pub const BACKGROUND_POSITIVE_SUBTLE: Rgb888 = UIColors::GREEN_900;
    pub const BACKGROUND_PRESSED: Rgb888 = UIColors::NEUTRAL_800;
    // Background
    pub const BACKGROUND_PRIMARY: Rgb888 = UIColors::NEUTRAL_950;
    pub const BACKGROUND_SELECTED: Rgb888 = UIColors::BLUE_800;
    // Primitives
    pub const BLACK: Rgb888 = UIColors::PRIMARY_BLACK;
    pub const BORDER_BRAND: Rgb888 = UIColors::BLUE_400;
    pub const BORDER_DISABLED: Rgb888 = UIColors::NEUTRAL_800;
    pub const BORDER_FOCUS: Rgb888 = UIColors::BLUE_400;
    pub const BORDER_INFO: Rgb888 = UIColors::BLUE_400;
    pub const BORDER_INVERSE: Rgb888 = UIColors::NEUTRAL_950;
    pub const BORDER_NEGATIVE: Rgb888 = UIColors::RED_500;
    pub const BORDER_NOTICE: Rgb888 = UIColors::LIGHT_COPPER_500;
    pub const BORDER_POSITIVE: Rgb888 = UIColors::GREEN_500;
    // Border
    pub const BORDER_PRIMARY: Rgb888 = UIColors::NEUTRAL_500;
    pub const BORDER_SECONDARY: Rgb888 = UIColors::NEUTRAL_700;
    pub const BORDER_TERTIARY: Rgb888 = UIColors::NEUTRAL_900;
    pub const CONTENT_BRAND: Rgb888 = UIColors::BLUE_400;
    pub const CONTENT_DISABLED: Rgb888 = UIColors::NEUTRAL_800;
    pub const CONTENT_LINK: Rgb888 = UIColors::BLUE_400;
    pub const CONTENT_LINK_HOVER: Rgb888 = UIColors::BLUE_300;
    pub const CONTENT_LINK_PRESSED: Rgb888 = UIColors::BLUE_200;
    pub const CONTENT_NEGATIVE: Rgb888 = UIColors::RED_500;
    pub const CONTENT_NEGATIVE_BOLD: Rgb888 = UIColors::RED_300;
    pub const CONTENT_NOTICE: Rgb888 = UIColors::LIGHT_COPPER_500;
    pub const CONTENT_NOTICE_BOLD: Rgb888 = UIColors::LIGHT_COPPER_300;
    pub const CONTENT_POSITIVE: Rgb888 = UIColors::GREEN_500;
    pub const CONTENT_POSITIVE_BOLD: Rgb888 = UIColors::GREEN_300;
    // Content
    pub const CONTENT_PRIMARY: Rgb888 = UIColors::PRIMARY_WHITE;
    pub const CONTENT_PRIMARY_INVERSE: Rgb888 = UIColors::NEUTRAL_950;
    pub const CONTENT_SECONDARY: Rgb888 = UIColors::NEUTRAL_100;
    pub const CONTENT_SECONDARY_INVERSE: Rgb888 = UIColors::NEUTRAL_900;
    pub const CONTENT_TERTIARY: Rgb888 = UIColors::NEUTRAL_400;
    pub const CONTENT_TERTIARY_INVERSE: Rgb888 = UIColors::NEUTRAL_700;
    pub const WHITE: Rgb888 = UIColors::PRIMARY_WHITE;
}

pub struct LightPalette;

impl LightPalette {
    pub const BACKGROUND_BRAND: Rgb888 = UIColors::BLUE_500;
    pub const BACKGROUND_BRAND_HOVER: Rgb888 = UIColors::BLUE_600;
    pub const BACKGROUND_BRAND_PRESSED: Rgb888 = UIColors::BLUE_700;
    pub const BACKGROUND_DISABLED: Rgb888 = UIColors::NEUTRAL_50;
    pub const BACKGROUND_HOVER: Rgb888 = UIColors::NEUTRAL_50;
    pub const BACKGROUND_INFO: Rgb888 = UIColors::BLUE_500;
    pub const BACKGROUND_INFO_SUBTLE: Rgb888 = UIColors::BLUE_100;
    pub const BACKGROUND_INVERSE: Rgb888 = UIColors::NEUTRAL_950;
    pub const BACKGROUND_NEGATIVE: Rgb888 = UIColors::RED_500;
    pub const BACKGROUND_NEGATIVE_SUBTLE: Rgb888 = UIColors::RED_100;
    pub const BACKGROUND_NOTICE: Rgb888 = UIColors::LIGHT_COPPER_500;
    pub const BACKGROUND_NOTICE_SUBTLE: Rgb888 = UIColors::LIGHT_COPPER_100;
    pub const BACKGROUND_POSITIVE: Rgb888 = UIColors::GREEN_500;
    pub const BACKGROUND_POSITIVE_SUBTLE: Rgb888 = UIColors::GREEN_100;
    pub const BACKGROUND_PRESSED: Rgb888 = UIColors::NEUTRAL_100;
    // Background
    pub const BACKGROUND_PRIMARY: Rgb888 = UIColors::PRIMARY_WHITE;
    pub const BACKGROUND_SELECTED: Rgb888 = UIColors::BLUE_100;
    // Primitives
    pub const BLACK: Rgb888 = UIColors::PRIMARY_BLACK;
    pub const BORDER_BRAND: Rgb888 = UIColors::BLUE_500;
    pub const BORDER_DISABLED: Rgb888 = UIColors::NEUTRAL_200;
    pub const BORDER_FOCUS: Rgb888 = UIColors::BLUE_500;
    pub const BORDER_INFO: Rgb888 = UIColors::BLUE_500;
    pub const BORDER_INVERSE: Rgb888 = UIColors::PRIMARY_WHITE;
    pub const BORDER_NEGATIVE: Rgb888 = UIColors::RED_500;
    pub const BORDER_NOTICE: Rgb888 = UIColors::LIGHT_COPPER_500;
    pub const BORDER_POSITIVE: Rgb888 = UIColors::GREEN_500;
    // Border
    pub const BORDER_PRIMARY: Rgb888 = UIColors::NEUTRAL_800;
    pub const BORDER_SECONDARY: Rgb888 = UIColors::NEUTRAL_600;
    pub const BORDER_TERTIARY: Rgb888 = UIColors::NEUTRAL_200;
    pub const CONTENT_BRAND: Rgb888 = UIColors::BLUE_500;
    pub const CONTENT_DISABLED: Rgb888 = UIColors::NEUTRAL_300;
    pub const CONTENT_LINK: Rgb888 = UIColors::BLUE_500;
    pub const CONTENT_LINK_HOVER: Rgb888 = UIColors::BLUE_600;
    pub const CONTENT_LINK_PRESSED: Rgb888 = UIColors::BLUE_700;
    pub const CONTENT_NEGATIVE: Rgb888 = UIColors::RED_500;
    pub const CONTENT_NEGATIVE_BOLD: Rgb888 = UIColors::RED_700;
    pub const CONTENT_NOTICE: Rgb888 = UIColors::LIGHT_COPPER_500;
    pub const CONTENT_NOTICE_BOLD: Rgb888 = UIColors::LIGHT_COPPER_700;
    pub const CONTENT_POSITIVE: Rgb888 = UIColors::GREEN_500;
    pub const CONTENT_POSITIVE_BOLD: Rgb888 = UIColors::GREEN_700;
    // Content
    pub const CONTENT_PRIMARY: Rgb888 = UIColors::NEUTRAL_950;
    pub const CONTENT_PRIMARY_INVERSE: Rgb888 = UIColors::PRIMARY_WHITE;
    pub const CONTENT_SECONDARY: Rgb888 = UIColors::NEUTRAL_900;
    pub const CONTENT_SECONDARY_INVERSE: Rgb888 = UIColors::NEUTRAL_100;
    pub const CONTENT_TERTIARY: Rgb888 = UIColors::NEUTRAL_800;
    pub const CONTENT_TERTIARY_INVERSE: Rgb888 = UIColors::NEUTRAL_300;
    pub const WHITE: Rgb888 = UIColors::PRIMARY_WHITE;
}
