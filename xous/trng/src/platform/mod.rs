#[cfg(not(keyos))]
mod hosted;

#[cfg(not(keyos))]
pub use hosted::*;

#[cfg(keyos)]
mod atsama5d2;

#[cfg(keyos)]
pub use atsama5d2::*;
