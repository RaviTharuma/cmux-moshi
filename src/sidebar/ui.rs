//! Minimal ratatui surface for the optional workspace picker.
//!
//! Intentionally small: query, friendly titles, reconnect copy, key hint.
//! This is not a Moshi panel.

use crate::sidebar::picker::{PickerView, Status, VisibleRow};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;
use std::collections::HashSet;

/// Draws the picker into `frame`.
pub fn draw(frame: &mut Frame<'_>, view: &PickerView<'_>) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_prompt(frame, chunks[0], view.query);
    match view.status {
        Status::Ready => draw_rows(frame, chunks[1], view),
        Status::Reconnecting { message } => draw_reconnect(frame, chunks[1], message),
    }
    draw_footer(frame, chunks[2], view);
}

fn draw_prompt(frame: &mut Frame<'_>, area: Rect, query: &str) {
    let width = area.width.saturating_sub(3) as usize;
    let query = tail_chars(query, width);
    let line = Line::from(vec![
        Span::raw("> "),
        Span::styled(query, Style::new().add_modifier(Modifier::BOLD)),
        Span::raw("█"),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_rows(frame: &mut Frame<'_>, area: Rect, view: &PickerView<'_>) {
    if area.height == 0 {
        return;
    }
    if view.rows.is_empty() {
        frame.render_widget(Paragraph::new("No workspaces"), area);
        return;
    }

    let visible_height = area.height as usize;
    let offset = scroll_offset(view.selected, visible_height, view.rows.len());
    for (line_idx, visible) in view
        .rows
        .iter()
        .skip(offset)
        .take(visible_height)
        .enumerate()
    {
        let y = area.y + line_idx as u16;
        let selected = offset + line_idx == view.selected;
        frame.render_widget(
            Paragraph::new(row_line(visible, area.width as usize, selected)),
            Rect::new(area.x, y, area.width, 1),
        );
    }
}

fn draw_reconnect(frame: &mut Frame<'_>, area: Rect, message: &str) {
    let width = area.width as usize;
    let lines = [
        "Waiting for cmux",
        message,
        "This picker is optional packaging. Moshi phones use the cmux-moshi CLI.",
    ];
    for (idx, line) in lines.iter().enumerate() {
        if idx >= area.height as usize {
            break;
        }
        frame.render_widget(
            Paragraph::new(Line::from(truncate(line, width))),
            Rect::new(area.x, area.y + idx as u16, area.width, 1),
        );
    }
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, view: &PickerView<'_>) {
    let text = format!(
        "{}/{}  enter select · esc clear · ctrl-c exit",
        view.rows.len(),
        view.total_rows
    );
    frame.render_widget(Paragraph::new(truncate(&text, area.width as usize)), area);
}

fn row_line(row: &VisibleRow<'_>, width: usize, selected: bool) -> Line<'static> {
    let marker = if row.row.active { "* " } else { "  " };
    let hint = format!("  {}", row.row.hint());
    let title_budget = width
        .saturating_sub(marker.chars().count() + hint.chars().count())
        .max(1);
    let title = truncate(&row.row.title, title_budget);
    let matches: HashSet<usize> = row.match_positions.iter().copied().collect();
    let base = if selected {
        Style::new().add_modifier(Modifier::REVERSED)
    } else {
        Style::new()
    };

    let mut spans = vec![Span::styled(marker.to_string(), base)];
    for (idx, ch) in title.chars().enumerate() {
        let mut style = base;
        if matches.contains(&idx) {
            style = style.add_modifier(Modifier::BOLD);
        }
        spans.push(Span::styled(ch.to_string(), style));
    }
    spans.push(Span::styled(hint, base.add_modifier(Modifier::DIM)));
    Line::from(spans)
}

fn scroll_offset(selected: usize, visible_height: usize, total: usize) -> usize {
    if visible_height == 0 || total <= visible_height {
        return 0;
    }
    if selected < visible_height {
        return 0;
    }
    (selected + 1)
        .saturating_sub(visible_height)
        .min(total - visible_height)
}

fn tail_chars(input: &str, max_chars: usize) -> String {
    let chars: Vec<char> = input.chars().collect();
    if chars.len() <= max_chars {
        return input.to_string();
    }
    chars[chars.len().saturating_sub(max_chars)..]
        .iter()
        .collect()
}

fn truncate(input: &str, max_chars: usize) -> String {
    let chars: Vec<char> = input.chars().collect();
    if chars.len() <= max_chars {
        return input.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }
    chars.into_iter().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_narrow_sidebar_titles() {
        assert_eq!(truncate("accounting-workspace", 6), "accoun");
        assert_eq!(truncate("ok", 10), "ok");
    }

    #[test]
    fn scroll_stays_in_range() {
        assert_eq!(scroll_offset(0, 5, 3), 0);
        assert_eq!(scroll_offset(6, 5, 10), 2);
    }
}
