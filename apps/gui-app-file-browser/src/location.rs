// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use slint_keyos_platform::gui_server_api::navigation::filepicker::Location;

use crate::TrId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationKey {
    External,
    #[cfg(not(feature = "recovery-os"))]
    Internal,
    #[cfg(not(feature = "recovery-os"))]
    Airlock,
}

impl LocationKey {
    #[cfg(not(feature = "recovery-os"))]
    pub const ALL: [LocationKey; 3] = [LocationKey::Internal, LocationKey::Airlock, LocationKey::External];
    #[cfg(feature = "recovery-os")]
    pub const ALL: [LocationKey; 1] = [LocationKey::External];
    #[cfg(not(feature = "recovery-os"))]
    pub const DEFAULT: LocationKey = LocationKey::Internal;
    #[cfg(feature = "recovery-os")]
    pub const DEFAULT: LocationKey = LocationKey::External;

    pub fn from_nav(location: Location) -> Self {
        #[cfg(feature = "recovery-os")]
        {
            let _ = location;
            LocationKey::External
        }
        #[cfg(not(feature = "recovery-os"))]
        match location {
            Location::External => LocationKey::External,
            Location::Internal => LocationKey::Internal,
            Location::Airlock => LocationKey::Airlock,
        }
    }

    pub fn to_nav(self) -> Location {
        match self {
            LocationKey::External => Location::External,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Internal => Location::Internal,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Airlock => Location::Airlock,
        }
    }

    pub fn from_fs(location: fs::Location) -> Option<Self> {
        match location {
            fs::Location::Usb => Some(LocationKey::External),
            #[cfg(not(feature = "recovery-os"))]
            fs::Location::User => Some(LocationKey::Internal),
            #[cfg(not(feature = "recovery-os"))]
            fs::Location::Airlock => Some(LocationKey::Airlock),
            _ => None,
        }
    }

    pub fn to_fs(self) -> fs::Location {
        match self {
            LocationKey::External => fs::Location::Usb,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Internal => fs::Location::User,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Airlock => fs::Location::Airlock,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Internal => "prime",
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Airlock => "airlock",
            LocationKey::External => "usb",
        }
    }

    pub fn tr_id(self) -> TrId {
        match self {
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Internal => TrId::MainInternal,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Airlock => TrId::MainAirlock,
            LocationKey::External => TrId::MainExternal,
        }
    }
}

#[derive(Clone)]
pub struct LocationMap<T> {
    external: T,
    #[cfg(not(feature = "recovery-os"))]
    internal: T,
    #[cfg(not(feature = "recovery-os"))]
    airlock: T,
}

impl<T> LocationMap<T> {
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(LocationKey) -> T,
    {
        LocationMap {
            external: f(LocationKey::External),
            #[cfg(not(feature = "recovery-os"))]
            internal: f(LocationKey::Internal),
            #[cfg(not(feature = "recovery-os"))]
            airlock: f(LocationKey::Airlock),
        }
    }

    pub fn get(&self, key: LocationKey) -> &T {
        match key {
            LocationKey::External => &self.external,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Internal => &self.internal,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Airlock => &self.airlock,
        }
    }

    pub fn get_mut(&mut self, key: LocationKey) -> &mut T {
        match key {
            LocationKey::External => &mut self.external,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Internal => &mut self.internal,
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Airlock => &mut self.airlock,
        }
    }

    pub fn insert(&mut self, key: LocationKey, value: T) -> &mut T {
        match key {
            LocationKey::External => {
                self.external = value;
                &mut self.external
            }
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Internal => {
                self.internal = value;
                &mut self.internal
            }
            #[cfg(not(feature = "recovery-os"))]
            LocationKey::Airlock => {
                self.airlock = value;
                &mut self.airlock
            }
        }
    }
}
