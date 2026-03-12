#[cfg(not(keyos))]
pub mod hosted;
#[cfg(not(keyos))]
pub use hosted::*;

#[cfg(keyos)]
#[macro_use]
pub mod atsama5d2;
#[cfg(keyos)]
pub use atsama5d2::*;
