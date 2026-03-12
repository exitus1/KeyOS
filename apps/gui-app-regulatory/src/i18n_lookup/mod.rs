// SPDX-FileCopyrightText: 2023-2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

// AUTO-GENERATED FILE - DO NOT EDIT

use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::Lazy;

static LOCALE: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("en".to_string()));

pub fn set_locale(locale: &str) {
    let mut current_locale = LOCALE.write().unwrap();
    *current_locale = locale.to_string();
}

pub fn lookup(id: &str) -> String {
    let locale = LOCALE.read().unwrap();
    match locale.as_str() {
        "en" => EN_TRANSLATIONS.get(id).map(|&s| s.to_string()).unwrap_or_else(|| id.to_string()),
        "es" => ES_TRANSLATIONS.get(id).map(|&s| s.to_string()).unwrap_or_else(|| id.to_string()),
        _ => id.to_string(),
    }
}

static EN_TRANSLATIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("common.button.apply", "Apply");
    m.insert("common.button.back", "Back");
    m.insert("common.button.cancel", "Cancel");
    m.insert("common.button.confirm", "Confirm");
    m.insert("common.button.continue", "Continue");
    m.insert("common.button.create", "Create");
    m.insert("common.button.delete", "Delete");
    m.insert("common.button.dismiss", "Dismiss");
    m.insert("common.button.done", "Done");
    m.insert("common.button.dontShowAgain", "Don't show again");
    m.insert("common.button.getStarted", "Get Started");
    m.insert("common.button.ok", "OK");
    m.insert("common.button.recover", "Recover");
    m.insert("common.button.retry", "Retry");
    m.insert("common.button.return", "Back");
    m.insert("common.button.save", "Save");
    m.insert("common.button.saveToFile", "Save to File");
    m.insert("common.button.shutDown", "Shut Down");
    m.insert("common.button.skip", "Skip");
    m.insert("common.button.startKeyos", "Start KeyOS");
    m.insert("common.button.verify", "Verify");
    m.insert("common.button.verifyAddress", "Verify Address");
    m.insert("common.button.verifying", "Verifying...");
    m.insert("common.external", "External");
    m.insert("common.fillerText.search", "Search...");
    m.insert("common.month.aprilFull", "April");
    m.insert("common.month.aprilShort", "Apr");
    m.insert("common.month.augustFull", "August");
    m.insert("common.month.augustShort", "Aug");
    m.insert("common.month.decemberFull", "December");
    m.insert("common.month.decemberShort", "Dec");
    m.insert("common.month.februaryFull", "February");
    m.insert("common.month.februaryShort", "Feb");
    m.insert("common.month.januaryFull", "January");
    m.insert("common.month.januaryShort", "Jan");
    m.insert("common.month.julyFull", "July");
    m.insert("common.month.julyShort", "Jul");
    m.insert("common.month.juneFull", "June");
    m.insert("common.month.juneShort", "Jun");
    m.insert("common.month.marchFull", "March");
    m.insert("common.month.marchShort", "Mar");
    m.insert("common.month.mayFull", "May");
    m.insert("common.month.mayShort", "May");
    m.insert("common.month.novemberFull", "November");
    m.insert("common.month.novemberShort", "Nov");
    m.insert("common.month.octoberFull", "October");
    m.insert("common.month.octoberShort", "Oct");
    m.insert("common.month.septemberFull", "September");
    m.insert("common.month.septemberShort", "Sep");
    m.insert("common.slideToSign.helper", "Slide to sign");
    m.insert("common.slideToSign.sign", "Sign");
    m.insert("common.time.hourFull", "hour");
    m.insert("common.time.hourMed", "hr");
    m.insert("common.time.hourShort", "h");
    m.insert("common.time.hoursFull", "hours");
    m.insert("common.time.hoursMed", "hrs");
    m.insert("common.time.minuteFull", "minute");
    m.insert("common.time.minuteMed", "min");
    m.insert("common.time.minuteShort", "m");
    m.insert("common.time.minutesFull", "minutes");
    m.insert("common.time.minutesMed", "mins");
    m.insert("common.time.secondFull", "second");
    m.insert("common.time.secondMed", "sec");
    m.insert("common.time.secondShort", "s");
    m.insert("common.time.secondsFull", "seconds");
    m.insert("common.time.secondsMed", "secs");
    m.insert("common.weekday.fridayFull", "Friday");
    m.insert("common.weekday.fridayShort", "Fri");
    m.insert("common.weekday.mondayFull", "Monday");
    m.insert("common.weekday.mondayShort", "Mon");
    m.insert("common.weekday.saturdayFull", "Saturday");
    m.insert("common.weekday.saturdayShort", "Sat");
    m.insert("common.weekday.sundayFull", "Sunday");
    m.insert("common.weekday.sundayShort", "Sun");
    m.insert("common.weekday.thursdayFull", "Thursday");
    m.insert("common.weekday.thursdayShort", "Thu");
    m.insert("common.weekday.tuesdayFull", "Tuesday");
    m.insert("common.weekday.tuesdayShort", "Tue");
    m.insert("common.weekday.wednesdayFull", "Wednesday");
    m.insert("common.weekday.wednesdayShort", "Wed");
    m
});

static ES_TRANSLATIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("common.button.apply", "Apply");
    m.insert("common.button.back", "Back");
    m.insert("common.button.cancel", "Cancel");
    m.insert("common.button.confirm", "Confirm");
    m.insert("common.button.continue", "Continue");
    m.insert("common.button.create", "Create");
    m.insert("common.button.delete", "Delete");
    m.insert("common.button.dismiss", "Dismiss");
    m.insert("common.button.done", "Done");
    m.insert("common.button.dontShowAgain", "Don't show again");
    m.insert("common.button.getStarted", "Get Started");
    m.insert("common.button.ok", "OK");
    m.insert("common.button.recover", "Recover");
    m.insert("common.button.retry", "Retry");
    m.insert("common.button.return", "Back");
    m.insert("common.button.save", "Save");
    m.insert("common.button.saveToFile", "Save to File");
    m.insert("common.button.shutDown", "Shut Down");
    m.insert("common.button.skip", "Skip");
    m.insert("common.button.startKeyos", "Start KeyOS");
    m.insert("common.button.verify", "Verify");
    m.insert("common.button.verifyAddress", "Verify Address");
    m.insert("common.button.verifying", "Verifying...");
    m.insert("common.external", "External");
    m.insert("common.fillerText.search", "Search...");
    m.insert("common.month.aprilFull", "April");
    m.insert("common.month.aprilShort", "Apr");
    m.insert("common.month.augustFull", "August");
    m.insert("common.month.augustShort", "Aug");
    m.insert("common.month.decemberFull", "December");
    m.insert("common.month.decemberShort", "Dec");
    m.insert("common.month.februaryFull", "February");
    m.insert("common.month.februaryShort", "Feb");
    m.insert("common.month.januaryFull", "January");
    m.insert("common.month.januaryShort", "Jan");
    m.insert("common.month.julyFull", "July");
    m.insert("common.month.julyShort", "Jul");
    m.insert("common.month.juneFull", "June");
    m.insert("common.month.juneShort", "Jun");
    m.insert("common.month.marchFull", "March");
    m.insert("common.month.marchShort", "Mar");
    m.insert("common.month.mayFull", "May");
    m.insert("common.month.mayShort", "May");
    m.insert("common.month.novemberFull", "November");
    m.insert("common.month.novemberShort", "Nov");
    m.insert("common.month.octoberFull", "October");
    m.insert("common.month.octoberShort", "Oct");
    m.insert("common.month.septemberFull", "September");
    m.insert("common.month.septemberShort", "Sep");
    m.insert("common.slideToSign.helper", "Slide to sign");
    m.insert("common.slideToSign.sign", "Sign");
    m.insert("common.time.hourFull", "hour");
    m.insert("common.time.hourMed", "hr");
    m.insert("common.time.hourShort", "h");
    m.insert("common.time.hoursFull", "hours");
    m.insert("common.time.hoursMed", "hrs");
    m.insert("common.time.minuteFull", "minute");
    m.insert("common.time.minuteMed", "min");
    m.insert("common.time.minuteShort", "m");
    m.insert("common.time.minutesFull", "minutes");
    m.insert("common.time.minutesMed", "mins");
    m.insert("common.time.secondFull", "second");
    m.insert("common.time.secondMed", "sec");
    m.insert("common.time.secondShort", "s");
    m.insert("common.time.secondsFull", "seconds");
    m.insert("common.time.secondsMed", "secs");
    m.insert("common.weekday.fridayFull", "Friday");
    m.insert("common.weekday.fridayShort", "Fri");
    m.insert("common.weekday.mondayFull", "Monday");
    m.insert("common.weekday.mondayShort", "Mon");
    m.insert("common.weekday.saturdayFull", "Saturday");
    m.insert("common.weekday.saturdayShort", "Sat");
    m.insert("common.weekday.sundayFull", "Sunday");
    m.insert("common.weekday.sundayShort", "Sun");
    m.insert("common.weekday.thursdayFull", "Thursday");
    m.insert("common.weekday.thursdayShort", "Thu");
    m.insert("common.weekday.tuesdayFull", "Tuesday");
    m.insert("common.weekday.tuesdayShort", "Tue");
    m.insert("common.weekday.wednesdayFull", "Wednesday");
    m.insert("common.weekday.wednesdayShort", "Wed");
    m
});
