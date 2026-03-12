// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Cell, Chart, Clear, Dataset, Gauge, GraphType, List, ListItem, Paragraph, Row,
        Table, Tabs,
    },
    Frame,
};

use super::centered_rect;
use crate::state::process::{ProcessConfigTab, ProcessSortField, ProcessViewState};

pub fn render(frame: &mut Frame, rect: Rect, state: &mut ProcessViewState) {
    let Some(snapshot_state) = state.snapshot.as_ref() else {
        let block = Block::default().borders(Borders::ALL).title("Process List");
        let inner = block.inner(rect);
        frame.render_widget(block, rect);

        let empty_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
            .split(inner);

        let empty =
            Paragraph::new("No process snapshot yet. Press 'p' to request one.").alignment(Alignment::Center);
        frame.render_widget(empty, empty_layout[1]);
        state.table_state.select(None);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(rect);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[1]);

    let snapshot_age = snapshot_state.received_at.elapsed().as_secs();
    let mut table_block =
        Block::default().borders(Borders::ALL).title(format!("Process List ({}s ago)", snapshot_age));

    table_block = table_block.border_style(Style::default().fg(Color::Yellow));

    let visible_rows = state.visible_rows();
    let rows = visible_rows
        .iter()
        .map(|process| {
            Row::new(vec![
                Cell::from(process.pid.to_string()),
                Cell::from(process.ppid.to_string()),
                Cell::from(process.process.clone()),
                Cell::from(process.state.clone()),
                Cell::from(format!("{}%", process.cpu_percent)),
                Cell::from(format!("{}K", process.ram_kb)),
                Cell::from(process.connections.to_string()),
            ])
        })
        .collect::<Vec<_>>();

    ensure_table_selection(&mut state.table_state, rows.len());

    let header = Row::new(vec!["PID", "PPID", "Process", "State", "CPU", "RAM", "Conn"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(table_block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .column_spacing(1)
    .highlight_symbol(">> ");

    frame.render_stateful_widget(table, chunks[0], &mut state.table_state);

    let first_at = state.usage_history.front().map(|sample| sample.at).unwrap_or(snapshot_state.received_at);
    let cpu_points = state
        .usage_history
        .iter()
        .map(|sample| (sample.at.duration_since(first_at).as_secs_f64(), sample.cpu as f64))
        .collect::<Vec<_>>();
    let ram_points = state
        .usage_history
        .iter()
        .map(|sample| (sample.at.duration_since(first_at).as_secs_f64(), sample.ram as f64))
        .collect::<Vec<_>>();
    let x_max = cpu_points.last().map(|(x, _)| *x).unwrap_or(1.0);

    let history_datasets = vec![
        Dataset::default()
            .name("CPU")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::LightBlue))
            .data(&cpu_points),
        Dataset::default()
            .name("RAM")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::LightGreen))
            .data(&ram_points),
    ];
    let history_chart = Chart::new(history_datasets)
        .block(Block::default().borders(Borders::ALL).title("History"))
        .x_axis(Axis::default().bounds([0.0, x_max.max(1.0)]))
        .y_axis(Axis::default().bounds([0.0, 100.0]));
    frame.render_widget(history_chart, bottom_chunks[0]);

    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(bottom_chunks[1]);

    let cpu_percent = snapshot_state.snapshot.cpu_usage_percent;
    let ram_percent = snapshot_state.snapshot.ram_usage_percent;

    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("CPU Usage"))
        .gauge_style(Style::default().fg(Color::LightBlue).bg(Color::Black))
        .percent(cpu_percent as u16)
        .label(format!("{}%", cpu_percent));
    frame.render_widget(cpu_gauge, gauge_chunks[0]);

    let ram_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("RAM Usage"))
        .gauge_style(Style::default().fg(Color::LightGreen).bg(Color::Black))
        .percent(ram_percent as u16)
        .label(format!(
            "{}% ({}K / {}K)",
            ram_percent, snapshot_state.snapshot.ram_used_kb, snapshot_state.snapshot.ram_total_kb
        ));
    frame.render_widget(ram_gauge, gauge_chunks[1]);

    if state.modal_open {
        render_filter_modal(frame, state, frame.area());
    }
}

fn render_filter_modal(frame: &mut Frame, state: &mut ProcessViewState, area: Rect) {
    let overlay = centered_rect(56, 56, area);
    frame.render_widget(Clear, overlay);
    frame.render_widget(Block::default().style(Style::default().bg(Color::DarkGray)), overlay);
    let inner = overlay.inner(Margin { vertical: 1, horizontal: 2 });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    let tab_titles = ["Processes", "Sort"];
    let tab_index = match state.modal_tab {
        ProcessConfigTab::Processes => 0,
        ProcessConfigTab::Sort => 1,
    };
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(tab_index)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, chunks[0]);

    let items = match state.modal_tab {
        ProcessConfigTab::Processes => state
            .process_names
            .iter()
            .map(|process_name| {
                let checked = if state.process_name_filters.contains(process_name) { "[x]" } else { "[ ]" };
                ListItem::new(Line::from(format!("{} {}", checked, process_name)))
            })
            .collect::<Vec<_>>(),
        ProcessConfigTab::Sort => {
            let sort_fields = [
                ProcessSortField::Pid,
                ProcessSortField::Ppid,
                ProcessSortField::Process,
                ProcessSortField::State,
                ProcessSortField::Cpu,
                ProcessSortField::Ram,
                ProcessSortField::Connections,
            ];
            sort_fields
                .iter()
                .map(|field| {
                    let status = if *field == state.sort_by {
                        if state.sort_desc {
                            "[D]"
                        } else {
                            "[A]"
                        }
                    } else {
                        "[ ]"
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(status, Style::default().fg(Color::Cyan)),
                        Span::raw(" "),
                        Span::raw(field.label()),
                    ]))
                })
                .collect::<Vec<_>>()
        }
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    match state.modal_tab {
        ProcessConfigTab::Processes => {
            frame.render_stateful_widget(list, chunks[1], &mut state.process_names_state);
        }
        ProcessConfigTab::Sort => {
            frame.render_stateful_widget(list, chunks[1], &mut state.sort_state);
        }
    }

    let help = Paragraph::new("f/Esc: close | <-/->: tabs | Up/Down: move | Space: toggle | a: all/reset")
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(help, chunks[2]);
}

fn ensure_table_selection(table_state: &mut ratatui::widgets::TableState, len: usize) {
    match (table_state.selected(), len) {
        (_, 0) => table_state.select(None),
        (None, _) => table_state.select(Some(0)),
        (Some(selected), _) if selected >= len => table_state.select(Some(len - 1)),
        _ => {}
    }
}
