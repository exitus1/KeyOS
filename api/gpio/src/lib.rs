// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg(keyos)]

pub mod messages;
mod pins;

pub use messages::IrqMessage;
pub use pins::{GpioPin, PinSettings};
use server::{CheckedConn, CheckedPermissions, MessageAllowed, Server, ServerContext};
use {
    crate::messages::*,
    num_derive::{FromPrimitive, ToPrimitive},
    num_traits::{FromPrimitive, ToPrimitive},
};

/// GPIO server API handle. Holds the connection to the GPIO server obtained through
/// `xous-names`.
///
/// # Example (interrupt-based button handling with debouncing)
/// ```rust
/// # pub fn main(context: &mut ServerContext) -> Result<(), gpio::GpioError> {
/// use {
///     gpio::{GpioApi, GpioPin, PinSettings},
///     server::xous,
/// };
///
/// let api = GpioApi::new();
/// api.claim_pin(
///     GpioPin::PowerButton,
///     PinSettings::InterruptBoth,
///     true, /* enable debounce */
/// )?;
///
/// api.enable_irq(GpioPin::PowerButton, context)?;
/// # Ok(())
/// # }
/// ```
///
/// # Example (driving GPIO pin to reset a peripheral)
///
/// ```rust
/// # pub fn main() -> Result<(), gpio::GpioError> {
/// use {
///     gpio::{GpioApi, GpioPin, PinSettings},
///     std::{thread, time::Duration},
/// };
///
/// let api = GpioApi::new();
/// api.claim_pin(GpioPin::CtpRstB, PinSettings::OutputHigh, false)?;
///
/// // Do a reset pulse
/// api.set_pin(GpioPin::CtpRstB, false)?;
/// thread::sleep(Duration::from_millis(10));
/// api.set_pin(GpioPin::CtpRstB, true)?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct GpioApi<P: CheckedPermissions> {
    conn: CheckedConn<P>,
}

#[macro_export]
macro_rules! use_api {
    () => {
        mod gpio_permissions {
            use gpio::messages::*;
            #[derive(Clone, Default, server::Permissions)]
            #[server_name = "os/gpio"]
            pub struct GpioPermissions;
        }
        type GpioApi = gpio::GpioApi<gpio_permissions::GpioPermissions>;
    };
}

impl<P: CheckedPermissions> GpioApi<P> {
    /// Takes exclusive ownership of the hardware GPIO pin and configures it.
    /// See [`PinSettings`] for the supported ways to configure the GPIO pin.
    ///
    /// If the `debounce` is `true`, then a hardware debounce filter is enabled (used for
    /// hardware switches such as buttons).
    ///
    /// # Errors
    /// - [`GpioError::AlreadyClaimed`], the pin has been already claimed by the same or some other process
    ///   and can't be reclaimed.
    /// - [`GpioError::InternalError`], an unknown error occurred
    pub fn claim_pin(&self, pin: GpioPin, settings: PinSettings, debounce: bool) -> Result<(), GpioApiError>
    where
        P: MessageAllowed<ClaimPin>,
    {
        self.conn.try_send_blocking_scalar(ClaimPin::new(pin, settings, debounce))?
    }

    /// Enables IRQs for the pin.
    /// The pin must be claimed and configured as an IRQ source and the IRQ handler must
    /// be registered.
    ///
    /// # Errors
    /// - [`GpioError::PinNotClaimed`] the pin is not claimed, use [`claim_pin()`] first.
    /// - [`GpioError::AccessDenied`] the pin is already claimed by some other process.
    /// - [`GpioError::PinNotConfiguredAsIrq`] the pin is not configured as an IRQ source. Check the
    ///   parameters of [`claim_pin()`].
    /// - [`GpioError::IrqHandlerNotRegistered`] the [`register_irq_handler`] must be called first before
    ///   enabling IRQs.
    pub fn enable_irq<S>(&self, pin: GpioPin, context: &mut ServerContext<S>) -> Result<(), GpioApiError>
    where
        S: Server + server::ScalarEventHandler<IrqMessage>,
        P: MessageAllowed<EnableIrq>,
    {
        self.conn.subscribe_scalar(EnableIrq(pin), context)
    }

    /// Sets the pin's IRQs active or not.
    /// The pin must be claimed and configured as an IRQ source.
    ///
    /// # Errors
    /// - [`GpioError::PinNotClaimed`] the pin is not claimed, use [`claim_pin()`] first.
    /// - [`GpioError::AccessDenied`] the pin is already claimed by some other process.
    /// - [`GpioError::PinNotConfiguredAsIrq`] the pin is not configured as an IRQ source. Check the
    ///   parameters of [`claim_pin()`].
    pub fn set_irq(&self, pin: GpioPin, is_active: bool) -> Result<(), GpioApiError>
    where
        P: MessageAllowed<SetIrq>,
    {
        self.conn.try_send_blocking_scalar(SetIrq(pin, is_active))?
    }

    /// Sets the pin's digital output state as `HIGH` or `LOW`.
    /// The pin must be claimed and configured as a digital output.
    ///
    /// # Errors
    /// - [`GpioError::PinNotClaimed`] the pin is not claimed, use [`claim_pin()`] first.
    /// - [`GpioError::AccessDenied`] the pin is already claimed by some other process.
    /// - [`GpioError::PinNotConfiguredAsOutput`] the pin is not configured as a digital output. Check the
    ///   parameters of [`claim_pin()`].
    pub fn set_pin(&self, pin: GpioPin, is_high: bool) -> Result<(), GpioApiError>
    where
        P: MessageAllowed<SetPin>,
    {
        self.conn.try_send_blocking_scalar(SetPin(pin, is_high))?
    }

    /// Gets the pin's digital output state as `HIGH` or `LOW`.
    /// The pin must be claimed.
    ///
    /// # Errors
    /// - [`GpioError::PinNotClaimed`] the pin is not claimed, use [`claim_pin()`] first.
    /// - [`GpioError::AccessDenied`] the pin is already claimed by some other process.
    pub fn get_pin(&self, pin: GpioPin) -> Result<bool, GpioApiError>
    where
        P: MessageAllowed<GetPin>,
    {
        self.conn.try_send_blocking_scalar(GetPin(pin))?
    }
}

#[derive(
    Debug, Copy, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, FromPrimitive, ToPrimitive,
)]
pub enum GpioApiError {
    AlreadyClaimed = 1,
    PinNotClaimed,
    AccessDenied,
    PinNotConfiguredAsIrq,
    PinNotConfiguredAsOutput,
    IrqHandlerNotRegistered,
    InternalError,
}

impl server::AsScalar<1> for GpioApiError {
    fn as_scalar(&self) -> [u32; 1] { [self.to_u32().unwrap()] }
}

impl server::FromScalar<1> for GpioApiError {
    fn from_scalar([value]: [u32; 1]) -> Self { Self::from_u32(value).unwrap_or(Self::InternalError) }
}

impl TryFrom<u32> for GpioApiError {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> { GpioApiError::from_u32(value).ok_or(()) }
}

impl From<xous::Error> for GpioApiError {
    fn from(_value: xous::Error) -> Self { GpioApiError::InternalError }
}

impl eh_1::digital::Error for GpioApiError {
    fn kind(&self) -> eh_1::digital::ErrorKind { eh_1::digital::ErrorKind::Other }
}

pub struct HalPin<P: CheckedPermissions> {
    api: GpioApi<P>,
    pin: GpioPin,
}

impl GpioPin {
    pub fn into_hal_pin<P: CheckedPermissions>(self, api: &GpioApi<P>) -> HalPin<P> {
        HalPin { api: GpioApi { conn: api.conn.clone() }, pin: self }
    }
}

impl<P> eh_0::digital::v2::OutputPin for HalPin<P>
where
    P: CheckedPermissions,
    P: MessageAllowed<SetPin>,
    P: MessageAllowed<GetPin>,
{
    type Error = GpioApiError;

    fn set_low(&mut self) -> Result<(), Self::Error> { self.api.set_pin(self.pin, false) }

    fn set_high(&mut self) -> Result<(), Self::Error> { self.api.set_pin(self.pin, true) }
}

impl<P: CheckedPermissions> eh_1::digital::ErrorType for HalPin<P> {
    type Error = GpioApiError;
}

impl<P> eh_1::digital::OutputPin for HalPin<P>
where
    P: CheckedPermissions,
    P: MessageAllowed<SetPin>,
    P: MessageAllowed<GetPin>,
{
    fn set_low(&mut self) -> Result<(), Self::Error> { self.api.set_pin(self.pin, false) }

    fn set_high(&mut self) -> Result<(), Self::Error> { self.api.set_pin(self.pin, true) }
}

impl<P> eh_1::digital::InputPin for HalPin<P>
where
    P: CheckedPermissions,
    P: MessageAllowed<SetPin>,
    P: MessageAllowed<GetPin>,
{
    fn is_low(&mut self) -> Result<bool, Self::Error> { self.api.get_pin(self.pin).map(|b| !b) }

    fn is_high(&mut self) -> Result<bool, Self::Error> { self.api.get_pin(self.pin) }
}
