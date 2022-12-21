use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};
use tui_logger::TuiLoggerWidget;

use super::{
    actions::{Action, Actions},
    state::App,
};

pub fn ui(f: &mut Frame<impl Backend>, app: &mut App) {
    let area = f.size();
    check_size(&area);

    let chunks = Layout::default()
        .constraints([Constraint::Min(20), Constraint::Length(3)].as_ref())
        .split(area);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(40)].as_ref())
        .split(chunks[0]);

    {
        // Body & Help
        let sidebar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(body_chunks[1]);

        let help = draw_help(&app.actions);
        f.render_widget(help, sidebar[1]);

        f.render_widget(draw_selection(), sidebar[0]);

        match app.show_input {
            true => {
                f.render_widget(draw_filter(app), chunks[1]);
                f.set_cursor(
                    // Put cursor past the end of the input text
                    chunks[1].x + app.input_cursor_position as u16 + 1,
                    // Move one line down, from the border to the input line
                    chunks[1].y + 1,
                )
            }
            false => f.render_widget(draw_logs(), chunks[1]),
        };
    }

    {
        let content_bar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(20)].as_ref())
            .split(body_chunks[0]);

        f.render_widget(draw_sort(app), content_bar[0]);

        draw_table(app, f, content_bar[1]);
    }

    if app.show_popup {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        let p = Paragraph::new("Preparing mirrors. Please wait...")
            .block(block)
            .alignment(Alignment::Center);
        let area = centered_rect(60, 20, area);
        f.render_widget(Clear, area);
        f.render_widget(p, area);
    }
}

fn draw_table(app: &mut App, f: &mut Frame<impl Backend>, region: Rect) {
    let header_cells = ["  index", "╭─── country", "mirrors"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default()));
    let items: Vec<_> = if let Some(items) = app.mirrors.as_ref() {
        items
            .countries
            .iter()
            .enumerate()
            .filter_map(|(idx, f)| {
                if f.name
                    .to_ascii_lowercase()
                    .contains(&app.input.to_ascii_lowercase())
                {
                    let mut selected = false;
                    let default = format!("├─ [{}] {}", f.code, f.name);
                    let item_name = match app.table_state.selected() {
                        Some(index) => {
                            if idx == index {
                                selected = true;
                                format!("├─»[{}] {}«", f.code, f.name)
                            } else {
                                default
                            }
                        }
                        None => default,
                    };
                    let index = format!("  {idx}│");
                    return Some(Row::new(
                        [index, item_name, f.mirrors.len().to_string()]
                            .iter()
                            .map(|c| {
                                Cell::from(c.clone()).style(if selected {
                                    Style::default()
                                        .add_modifier(Modifier::BOLD)
                                        .fg(Color::Green)
                                } else {
                                    Style::default()
                                })
                            }),
                    ));
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    };

    let count = items.len();
    let header = Row::new(header_cells).height(1);

    let t = Table::new(items)
        .header(header)
        .block(create_block(format!("Results from ({count}) countries")))
        .widths(&[
            Constraint::Percentage(6),
            Constraint::Length(33),
            Constraint::Min(10),
        ]);

    f.render_stateful_widget(t, region, &mut app.table_state);
}

fn draw_help(actions: &Actions) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let rows = actions.actions().iter().filter_map(|action| match action {
        Action::NavigateDown | Action::NavigateUp => None,
        _ => {
            let mut actions: Vec<_> = action
                .keys()
                .iter()
                .map(|k| Span::styled(k.to_string(), key_style))
                .collect();

            if actions.len() == 1 {
                actions.push(Span::raw(""));
            }

            let text = Span::styled(action.to_string(), help_style);
            actions.push(text);
            Some(Row::new(actions))
        }
    });

    Table::new(rows)
        .block(create_block("Help"))
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(60),
        ])
        .column_spacing(1)
}

fn check_size(area: &Rect) {
    if area.width < 52 {
        panic!("Require width >= 52, (got {})", area.width);
    }
    if area.height < 28 {
        panic!("Require height >= 28, (got {})", area.height);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn draw_logs<'a>() -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Blue))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Magenta))
        .style_info(Style::default().fg(Color::Green))
        .output_file(false)
        .output_timestamp(None)
        .output_line(false)
        .output_target(false)
        .block(create_block("Logs"))
}

fn draw_filter(app: &App) -> Paragraph {
    Paragraph::new(app.input.as_ref()).block(create_block("Filter"))
}

fn draw_selection<'a>() -> Block<'a> {
    create_block("Selection")
}

fn draw_sort<'a>(app: &App) -> Paragraph<'a> {
    let count = app.active_sort.len() + app.active_filter.len();
    let mut sorts: Vec<_> = app
        .active_sort
        .iter()
        .enumerate()
        .flat_map(|(idx, f)| {
            let mut ret = vec![
                Span::raw(format!(" [{f}]")),
                Span::styled(" ⇣", Style::default()),
            ];
            if idx < count - 1 {
                ret.push(Span::styled(" 🢒", Style::default().fg(Color::Black)))
            }
            ret
        })
        .collect();

    let count = app.active_filter.len();

    let mut filters: Vec<_> = app
        .active_filter
        .iter()
        .enumerate()
        .flat_map(|(idx, f)| {
            let mut ret = vec![Span::styled(
                format!(" {f}"),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )];
            if idx < count - 1 {
                ret.push(Span::styled(" 🢒", Style::default().fg(Color::Black)))
            }
            ret
        })
        .collect();

    sorts.append(&mut filters);

    let widget = Spans::from(sorts);

    let bt = format!("Sort ({count})");

    Paragraph::new(widget).block(create_block(bt))
}

fn create_block<'a>(title: impl Into<String>) -> Block<'a> {
    let title = title.into();
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Black))
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ))
}
