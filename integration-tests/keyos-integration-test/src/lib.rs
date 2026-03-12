// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint_keyos_platform::xous;

pub fn fail_shutdown() -> ! {
    xous::rsyscall(xous::SysCall::Shutdown(1)).unwrap();
    unreachable!()
}

pub fn pass() -> ! {
    xous::rsyscall(xous::SysCall::Shutdown(0)).unwrap();
    unreachable!()
}

#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {{
        eprintln!("[{}:{}] {}", file!(), line!(), format_args!($($arg)*));
        $crate::fail_shutdown();
    }}
}

#[macro_export]
macro_rules! assert {
    ($cond:expr) => {
        if !$cond {
            $crate::fail!("assertion failed: {}", stringify!($cond));
        }
    };
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            $crate::fail!("assertion failed: {}\n  {}", format!($($arg)*), stringify!($cond));
        }
    };
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;
        if left != right {
            $crate::fail!(
                "assertion failed\n  `{} == {}`\n   left: `{:?}`\n  right: `{:?}`",
                stringify!($left), stringify!($right), left, right
            );
        }
    }};
    ($left:expr, $right:expr, $($arg:tt)*) => {{
        let left = $left;
        let right = $right;
        if left != right {
            $crate::fail!(
                "assertion failed: {}\n  `{} == {}`\n   left: `{:?}`\n  right: `{:?}`",
                format!($($arg)*), stringify!($left), stringify!($right), left, right
            );
        }
    }};
}

#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;
        if left == right {
            $crate::fail!(
                "assertion failed\n  `{} != {}`\n   left: `{:?}`\n  right: `{:?}`",
                stringify!($left), stringify!($right), left, right
            );
        }
    }};
    ($left:expr, $right:expr, $($arg:tt)*) => {{
        let left = $left;
        let right = $right;
        if left == right {
            $crate::fail!(
                "assertion failed: {}\n  `{} != {}`\n   left: `{:?}`\n  right: `{:?}`",
                format!($($arg)*), stringify!($left), stringify!($right), left, right
            );
        }
    }};
}
