// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;

use slint_keyos_platform::{
    async_archive,
    settings::{global::TimeZone as SettingTimeZone, messages::ListTimeZone},
    slint::{Model, ModelNotify, ModelTracker, SharedString},
};

use crate::{settings_permissions::SettingsPermissions, TimeZone};

pub struct TimeZoneModel {
    cache: RefCell<TimeZoneCache>,
    notify: ModelNotify,
}

#[derive(Clone)]
struct TimeZoneEntry {
    /// search optimized string
    search: String,
    slint_tz: TimeZone,
}

impl crate::TimeZone {
    pub fn new(timezone: SettingTimeZone, timestamp: jiff::Timestamp) -> Self {
        let tz = timezone.timezone();
        let info = tz.to_offset_info(timestamp);
        let offset = info.offset().seconds();

        let hours = offset / 3600;
        let minutes = (offset.abs() % 3600) / 60;
        let offset_str = format!("{:+03}:{:02}", hours, minutes);

        let clean_name = timezone.name().replace("_", " ");

        Self {
            display_full: SharedString::from(format!("{clean_name} ({offset_str})")),
            display_short: SharedString::from(clean_name),
            id: SharedString::from(timezone.name()),
        }
    }
}

struct TimeZoneCache {
    all_timezones: Vec<TimeZoneEntry>,
    filtered_indices: Vec<usize>,
}

impl TimeZoneModel {
    pub fn new() -> Self {
        let now = jiff::Timestamp::now();

        let all_timezones: Vec<TimeZoneEntry> = async_archive::<SettingsPermissions, _>(ListTimeZone { offset: None, count: None })
                // using async for "move" messages with unlimited buffer size
                .block_on()
                .into_iter()
                .map(|setting_tz| TimeZoneEntry {
                    search: setting_tz.name().to_lowercase().replace("_", " ").replace("/", " "),
                    slint_tz: TimeZone::new(setting_tz, now)})
                .collect();

        let filtered_indices: Vec<usize> = (0..all_timezones.len()).collect();

        Self {
            cache: RefCell::new(TimeZoneCache { all_timezones, filtered_indices }),
            notify: Default::default(),
        }
    }

    pub fn set_search(&self, search: &str) {
        let mut cache = self.cache.borrow_mut();
        let search = search.to_lowercase();
        cache.filtered_indices = cache
            .all_timezones
            .iter()
            .enumerate()
            .filter(|(_, tz)| tz.search.contains(&search))
            .map(|(i, _)| i)
            .collect();

        self.notify.reset();
    }
}

impl Model for TimeZoneModel {
    type Data = TimeZone;

    fn row_count(&self) -> usize { self.cache.borrow().filtered_indices.len() }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let cache = self.cache.borrow();

        if row >= cache.filtered_indices.len() {
            return None;
        }

        let index = cache.filtered_indices[row];
        Some(cache.all_timezones[index].slint_tz.clone())
    }

    fn model_tracker(&self) -> &dyn ModelTracker { &self.notify }
}
