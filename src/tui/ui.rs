use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use super::app::{App, View};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // filter bar
            Constraint::Min(5),   // main content
            Constraint::Length(3), // keybind bar
        ])
        .split(f.area());

    draw_filter_bar(f, app, chunks[0]);

    match &app.view {
        View::List => draw_list_view(f, app, chunks[1]),
        View::Tree => draw_tree_view(f, app, chunks[1]),
        View::Detail => draw_detail_view(f, app, chunks[1]),
        View::Search => draw_search_view(f, app, chunks[1]),
    }

    draw_keybind_bar(f, app, chunks[2]);
}

fn draw_filter_bar(f: &mut Frame, app: &App, area: Rect) {
    let filter = &app.filter;

    let mut spans = vec![Span::styled(" Filters: ", Style::default().fg(Color::DarkGray))];

    if filter.is_empty() && !app.filter_panel_open {
        spans.push(Span::styled("(none)", Style::default().fg(Color::DarkGray)));
    } else {
        if let Some(ref k) = filter.kind {
            spans.push(Span::styled(
                format!(" kind:{} ", k),
                Style::default().bg(Color::Blue).fg(Color::White),
            ));
            spans.push(Span::raw(" "));
        }
        if let Some(ref w) = filter.weight {
            spans.push(Span::styled(
                format!(" weight:{} ", w),
                Style::default().bg(Color::Magenta).fg(Color::White),
            ));
            spans.push(Span::raw(" "));
        }
        if let Some(ref s) = filter.status {
            spans.push(Span::styled(
                format!(" status:{} ", s),
                Style::default().bg(Color::Green).fg(Color::White),
            ));
            spans.push(Span::raw(" "));
        }
        if let Some(ref l) = filter.level {
            spans.push(Span::styled(
                format!(" level:{} ", l),
                Style::default().bg(Color::Cyan).fg(Color::White),
            ));
            spans.push(Span::raw(" "));
        }
        if let Some(ref sc) = filter.scope {
            spans.push(Span::styled(
                format!(" scope:{} ", sc),
                Style::default().bg(Color::Yellow).fg(Color::Black),
            ));
        }
    }

    if app.filter_panel_open {
        spans.push(Span::styled(
            " | 1:kind 2:weight 3:status 4:level 0:clear ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let filter_bar = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .title(" dictum ")
            .title_style(Style::default().add_modifier(Modifier::BOLD)),
    );
    f.render_widget(filter_bar, area);
}

fn draw_list_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: decision table
    let header = Row::new(vec!["ID", "Kind", "Wt", "Level", "Title"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .decisions
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let style = if i == app.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                match d.status {
                    crate::model::decision::Status::Deprecated => {
                        Style::default().fg(Color::DarkGray)
                    }
                    crate::model::decision::Status::Superseded => {
                        Style::default().fg(Color::DarkGray)
                    }
                    _ => Style::default(),
                }
            };
            Row::new(vec![
                Cell::from(d.id.chars().take(8).collect::<String>()),
                Cell::from(d.kind.to_string()),
                Cell::from(d.weight.to_string()),
                Cell::from(d.level.to_string()),
                Cell::from(d.title.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Decisions ({}) ", app.decisions.len())),
    );

    f.render_widget(table, chunks[0]);

    // Right: detail panel
    draw_detail_panel(f, app, chunks[1]);
}

fn draw_tree_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: tree list
    let lines: Vec<Line> = app
        .tree_nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let indent = "  ".repeat(node.depth);
            let marker = if node.has_children {
                if app.expanded_nodes.contains(&node.id) {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let id_short: String = node.id.chars().take(8).collect();
            let text = format!("{}{}{} {}", indent, marker, id_short, node.title);

            let style = if i == app.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Line::styled(text, style)
        })
        .collect();

    let tree_widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Tree ({}) ", app.tree_nodes.len())),
    );
    f.render_widget(tree_widget, chunks[0]);

    // Right: detail panel
    draw_detail_panel(f, app, chunks[1]);
}

fn draw_detail_panel(f: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(ref d) = app.selected_decision {
        build_detail_lines(d, &app.selected_links)
    } else {
        vec![Line::styled(
            "No decision selected",
            Style::default().fg(Color::DarkGray),
        )]
    };

    let detail = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Detail "))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    f.render_widget(detail, area);
}

fn draw_detail_view(f: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(ref d) = app.selected_decision {
        build_detail_lines(d, &app.selected_links)
    } else {
        vec![Line::styled(
            "No decision selected",
            Style::default().fg(Color::DarkGray),
        )]
    };

    let detail = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Detail "))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    f.render_widget(detail, area);
}

fn build_detail_lines(d: &crate::model::Decision, links: &[crate::model::Link]) -> Vec<Line<'static>> {
    let label = |name: &str| {
        Span::styled(
            format!("{}: ", name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    };

    let mut lines = vec![
        Line::from(vec![label("ID"), Span::raw(d.id.clone())]),
        Line::from(vec![
            label("Title"),
            Span::styled(
                d.title.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![label("Kind"), Span::raw(d.kind.to_string())]),
        Line::from(vec![label("Weight"), Span::raw(d.weight.to_string())]),
        Line::from(vec![label("Level"), Span::raw(d.level.to_string())]),
        Line::from(vec![label("Status"), Span::raw(d.status.to_string())]),
        Line::from(vec![label("Author"), Span::raw(d.author.clone())]),
        Line::from(vec![label("Created"), Span::raw(d.created_at.clone())]),
        Line::from(vec![label("Updated"), Span::raw(d.updated_at.clone())]),
    ];

    if let Some(ref scope) = d.scope {
        lines.push(Line::from(vec![label("Scope"), Span::raw(scope.clone())]));
    }

    if let Some(ref rebuttal) = d.rebuttal {
        lines.push(Line::from(vec![
            label("Rebuttal"),
            Span::raw(rebuttal.clone()),
        ]));
    }

    if let Some(ref superseded_by) = d.superseded_by {
        lines.push(Line::from(vec![
            label("Superseded by"),
            Span::raw(superseded_by.clone()),
        ]));
    }

    if !d.labels.is_empty() {
        lines.push(Line::from(vec![
            label("Labels"),
            Span::raw(d.labels.join(", ")),
        ]));
    }

    if let Some(ref body) = d.body {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![label("Body")]));
        for line in body.lines() {
            lines.push(Line::raw(line.to_string()));
        }
    }

    if !links.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![Span::styled(
            "Links:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));

        for link in links {
            let (arrow, other_id) = if link.source_id == d.id {
                ("->", &link.target_id)
            } else {
                ("<-", &link.source_id)
            };
            let mut spans = vec![
                Span::raw(format!("  {} ", arrow)),
                Span::styled(
                    link.kind.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(format!(" {}", other_id)),
            ];
            if let Some(ref reason) = link.reason {
                spans.push(Span::styled(
                    format!(" ({})", reason),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    lines
}

fn draw_search_view(f: &mut Frame, app: &App, area: Rect) {
    // Draw the list in the background
    draw_list_view(f, app, area);

    // Overlay search input
    let search_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4).min(60),
        height: 3,
    };

    f.render_widget(Clear, search_area);

    let input = Paragraph::new(Line::from(vec![
        Span::styled("/ ", Style::default().fg(Color::Yellow)),
        Span::raw(app.search_query.clone()),
        Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Search ")
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(input, search_area);
}

fn draw_keybind_bar(f: &mut Frame, app: &App, area: Rect) {
    let hints = match &app.view {
        View::List => vec![
            ("q", "quit"),
            ("j/k", "navigate"),
            ("Enter", "detail"),
            ("Tab", "tree"),
            ("/", "search"),
            ("f", "filter"),
        ],
        View::Tree => vec![
            ("q", "quit"),
            ("j/k", "navigate"),
            ("Space", "expand"),
            ("Enter", "detail"),
            ("Tab", "list"),
            ("/", "search"),
            ("f", "filter"),
        ],
        View::Detail => vec![
            ("q", "quit"),
            ("Esc", "back"),
            ("C-d/C-u", "scroll"),
        ],
        View::Search => vec![
            ("Enter", "search"),
            ("Esc", "cancel"),
        ],
    };

    let spans: Vec<Span> = hints
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::Gray)),
            ]
        })
        .collect();

    let bar = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(bar, area);
}
