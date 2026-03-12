// Feature flags are checked inside build.rs for sanity, so we
// can use them here assuming all targets are mutually exclusive.

#[cfg(feature = "atsama5d27")]
mod atsama5d27;
#[cfg(feature = "atsama5d27")]
pub use atsama5d27::*;

// Hosted mode includes nothing, as it relies on the abstract host
// architecture for I/O; so this file is empty when the "hosted"
// configuration is selected and there are no corresponding "hosted"
// mode feature flags.
