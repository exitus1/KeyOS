// SPDX-FileCopyrightText: 2020 Sean Cross <sean@xobs.io>
// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: Apache-2.0

/// Prints to the debug output directly.
#[cfg(all(keyos, any(not(feature = "production"), feature = "log-serial")))]
#[macro_export]
macro_rules! print {
    ($($args:tt)+) => {{
		use core::fmt::Write;
		#[allow(unused_unsafe)]
        $crate::debug::serial::with_output(|stream|
			write!(stream, $($args)+).unwrap()
        )
    }};
}

#[cfg(all(keyos, feature = "production", not(feature = "log-serial")))]
#[macro_export]
macro_rules! print {
    ($($_args:tt)+) => {{}};
}

/// Prints to the debug output directly, with a newline.
#[cfg(keyos)]
#[macro_export]
macro_rules! println {
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!("{}\r\n", format_args!($fmt))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!("{}\r\n", format_args!($fmt, $($args)+))
	});
}

#[cfg(feature = "debug-print")]
#[macro_export]
macro_rules! klog {
	() => ({
		println!(" [{}:{}]", file!(), line!())
	});
	($fmt:expr) => ({
        println!(" [{}:{} {}]", file!(), line!(), format_args!($fmt))
	});
	($fmt:expr, $($args:tt)+) => ({
		println!(" [{}:{} {}]", file!(), line!(), format_args!($fmt, $($args)+))
	});
}

#[cfg(not(feature = "debug-print"))]
#[macro_export]
macro_rules! klog {
    ($($args:tt)+) => {{}};
}
