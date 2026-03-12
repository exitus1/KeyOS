// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

slint::include_modules!();

/* Slint Problems/TODO:

- There is no flow layout
- GridLayout crashes when creating Row and content dynamically
- Slint functions say they are returning () instead of the real type, even when all paths are covered with returns
- Text widgets do not get drawn if they don't fit in the view.  Should just be clipped.
- What is the difference between inheriting from a component vs. just using the same component as the root? (CrossLayout didn't work)
- How to make a component in a layout take its natural size and not scale without setting a fixed size?
    (Use alignment: stretch, then apply horizontal-stretch: 1, etc. to the components that need to expand)
- Layouts don't seem to respect padding - they should fit within the padding bounds
    (Don't set a width and the layout will respect the padding)
- Does Slint scroll the focused input into view if in a scroll view?
- How can I have an image that doesn't scale?
- Binding loop detection is too aggressive.  If I have a specified width or height, then I should be able to use the correspodning value internal to the component.
- Want to be able to convert between float/int and percent (can divide go percent to float, but not the other way it seems)
- Add support for angular gradients to get circular spinner gradient effects

*/

#[derive(Default)]
struct FakeContext {
    fs: (),
}
fn main() {
    let slintbook = SlintBook::new().unwrap();
    let cx = FakeContext::default();
    slint_keyos_platform::_internal_init_ui_utils!(Utils, slintbook);
    slint_keyos_platform::_internal_init_images!(Images, slintbook, cx);
    slintbook.run().unwrap();
}
