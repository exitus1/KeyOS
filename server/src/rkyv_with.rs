// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Custom rkyv serialization helpers for KeyOS

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rkyv::{
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Archived, Place, Serialize,
};

/// A custom UnixTimestamp implementation that doesn't have an error.
/// making it compatible with infallible error types
pub struct WithUnixTimestamp;

impl ArchiveWith<SystemTime> for WithUnixTimestamp {
    type Archived = Archived<Duration>;
    type Resolver = <Duration as Archive>::Resolver;

    #[inline]
    fn resolve_with(field: &SystemTime, resolver: Self::Resolver, out: Place<Self::Archived>) {
        let duration = field.duration_since(UNIX_EPOCH).unwrap_or_default();
        Archive::resolve(&duration, resolver, out);
    }
}

impl<S> SerializeWith<SystemTime, S> for WithUnixTimestamp
where
    S: rkyv::rancor::Fallible + ?Sized,
{
    fn serialize_with(field: &SystemTime, s: &mut S) -> Result<Self::Resolver, S::Error> {
        let duration = field.duration_since(UNIX_EPOCH).unwrap_or_default();
        duration.serialize(s)
    }
}

impl<D> DeserializeWith<Archived<Duration>, SystemTime, D> for WithUnixTimestamp
where
    D: rkyv::rancor::Fallible + ?Sized,
{
    fn deserialize_with(field: &Archived<Duration>, _: &mut D) -> Result<SystemTime, D::Error> {
        Ok(UNIX_EPOCH + Duration::from(*field))
    }
}
