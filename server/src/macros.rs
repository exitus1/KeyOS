// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[macro_export]
macro_rules! wrapped_scalar {
    ($name:ty) => {
        impl $crate::FromScalar<4> for $name {
            fn from_scalar(value: [u32; 4]) -> Self { Self($crate::FromScalar::from_scalar(value)) }
        }

        impl $crate::AsScalar<4> for $name {
            fn as_scalar(&self) -> [u32; 4] { self.0.as_scalar() }
        }
    };
}
