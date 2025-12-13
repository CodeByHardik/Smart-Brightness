use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};

use crate::config::{save_config, Config, DaemonMode};

struct App {
    config: Config,
    state: ListState,
    items: Vec<&'static str>,
    edit_mode: bool,
    input_buffer: String,
    status_message: String,
}

impl App {
    fn new(config: Config) -> App {
        let mut state = ListState::default();
        state.select(Some(0));
        App {
            config,
            state,
            items: vec![
                "Daemon Mode",
                "Run Duration (Boot/Interval)",
                "Pause Interval (Interval)",
                "Min Brightness",
                "Max Brightness",
                "Smoothing Factor",
                "Save & Exit",
                "Cancel",
            ],
            edit_mode: false,
            input_buffer: String::new(),
            status_message: String::from("Press 'Enter' to edit, 'q' to quit"),
        }
    }

    fn next(&mut self) {
        if self.edit_mode { return; }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.edit_mode { return; }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn current_value(&self) -> String {
        match self.state.selected().unwrap_or(0) {
            0 => format!("{:?}", self.config.mode),
            1 => format!("{:.1}", self.config.run_duration),
            2 => format!("{:.1}", self.config.pause_interval),
            3 => format!("{}", self.config.real_min_brightness),
            4 => format!("{}", self.config.real_max_brightness),
            5 => format!("{:.2}", self.config.smoothing_factor),
            _ => String::new(),
        }
    }

    fn enter_edit(&mut self) {
        let idx = self.state.selected().unwrap_or(0);
        if idx >= 6 { return; } // Don't edit action buttons
        self.edit_mode = true;
        self.input_buffer = self.current_value();
        self.status_message = String::from("Editing... Press Enter to confirm, Esc to cancel");
    }

    fn submit_edit(&mut self) {
        let idx = self.state.selected().unwrap_or(0);
        match idx {
            0 => {
                // Cycle modes for simplicity if typing is annoying, or parse
                 match self.input_buffer.to_lowercase().as_str() {
                    "boot" => self.config.mode = DaemonMode::Boot,
                    "interval" => self.config.mode = DaemonMode::Interval,
                    "realtime" => self.config.mode = DaemonMode::Realtime,
                    _ => self.status_message = String::from("Invalid mode! Use: boot, interval, realtime"),
                }
            }
            1 => if let Ok(v) = self.input_buffer.parse() { self.config.run_duration = v; },
            2 => if let Ok(v) = self.input_buffer.parse() { self.config.pause_interval = v; },
            3 => if let Ok(v) = self.input_buffer.parse() { self.config.real_min_brightness = v; },
            4 => if let Ok(v) = self.input_buffer.parse() { self.config.real_max_brightness = v; },
            5 => if let Ok(v) = self.input_buffer.parse() { self.config.smoothing_factor = v; },
            _ => {}
        }
        self.edit_mode = false;
        self.status_message = String::from("Value updated. Don't forget to 'Save & Exit'");
    }
}

pub fn run(initial_config: Config) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(initial_config);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if app.edit_mode {
                match key.code {
                    KeyCode::Enter => app.submit_edit(),
                    KeyCode::Esc => {
                        app.edit_mode = false;
                        app.status_message = String::from("Editing cancelled");
                    },
                    KeyCode::Backspace => { app.input_buffer.pop(); },
                    KeyCode::Char(c) => { app.input_buffer.push(c); },
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Enter => {
                        let idx = app.state.selected().unwrap_or(0);
                        match idx {
                            6 => { // Save & Exit
                                if let Err(e) = save_config(&app.config) {
                                    app.status_message = format!("Error saving: {}", e);
                                } else {
                                    return Ok(());
                                }
                            },
                            7 => return Ok(()), // Cancel
                            _ => app.enter_edit(),
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Smart Brightness Configurator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let val = if i < 6 {
                format!(": {}", match i {
                    0 => format!("{:?}", app.config.mode),
                    1 => format!("{:.1}s", app.config.run_duration),
                    2 => format!("{:.1}s", app.config.pause_interval),
                    3 => format!("{}", app.config.real_min_brightness),
                    4 => format!("{}", app.config.real_max_brightness),
                    5 => format!("{:.2}", app.config.smoothing_factor),
                    _ => String::new(),
                })
            } else {
                String::new()
            };

            let content = Line::from(vec![
                Span::raw(format!("{:<30}", item)),
                Span::styled(val, Style::default().fg(Color::Yellow)),
            ]);
            
            ListItem::new(content).style(Style::default().fg(Color::White))
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(items, chunks[1], &mut app.state);

    let help_text = if app.edit_mode {
        format!("EDITING: {} (Current: {})", app.input_buffer, app.current_value())
    } else {
        app.status_message.clone()
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(if app.edit_mode { Color::Red } else { Color::Green }))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
