// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::OnceLock;

use log::Level;
use regex::Regex;

use crate::state::{
    log::{LogItem, LogMessage, PanicMessage},
    process::{ProcessListSnapshot, ProcessRow},
};

pub enum ParseItem {
    Log(LogItem),
    ProcessSnapshot(ProcessListSnapshot),
}

pub fn parse_payload(payload: &str, existing_entries: &[LogItem]) -> Option<ParseItem> {
    if let Some(record) = parse_log_record(payload, existing_entries) {
        return Some(ParseItem::Log(record));
    }

    if let Some(snapshot) = parse_snapshot_payload(payload) {
        return Some(ParseItem::ProcessSnapshot(snapshot));
    }

    Some(ParseItem::Log(LogItem::Raw(payload.trim_end_matches(['\r', '\n']).to_string())))
}

fn parse_log_record(payload: &str, existing_entries: &[LogItem]) -> Option<LogItem> {
    match parse_log_line(payload) {
        ParsedLine::Log(mut log) => {
            let is_session_separator = existing_entries
                .iter()
                .rposition(|entry| matches!(entry, LogItem::Log(_)))
                .and_then(|idx| match &existing_entries[idx] {
                    LogItem::Log(prev) => Some(is_session_reset(prev.timestamp, log.timestamp)),
                    _ => None,
                })
                .unwrap_or(false);

            log.is_session_separator = is_session_separator;
            Some(LogItem::Log(log))
        }
        ParsedLine::Panic(panic) => Some(LogItem::Panic(panic)),
        ParsedLine::Raw => None,
    }
}

fn is_session_reset(previous_timestamp: f64, current_timestamp: f64) -> bool {
    const TOLERANCE_SECONDS: f64 = 3.0;
    previous_timestamp - current_timestamp > TOLERANCE_SECONDS
}

#[derive(Debug)]
enum ParsedLine {
    Log(LogMessage),
    Panic(PanicMessage),
    Raw,
}

fn parse_log_line(s: &str) -> ParsedLine {
    static LOG_RE: OnceLock<Regex> = OnceLock::new();
    static PANIC_RE: OnceLock<Regex> = OnceLock::new();

    let re = LOG_RE.get_or_init(|| {
        Regex::new(r"(?s)^\[([^\]]+)\]\s(\w+)\s+(\d+)\s([a-z0-9_]+)([a-z0-9_.]*)([a-z0-9_.:]*):\s*(.*)$")
            .unwrap()
    });
    let panic_re = PANIC_RE.get_or_init(|| Regex::new(r"(?s)^PANIC in PID\s+(\d+):\s*(.*)$").unwrap());
    let s = s.trim_end_matches(['\r', '\n']);
    if let Some(caps) = re.captures(s) {
        let Some(level) = parse_level_token(caps.get(2).unwrap().as_str()) else {
            return ParsedLine::Raw;
        };
        let Ok(timestamp) = caps.get(1).unwrap().as_str().trim().parse::<f64>() else {
            return ParsedLine::Raw;
        };
        let pid: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);
        let server = caps.get(4).unwrap().as_str().to_string();
        let path = caps.get(6).unwrap().as_str().trim().trim_start_matches(':').to_string();
        let message = caps.get(7).unwrap().as_str().to_string();

        ParsedLine::Log(LogMessage {
            timestamp,
            level,
            pid,
            server,
            path,
            message,
            is_session_separator: false,
        })
    } else if let Some(caps) = panic_re.captures(s) {
        let pid: u32 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
        let message =
            caps.get(2).map(|m| m.as_str()).unwrap_or("").trim_end_matches(['\r', '\n']).trim().to_string();
        ParsedLine::Panic(PanicMessage { pid, message })
    } else {
        ParsedLine::Raw
    }
}

fn parse_level_token(token: &str) -> Option<Level> {
    match token {
        "ERR" | "ERROR" => Some(Level::Error),
        "WRN" | "WARN" | "WARNING" => Some(Level::Warn),
        "INF" | "INFO" => Some(Level::Info),
        "DBG" | "DEBUG" => Some(Level::Debug),
        "TRC" | "TRACE" => Some(Level::Trace),
        _ => None,
    }
}

pub fn parse_snapshot_payload(payload: &str) -> Option<ProcessListSnapshot> {
    let mut lines =
        payload.lines().map(|line| line.trim_end_matches('\r')).filter(|line| !line.trim().is_empty());

    let first = lines.next()?;
    let expected_rows = parse_proc_header(first)?;

    let mut rows = Vec::with_capacity(expected_rows);
    for _ in 0..expected_rows {
        let row_line = lines.next()?;
        let row = parse_compact_row(row_line)?;
        rows.push(row);
    }

    let (cpu_usage_percent, ram_used_kb, ram_total_kb) = parse_summary(lines.next()?)?;

    if lines.next().is_some() {
        return None;
    }
    let ram_usage_percent =
        if ram_total_kb == 0 { 0 } else { ((ram_used_kb as u64 * 100) / ram_total_kb as u64).min(100) as u8 };

    Some(ProcessListSnapshot { rows, cpu_usage_percent, ram_usage_percent, ram_used_kb, ram_total_kb })
}

fn parse_proc_header(line: &str) -> Option<usize> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "PROC" {
        return None;
    }
    let count = parts.next()?.parse::<usize>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(count)
}

fn parse_compact_row(line: &str) -> Option<ProcessRow> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "R" {
        return None;
    }

    let pid = parts.next()?.parse::<u32>().ok()?;
    let ppid = parts.next()?.parse::<u32>().ok()?;
    let process = parts.next()?.to_string();
    let state = parts.next()?.to_string();
    let cpu_percent = parts.next()?.parse::<u8>().ok()?;
    let ram_kb = parts.next()?.parse::<u32>().ok()?;
    let connections = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some(ProcessRow { pid, ppid, process, state, cpu_percent, ram_kb, connections })
}

fn parse_summary(line: &str) -> Option<(u8, u32, u32)> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "SUM" {
        return None;
    }
    let cpu = parts.next()?.parse::<u8>().ok()?;
    let ram_used = parts.next()?.parse::<u32>().ok()?;
    let ram_total = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((cpu, ram_used, ram_total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiline_structured_log_payload() {
        let payload = "[  12.345] INF 7 app::run: top line\nsecond line\nthird line";
        let Some(ParseItem::Log(record)) = parse_payload(payload, &[]) else {
            panic!("expected log event");
        };
        let LogItem::Log(log) = record else {
            panic!("expected log item");
        };
        assert_eq!(log.message, "top line\nsecond line\nthird line");
    }

    #[test]
    fn parses_multiline_panic_payload() {
        let payload = "PANIC in PID 40:\nthread 'main' panicked\nTerminating process";
        let Some(ParseItem::Log(record)) = parse_payload(payload, &[]) else {
            panic!("expected panic log event");
        };
        let LogItem::Panic(panic) = record else {
            panic!("expected panic item");
        };
        assert_eq!(panic.pid, 40);
        assert_eq!(panic.message, "PANIC in PID 40:\nthread 'main' panicked\nTerminating process");
    }

    #[test]
    fn parses_panic_payload_without_newline_after_prefix() {
        let payload = "PANIC in PID 40:thread 'main' panicked\nTerminating process";
        let Some(ParseItem::Log(record)) = parse_payload(payload, &[]) else {
            panic!("expected panic log event");
        };
        let LogItem::Panic(panic) = record else {
            panic!("expected panic item");
        };
        assert_eq!(panic.pid, 40);
        assert_eq!(panic.message, "PANIC in PID 40:\nthread 'main' panicked\nTerminating process");
    }

    #[test]
    fn parses_compact_process_payload() {
        let payload = "PROC 2\nR 1 0 kernel R 0 1024 0\nR 3 1 xous_ticktimer w 2 64 4\nSUM 12 1088 3200";
        let Some(ParseItem::ProcessSnapshot(snapshot)) = parse_payload(payload, &[]) else {
            panic!("expected process snapshot event");
        };
        assert_eq!(snapshot.rows.len(), 2);
        assert_eq!(snapshot.cpu_usage_percent, 12);
    }

    #[test]
    fn emits_raw_log_for_unstructured_payload() {
        let payload = "kernel output without log prefix";
        let Some(ParseItem::Log(record)) = parse_payload(payload, &[]) else {
            panic!("expected raw log event");
        };
        let LogItem::Raw(raw) = record else {
            panic!("expected raw item");
        };
        assert_eq!(raw, payload);
    }

    #[test]
    fn marks_session_separator_when_timestamp_resets() {
        let prev = LogItem::Log(LogMessage {
            timestamp: 100.0,
            level: Level::Info,
            pid: 1,
            server: "kernel".to_string(),
            path: String::new(),
            message: "old".to_string(),
            is_session_separator: false,
        });

        let payload = "[   0.100] INF 1 kernel: rebooted";
        let Some(ParseItem::Log(record)) = parse_payload(payload, &[prev]) else {
            panic!("expected log event");
        };
        let LogItem::Log(log) = record else {
            panic!("expected log item");
        };
        assert!(log.is_session_separator);
    }

    #[test]
    fn parses_complete_snapshot_payload() {
        let payload = "PROC 2\nR 1 0 kernel R 0 1024 0\nR 3 1 xous_ticktimer w 2 64 4\nSUM 12 1088 3200";
        let snapshot = parse_snapshot_payload(payload).expect("snapshot payload should parse");

        assert_eq!(snapshot.rows.len(), 2);
        assert_eq!(snapshot.cpu_usage_percent, 12);
        assert_eq!(snapshot.ram_usage_percent, 34);
    }
}
