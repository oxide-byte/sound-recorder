use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::model::{AppState, MonitoringSubState, TuiContext};

pub fn render(frame: &mut Frame, ctx: &TuiContext) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let is_recording = matches!(ctx.app_state, AppState::Recording(_));
    let is_playing = matches!(ctx.app_state, AppState::Playing(_));
    let is_idle = matches!(ctx.app_state, AppState::Idle);
    let is_monitoring = matches!(ctx.app_state, AppState::Monitoring(_));
    let is_capturing = if let AppState::Monitoring(ref h) = ctx.app_state {
        matches!(h.sub_state, MonitoringSubState::Capturing)
    } else {
        false
    };

    let record_style = if is_recording {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if is_idle {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let monitor_style = if is_monitoring || is_capturing {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if is_idle {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let play_style = if is_playing {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else if is_idle && !ctx.wav_files.is_empty() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let stop_style = if is_recording || is_playing || is_monitoring {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let button_line = Line::from(vec![
        Span::styled(" [ Record ] ", record_style),
        Span::raw("  "),
        Span::styled(" [ Monitor ] ", monitor_style),
        Span::raw("  "),
        Span::styled(" [ Play ] ", play_style),
        Span::raw("  "),
        Span::styled(" [ Stop ] ", stop_style),
    ]);

    let title = match &ctx.defaults {
        Some(d) => format!(
            " sound-recorder — {}/{} ",
            d.profile.format.as_id(),
            d.profile.compression.as_id()
        ),
        None => " sound-recorder — config error ".to_string(),
    };
    let buttons =
        Paragraph::new(button_line).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(buttons, chunks[0]);

    let items: Vec<ListItem> = ctx
        .wav_files
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if is_playing {
                if let AppState::Playing(handle) = &ctx.app_state {
                    if handle.source_path == entry.path {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                }
            } else if ctx.selected_index == Some(i) {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(entry.name.clone()).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(ctx.selected_index);

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Recordings (↑/↓ to select) "),
        )
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    let status_text = if is_capturing {
        "Capturing — sound detected, recording segment…  's' to stop"
    } else {
        ctx.status_message.as_deref().unwrap_or({
            if ctx.wav_files.is_empty() {
                "Ready — press 'r' to record or 'm' to monitor"
            } else if is_idle {
                "Ready — ↑/↓ select  r record  m monitor  p play  q quit"
            } else {
                ""
            }
        })
    };

    let is_error = ctx
        .status_message
        .as_ref()
        .map(|s| s.contains("error") || s.contains("Error"))
        .unwrap_or(false);

    let status_style = if is_error {
        Style::default().fg(Color::Red)
    } else if is_capturing || is_monitoring {
        Style::default().fg(Color::Yellow)
    } else if is_recording {
        Style::default().fg(Color::Red)
    } else if is_playing {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    frame.render_widget(Paragraph::new(status_text).style(status_style), chunks[2]);
}
