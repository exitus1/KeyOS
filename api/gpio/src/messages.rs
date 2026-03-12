// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use {
    num_traits::FromPrimitive,
    server::{AsScalar, FromScalar},
};

pub use crate::{
    pins::{GpioPin, PinSettings},
    GpioApi, GpioApiError, HalPin,
};

#[derive(Debug, server::Message)]
#[response(Result<(), GpioApiError>)]
pub struct ClaimPin {
    pub pin: GpioPin,
    pub pin_settings: PinSettings,
    pub debounce: bool,
}

impl ClaimPin {
    pub fn new(pin: GpioPin, pin_settings: PinSettings, debounce: bool) -> Self {
        Self { pin, pin_settings, debounce }
    }
}

impl FromScalar<3> for ClaimPin {
    fn from_scalar(value: [u32; 3]) -> Self { value.try_into().expect("can't convert scalar to GpioPin") }
}

impl AsScalar<3> for ClaimPin {
    fn as_scalar(&self) -> [u32; 3] { [self.pin as u32, self.pin_settings as u32, self.debounce as u32] }
}

impl TryFrom<[u32; 3]> for ClaimPin {
    type Error = ();

    fn try_from(value: [u32; 3]) -> Result<Self, ()> {
        let pin = GpioPin::from_u32(value[0]).ok_or(())?;
        let pin_settings = PinSettings::from_u32(value[1]).ok_or(())?;
        let debounce = value[2] != 0;
        Ok(Self::new(pin, pin_settings, debounce))
    }
}

#[derive(Debug, server::Message, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[event(IrqMessage)]
#[error(GpioApiError)]
pub struct EnableIrq(pub GpioPin);

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct IrqMessage {
    pub pin: GpioPin,
    pub is_high: bool,
}

impl FromScalar<2> for IrqMessage {
    fn from_scalar([a, b]: [u32; 2]) -> Self {
        Self { pin: GpioPin::from_u32(a).expect("decode"), is_high: b != 0 }
    }
}

impl AsScalar<2> for IrqMessage {
    fn as_scalar(&self) -> [u32; 2] { [self.pin as u32, self.is_high as u32] }
}

#[derive(Debug, server::Message)]
#[response(Result<(), GpioApiError>)]
pub struct SetIrq(pub GpioPin, pub bool);

impl AsScalar<2> for SetIrq {
    fn as_scalar(&self) -> [u32; 2] { [self.0 as u32, self.1 as u32] }
}

impl FromScalar<2> for SetIrq {
    fn from_scalar([a, b]: [u32; 2]) -> Self { Self(GpioPin::from_u32(a).expect("decode"), b != 0) }
}

#[derive(Debug, server::Message)]
#[response(Result<(), GpioApiError>)]
pub struct SetPin(pub GpioPin, pub bool);

impl AsScalar<2> for SetPin {
    fn as_scalar(&self) -> [u32; 2] { [self.0 as u32, self.1 as u32] }
}

impl FromScalar<2> for SetPin {
    fn from_scalar([a, b]: [u32; 2]) -> Self { Self(GpioPin::from_u32(a).expect("decode"), b != 0) }
}

#[derive(Debug, server::Message)]
#[response(Result<bool, GpioApiError>)]
pub struct GetPin(pub GpioPin);

impl AsScalar<1> for GpioPin {
    fn as_scalar(&self) -> [u32; 1] { [*self as u32] }
}

impl FromScalar<1> for GpioPin {
    fn from_scalar([a]: [u32; 1]) -> Self { Self::from_u32(a).expect("decode") }
}
