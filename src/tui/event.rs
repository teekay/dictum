use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::error::Result;

use super::app::{App, View};

/// Handle input events. Returns true if the app should quit.
pub fn handle_events(app: &mut App) -> Result<bool> {
    if !event::poll(std::time::Duration::from_millis(100))? {
        return Ok(false);
    }

    if let Event::Key(key) = event::read()? {
        if key.kind != event::KeyEventKind::Press {
            return Ok(false);
        }

        match &app.view {
            View::Search => return handle_search_keys(app, key),
            _ => {}
        }

        if app.filter_panel_open {
            return handle_filter_keys(app, key);
        }

        return handle_normal_keys(app, key);
    }

    Ok(false)
}

fn handle_normal_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),

        KeyCode::Char('j') | KeyCode::Down => app.move_selection(1),
        KeyCode::Char('k') | KeyCode::Up => app.move_selection(-1),

        KeyCode::Char('g') => app.move_selection(-(app.item_count() as i32)),
        KeyCode::Char('G') => app.move_selection(app.item_count() as i32),

        KeyCode::Enter => {
            if !app.decisions.is_empty() || !app.tree_nodes.is_empty() {
                app.view = View::Detail;
                app.detail_scroll = 0;
            }
        }

        KeyCode::Esc => {
            if app.view == View::Detail {
                // Go back to whichever list view was active before
                if app.tree_nodes.is_empty() || app.expanded_nodes.is_empty() {
                    app.view = View::List;
                } else {
                    app.view = View::List;
                }
                // Restore previous view context
            }
        }

        KeyCode::Tab => {
            match app.view {
                View::List => {
                    app.view = View::Tree;
                    app.selected_index = 0;
                    app.load_selected_decision();
                }
                View::Tree => {
                    app.view = View::List;
                    app.selected_index = 0;
                    app.load_selected_decision();
                }
                _ => {}
            }
        }

        KeyCode::Char('/') => {
            app.view = View::Search;
            app.search_query.clear();
        }

        KeyCode::Char('f') => {
            app.filter_panel_open = !app.filter_panel_open;
        }

        KeyCode::Char(' ') => {
            if app.view == View::Tree {
                app.toggle_tree_node();
            }
        }

        // Scroll detail view
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.view == View::Detail {
                app.detail_scroll = app.detail_scroll.saturating_add(10);
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.view == View::Detail {
                app.detail_scroll = app.detail_scroll.saturating_sub(10);
            }
        }

        _ => {}
    }
    Ok(false)
}

fn handle_filter_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),

        KeyCode::Char('f') | KeyCode::Esc => {
            app.filter_panel_open = false;
        }

        KeyCode::Char('1') => {
            app.filter.cycle_field(0);
            app.refresh_list()?;
        }
        KeyCode::Char('2') => {
            app.filter.cycle_field(1);
            app.refresh_list()?;
        }
        KeyCode::Char('3') => {
            app.filter.cycle_field(2);
            app.refresh_list()?;
        }
        KeyCode::Char('4') => {
            app.filter.cycle_field(3);
            app.refresh_list()?;
        }

        // Clear all filters
        KeyCode::Char('0') => {
            app.filter = super::app::FilterState::default();
            app.refresh_list()?;
        }

        _ => {}
    }
    Ok(false)
}

fn handle_search_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.view = View::List;
            app.search_query.clear();
            // Restore full list
            app.refresh_list()?;
        }

        KeyCode::Enter => {
            app.search()?;
            app.view = View::List;
        }

        KeyCode::Backspace => {
            app.search_query.pop();
        }

        KeyCode::Char(c) => {
            app.search_query.push(c);
        }

        _ => {}
    }
    Ok(false)
}
