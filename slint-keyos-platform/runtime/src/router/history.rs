// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::any::Any;

use serde::{Deserialize, Serialize};

use super::RouteId;
use crate::route::RouteEntry;

#[derive(Serialize, Deserialize)]
pub struct NavHistory {
    max_size: usize,
    pub(crate) backward: Vec<HistoryEntry>,
    pub(crate) forward: Vec<HistoryEntry>,
}

impl std::fmt::Debug for NavHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavHistory")
            .field("backward", &self.backward.iter().map(|entry| &entry.path))
            .field("forward", &self.forward.iter().map(|entry| &entry.path))
            .finish()
    }
}

impl Default for NavHistory {
    fn default() -> Self { NavHistory { max_size: 50, backward: Vec::new(), forward: Vec::new() } }
}

impl NavHistory {
    /// true if we can navigate backward
    pub fn has_backward(&self) -> bool {
        // We want to have at least one entry in the history.
        self.backward.len() > 1
    }

    /// true if we can navigate forward
    pub fn has_forward(&self) -> bool { !self.forward.is_empty() }

    /// the current route entry
    pub fn get_current(&self) -> Option<&HistoryEntry> { self.backward.last() }

    /// the current path
    pub fn get_current_path(&self) -> Option<&str> { self.get_current().map(|entry| entry.path.as_str()) }

    /// the total number of entries in the history
    pub fn len(&self) -> usize { self.backward.len() + self.forward.len() }
}

impl NavHistory {
    /// returns Some(entry) if entry is already active.
    pub(crate) fn push(&mut self, entry: HistoryEntry) -> Option<HistoryEntry> {
        if let Some(last) = self.backward.last() {
            if last.path == entry.path {
                return Some(entry);
            }
        }

        // check if the maximum size is reached
        if self.backward.len() >= self.max_size {
            // remove the oldest entry from the backward vector
            self.backward.remove(0);
        }

        self.backward.push(entry);
        self.clear_forward();
        None
    }

    pub(crate) fn replace(&mut self, entry: HistoryEntry) -> bool {
        if let Some(last) = self.backward.last_mut() {
            *last = entry;
            true
        } else {
            false
        }
    }

    pub(crate) fn nav_backward(&mut self) -> bool {
        if self.has_backward() {
            let entry = self.backward.pop().unwrap();
            self.forward.push(entry);
            true
        } else {
            false
        }
    }

    pub(crate) fn nav_forward(&mut self) -> bool {
        if let Some(entry) = self.forward.pop() {
            self.backward.push(entry);
            true
        } else {
            false
        }
    }

    pub(crate) fn clear_forward(&mut self) { self.forward.clear(); }

    pub(crate) fn get_current_mut(&mut self) -> Option<&mut HistoryEntry> { self.backward.last_mut() }

    pub(crate) fn clear(&mut self) {
        self.backward.clear();
        self.forward.clear();
    }
}

#[derive(Serialize, Deserialize)]
pub struct HistoryEntry {
    pub(crate) route: RouteId,
    pub(crate) path: String,
    // Minor optimization: Store value to avoid de-serializing it every time
    // It is optional in the case we are de-serializing it for the first time
    // Can't deserialize on Any because we need to know the type to de-serialize
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) value: Option<Box<dyn CachedEntry>>,
}

impl std::fmt::Debug for HistoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryEntry")
            .field("route", &self.route)
            .field("path", &self.path)
            .field("value", if self.value.is_some() { &"Some" } else { &"None" })
            .finish()
    }
}

impl HistoryEntry {
    /// the current serialized route path
    pub fn path(&self) -> &str { &self.path }

    /// the current route id
    pub fn route(&self) -> &RouteId { &self.route }
}

impl HistoryEntry {
    pub(crate) fn new<T>(path: String, value: T) -> Self
    where
        T: RouteEntry + 'static,
    {
        Self { route: RouteId::new::<T>(), path, value: Some(Box::new(CachedEntryImpl { value })) }
    }

    pub(crate) fn new_raw(route: RouteId, path: String, data: Box<dyn CachedEntry>) -> Self {
        Self { route, path, value: Some(data) }
    }

    pub(crate) fn get_value<T>(&mut self) -> Option<&T>
    where
        T: RouteEntry + 'static,
    {
        if let Some(ref value) = self.value {
            value.as_any().downcast_ref::<T>()
        } else {
            let value = T::de_route(&self.path).ok()?;
            self.value = Some(Box::new(CachedEntryImpl { value }));
            self.value.as_ref().unwrap().as_any().downcast_ref::<T>()
        }
    }
}

pub(crate) trait CachedEntry {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub(crate) struct CachedEntryImpl<T> {
    pub(crate) value: T,
}

impl<T> CachedEntry for CachedEntryImpl<T>
where
    T: RouteEntry + 'static,
{
    fn as_any(&self) -> &dyn Any { &self.value }
}
