// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Tabs,
    },
    Frame,
};
use textwrap::wrap;

use super::centered_rect;
use crate::state::log::{
    CachedRenderLine, LogConfigTab, LogItem, LogLevel, LogMessage, LogRenderCache, LogRenderCacheKey,
    LogViewState, PanicMessage,
};

const LOG_COLUMN_SPACING: u16 = 1;
const LOG_SELECTION_SYMBOL: &str = ">> ";
const LOG_SELECTION_COLUMN_WIDTH: u16 = LOG_SELECTION_SYMBOL.len() as u16;
const LOG_LEVEL_COLUMN_WIDTH: u16 = 5;

pub fn render(frame: &mut Frame, rect: Rect, state: &mut LogViewState) {
    let show_search = state.search_mode || !state.search_query.is_empty();
    let (search_area, table_area) = if show_search {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(rect);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, rect)
    };

    if let Some(search_area) = search_area {
        render_search_bar(frame, search_area, state);
    }

    let log_block = {
        let mut block = Block::default().borders(Borders::ALL).title("Logs");
        if !state.search_mode {
            block = block.border_style(Style::default().fg(Color::Yellow));
        }
        block
    };
    frame.render_widget(log_block, table_area);

    let inner_area = table_area.inner(Margin { vertical: 1, horizontal: 1 });
    let header_height = inner_area.height.min(1);
    let [header_area, body_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(header_height), Constraint::Min(0)])
        .areas(inner_area);

    let [body_area, scrollbar_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .areas(body_area);

    state.set_viewport_line_count(body_area.height as usize);
    let previous_total_lines = state.render_cache.lines.len();
    let was_at_bottom = state.is_cursor_line_at_bottom(previous_total_lines);

    ensure_log_render_cache(state, body_area.width);

    sync_log_view_state(state, was_at_bottom);

    render_log_header(frame, header_area, state);
    render_visible_log_lines(
        frame,
        body_area,
        &state.render_cache,
        state.selected_entry_idx(),
        state.cursor_line,
        state.viewport_start,
    );

    let (content_length, position) = log_scrollbar_metrics(
        state.render_cache.lines.len(),
        state.viewport_start,
        state.viewport_line_count,
    );
    let mut scrollbar_state = ScrollbarState::new(content_length)
        .position(position)
        .viewport_content_length(state.viewport_line_count);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);

    if state.modal_open {
        render_filter_overlay(frame, state, frame.area());
    }
}

fn sync_log_view_state(state: &mut LogViewState, was_at_bottom: bool) {
    let total_lines = state.render_cache.lines.len();
    if total_lines == 0 {
        state.cursor_line = 0;
        state.viewport_start = 0;
        return;
    }

    if was_at_bottom {
        state.cursor_line = state.max_cursor_line(total_lines);
    }

    state.clamp_cursor_line(total_lines);
    state.sync_viewport_start_to_cursor_line(total_lines);
    state.clamp_viewport_start(total_lines);
}

fn render_search_bar(f: &mut Frame, area: Rect, state: &LogViewState) {
    let (input, color) = if state.search_mode {
        (&state.search_input, Color::Yellow)
    } else {
        (&state.search_query, Color::Gray)
    };

    let mut block =
        Block::default().borders(Borders::ALL).title("Search").border_style(Style::default().fg(color));
    if state.search_mode {
        block = block.title("(Enter: apply  Esc: clear)");
    }

    let paragraph = Paragraph::new(input.as_str()).style(Style::default().fg(color)).block(block);
    f.render_widget(paragraph, area);
}

fn render_log_header(frame: &mut Frame, area: Rect, state: &LogViewState) {
    let selection_width = LOG_SELECTION_COLUMN_WIDTH.min(area.width);
    let [_, content_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(selection_width), Constraint::Min(0)])
        .areas(area);

    let mut columns = vec![("Time", Constraint::Length(8)), ("LVL", Constraint::Length(5))];
    if state.show_pid {
        columns.push(("PID", Constraint::Length(5)));
    }
    if state.show_server {
        columns.push(("Server", Constraint::Length(14)));
    }
    if state.show_path {
        columns.push(("Path", Constraint::Length(18)));
    }
    columns.push(("Message", Constraint::Min(20)));

    let constraints = columns.iter().map(|(_, constraint)| *constraint).collect::<Vec<_>>();
    let cells = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .spacing(LOG_COLUMN_SPACING)
        .split(content_area);
    let style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    for ((label, _), cell_area) in columns.iter().zip(cells.iter()) {
        if cell_area.width == 0 {
            continue;
        }
        frame.render_widget(
            Paragraph::new(pad_or_truncate(label, cell_area.width as usize)).style(style),
            *cell_area,
        );
    }
}

fn render_visible_log_lines(
    frame: &mut Frame,
    area: Rect,
    cache: &LogRenderCache,
    selected_entry_idx: Option<usize>,
    cursor_line: usize,
    viewport_start: usize,
) {
    if cache.lines.is_empty() {
        return;
    }

    let viewport_start = viewport_start.min(cache.lines.len().saturating_sub(1));
    let viewport_end = viewport_start.saturating_add(area.height as usize).min(cache.lines.len());

    let content_x = area.x.saturating_add(LOG_SELECTION_COLUMN_WIDTH);
    let content_width = area.width.saturating_sub(LOG_SELECTION_COLUMN_WIDTH);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let buffer = frame.buffer_mut();
    for (offset, cached_line) in cache.lines[viewport_start..viewport_end].iter().enumerate() {
        let y = area.y.saturating_add(offset as u16);
        buffer.set_line(content_x, y, &cached_line.line, content_width);
        if selected_entry_idx == Some(cached_line.entry_idx) {
            buffer.set_style(Rect::new(content_x, y, content_width, 1), selected_style);
        }
    }

    let arrow_y = area.y.saturating_add((cursor_line - viewport_start) as u16);
    let arrow_area = Rect::new(area.x, arrow_y, LOG_SELECTION_COLUMN_WIDTH, 1);
    frame.render_widget(Paragraph::new(LOG_SELECTION_SYMBOL).style(selected_style), arrow_area);
}

fn log_scrollbar_metrics(
    total_lines: usize,
    viewport_start: usize,
    viewport_line_count: usize,
) -> (usize, usize) {
    if total_lines == 0 {
        return (0, 0);
    }
    let max_scroll_offset = total_lines.saturating_sub(viewport_line_count);
    let content_length = max_scroll_offset.saturating_add(1);
    let position = viewport_start.min(max_scroll_offset);
    (content_length, position)
}

fn ensure_log_render_cache(state: &mut LogViewState, body_width: u16) {
    let key = LogRenderCacheKey { width: body_width, generation: state.render_cache_generation };
    let entries_len = state.entries.len();

    let should_rebuild = state.render_cache.key != key || state.render_cache.source_len > entries_len;
    if should_rebuild {
        state.render_cache = rebuild_render_cache(state, key);
        return;
    }

    let append_start = state.render_cache.source_len;
    if append_start >= entries_len {
        return;
    }

    let render_ctx = build_render_context(state, body_width);
    for (offset, entry) in state.entries[append_start..].iter().enumerate() {
        if state.entry_visible(entry) {
            append_visible_entry_rows(&mut state.render_cache, &render_ctx, entry, append_start + offset);
        }
    }
    state.render_cache.source_len = entries_len;
}

fn rebuild_render_cache(state: &LogViewState, key: LogRenderCacheKey) -> LogRenderCache {
    let render_ctx = build_render_context(state, key.width);
    let mut cache = LogRenderCache { key, source_len: state.entries.len(), lines: Vec::new() };

    for (entry_idx, entry) in state.entries.iter().enumerate() {
        if !state.entry_visible(entry) {
            continue;
        }
        append_visible_entry_rows(&mut cache, &render_ctx, entry, entry_idx);
    }

    cache
}

#[derive(Clone)]
struct RenderContext {
    line_width: u16,
    separator_width: usize,
    show_pid: bool,
    show_server: bool,
    show_path: bool,
}

fn build_render_context(state: &LogViewState, body_width: u16) -> RenderContext {
    let line_width = body_width.saturating_sub(LOG_SELECTION_COLUMN_WIDTH);
    RenderContext {
        line_width,
        separator_width: body_width as usize,
        show_pid: state.show_pid,
        show_server: state.show_server,
        show_path: state.show_path,
    }
}

fn append_visible_entry_rows(
    cache: &mut LogRenderCache,
    render_ctx: &RenderContext,
    entry: &LogItem,
    entry_idx: usize,
) {
    let lines = match entry {
        LogItem::Log(log) => {
            if log.is_session_separator {
                let line = build_session_separator_row(render_ctx.separator_width);
                cache.lines.push(CachedRenderLine { entry_idx, line });
            }
            build_standard_row(render_ctx, log)
        }
        LogItem::Panic(panic) => build_panic_row(render_ctx, panic),
        LogItem::Raw(raw) => build_raw_row(render_ctx, raw),
    };

    for line in lines {
        cache.lines.push(CachedRenderLine { entry_idx, line });
    }
}

fn build_session_separator_row(width: usize) -> Line<'static> {
    let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let label = " SESSION RESET ";
    let fill_width = width.saturating_sub(label.len());
    let left_fill = fill_width / 2;
    let right_fill = fill_width.saturating_sub(left_fill);

    Line::from(vec![
        Span::styled("=".repeat(left_fill), style),
        Span::styled(label, style),
        Span::styled("=".repeat(right_fill), style),
    ])
}

fn build_standard_row(render_ctx: &RenderContext, log: &LogMessage) -> Vec<Line<'static>> {
    let level = LogLevel::Standard(log.level);
    let mut columns = vec![
        ColumnSpec::fixed(format!("{:.3}", log.timestamp), Constraint::Length(8), Style::default()),
        ColumnSpec::fixed(
            level.as_str().to_string(),
            Constraint::Length(5),
            Style::default().fg(level.color()),
        ),
    ];
    if render_ctx.show_pid {
        columns.push(ColumnSpec::fixed(log.pid.to_string(), Constraint::Length(5), Style::default()));
    }
    if render_ctx.show_server {
        columns.push(ColumnSpec::fixed(log.server.clone(), Constraint::Length(14), Style::default()));
    }
    if render_ctx.show_path {
        columns.push(ColumnSpec::fixed(log.path.clone(), Constraint::Length(18), Style::default()));
    }
    columns.push(ColumnSpec::wrapped(log.message.clone(), Constraint::Min(20), Style::default()));

    build_row_lines(&columns, render_ctx.line_width)
}

fn build_panic_row(render_ctx: &RenderContext, panic: &PanicMessage) -> Vec<Line<'static>> {
    let mut columns = vec![ColumnSpec::fixed(
        "PANIC".to_string(),
        Constraint::Length(LOG_LEVEL_COLUMN_WIDTH),
        Style::default().fg(LogLevel::Panic.color()),
    )];
    if render_ctx.show_pid {
        columns.push(ColumnSpec::fixed(panic.pid.to_string(), Constraint::Length(5), Style::default()));
    }
    columns.push(ColumnSpec::wrapped(panic.message.clone(), Constraint::Min(1), Style::default()));

    build_row_lines(&columns, render_ctx.line_width)
}

fn build_raw_row(render_ctx: &RenderContext, raw: &str) -> Vec<Line<'static>> {
    let columns = vec![
        ColumnSpec::fixed(
            "RAW".to_string(),
            Constraint::Length(LOG_LEVEL_COLUMN_WIDTH),
            Style::default().fg(LogLevel::Raw.color()),
        ),
        ColumnSpec::wrapped(raw.to_string(), Constraint::Min(1), Style::default()),
    ];

    build_row_lines(&columns, render_ctx.line_width)
}

#[derive(Clone)]
struct ColumnSpec {
    text: String,
    style: Style,
    constraint: Constraint,
    wrap: bool,
}

impl ColumnSpec {
    fn fixed(text: String, constraint: Constraint, style: Style) -> Self {
        Self { text, style, constraint, wrap: false }
    }

    fn wrapped(text: String, constraint: Constraint, style: Style) -> Self {
        Self { text, style, constraint, wrap: true }
    }
}

fn build_row_lines(columns: &[ColumnSpec], line_width: u16) -> Vec<Line<'static>> {
    enum ColumnContent {
        Fixed(String),
        Wrapped(Vec<String>),
    }

    let constraints = columns.iter().map(|column| column.constraint).collect::<Vec<_>>();
    let widths = compute_column_widths(line_width, &constraints);

    let content = columns
        .iter()
        .zip(widths.iter())
        .map(|(column, width)| {
            if column.wrap {
                ColumnContent::Wrapped(wrap_lines(&column.text, *width))
            } else {
                ColumnContent::Fixed(column.text.clone())
            }
        })
        .collect::<Vec<_>>();

    let line_count = content
        .iter()
        .map(|content| match content {
            ColumnContent::Fixed(_) => 1,
            ColumnContent::Wrapped(lines) => lines.len(),
        })
        .max()
        .unwrap_or(1);

    (0..line_count)
        .map(|line_idx| {
            let mut parts = Vec::new();
            for (idx, (column, width)) in columns.iter().zip(widths.iter()).enumerate() {
                if idx > 0 {
                    parts.push(Span::raw(" ".repeat(LOG_COLUMN_SPACING as usize)));
                }

                let text = match &content[idx] {
                    ColumnContent::Fixed(text) => {
                        if line_idx == 0 {
                            text.as_str()
                        } else {
                            ""
                        }
                    }
                    ColumnContent::Wrapped(lines) => lines.get(line_idx).map(String::as_str).unwrap_or(""),
                };
                parts.push(Span::styled(pad_or_truncate(text, *width), column.style));
            }

            Line::from(parts)
        })
        .collect()
}

fn wrap_lines(message: &str, width: usize) -> Vec<String> {
    wrap(message, width).into_iter().map(|part| part.into_owned()).collect::<Vec<_>>()
}

fn pad_or_truncate(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut out = value.chars().take(width).collect::<String>();
    let char_count = out.chars().count();
    if char_count < width {
        out.push_str(&" ".repeat(width - char_count));
    }
    out
}

fn compute_column_widths(content_width: u16, constraints: &[Constraint]) -> Vec<usize> {
    let rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.to_vec())
        .spacing(LOG_COLUMN_SPACING)
        .split(Rect::new(0, 0, content_width, 1));
    rects.iter().map(|rect| rect.width as usize).collect()
}

fn render_filter_overlay(f: &mut Frame, state: &mut LogViewState, area: Rect) {
    let overlay = centered_rect(56, 56, area);
    f.render_widget(Clear, overlay);
    f.render_widget(Block::default().style(Style::default().bg(Color::DarkGray)), overlay);
    let inner = overlay.inner(Margin { vertical: 1, horizontal: 2 });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    let tab_titles = ["Levels", "Servers", "Columns"];
    let tab_index = match state.modal_tab {
        LogConfigTab::Levels => 0,
        LogConfigTab::Servers => 1,
        LogConfigTab::Columns => 2,
    };
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(tab_index)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    let items = match state.modal_tab {
        LogConfigTab::Levels => state
            .levels
            .iter()
            .map(|level| {
                let checked = if state.level_filters.contains(level) { "[x]" } else { "[ ]" };
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} ", checked)),
                    Span::styled(level.as_str().to_string(), Style::default().fg(level.color())),
                ]))
            })
            .collect::<Vec<_>>(),
        LogConfigTab::Servers => state
            .pid_servers
            .iter()
            .map(|(pid, server)| {
                let checked =
                    if state.pid_server_filters.contains(&(*pid, server.clone())) { "[x]" } else { "[ ]" };
                ListItem::new(Line::from(format!("{} {} - {}", checked, pid, server)))
            })
            .collect::<Vec<_>>(),
        LogConfigTab::Columns => {
            [("PID", state.show_pid), ("Server", state.show_server), ("Path", state.show_path)]
                .into_iter()
                .map(|(label, enabled)| {
                    let check = if enabled { "[x]" } else { "[ ]" };
                    ListItem::new(Line::from(format!("{} {}", check, label)))
                })
                .collect::<Vec<_>>()
        }
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    match state.modal_tab {
        LogConfigTab::Levels => {
            f.render_stateful_widget(list, chunks[1], &mut state.levels_state);
        }
        LogConfigTab::Servers => {
            f.render_stateful_widget(list, chunks[1], &mut state.servers_state);
        }
        LogConfigTab::Columns => {
            f.render_stateful_widget(list, chunks[1], &mut state.columns_state);
        }
    }

    let help_text = if matches!(state.modal_tab, LogConfigTab::Servers) {
        "f/Esc: close | <-/->: tabs | Up/Down: move | Space: toggle | a: all/none | u: U2F"
    } else {
        "f/Esc: close | <-/->: tabs | Up/Down: move | Space: toggle | a: all/none"
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::Gray));
    f.render_widget(help, chunks[2]);
}

impl LogLevel {
    fn color(&self) -> Color {
        match self {
            LogLevel::Panic => Color::Red,
            LogLevel::Raw => Color::Gray,
            LogLevel::Standard(log::Level::Error) => Color::Red,
            LogLevel::Standard(log::Level::Warn) => Color::Rgb(255, 165, 0),
            LogLevel::Standard(log::Level::Info) => Color::Yellow,
            LogLevel::Standard(log::Level::Debug) => Color::Blue,
            LogLevel::Standard(log::Level::Trace) => Color::Green,
        }
    }
}
