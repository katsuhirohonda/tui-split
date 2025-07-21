use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal as RatatuiTerminal,
};
use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use tui_split::Terminal;

struct Pane {
    terminal: Terminal,
    output_buffer: Arc<Mutex<Vec<String>>>,
    title: String,
}

impl Pane {
    fn new(title: &str) -> Result<Self> {
        let terminal = Terminal::new()?;
        let output_buffer = Arc::new(Mutex::new(Vec::new()));
        
        Ok(Pane {
            terminal,
            output_buffer,
            title: title.to_string(),
        })
    }

    fn start_reader(&mut self) -> Result<()> {
        let buffer_clone = Arc::clone(&self.output_buffer);
        
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut accumulated = String::new();
            
            loop {
                // This is a simple version - in production, we'd use a proper terminal emulator
                match std::io::Read::read(&mut std::io::stdin(), &mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]);
                        accumulated.push_str(&text);
                        
                        // Simple line-based buffering
                        if accumulated.contains('\n') {
                            let lines: Vec<String> = accumulated.lines().map(String::from).collect();
                            if let Ok(mut buffer) = buffer_clone.lock() {
                                buffer.extend(lines);
                                // Keep only last 100 lines
                                if buffer.len() > 100 {
                                    let drain_count = buffer.len() - 100;
                                    buffer.drain(0..drain_count);
                                }
                            }
                            accumulated.clear();
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        self.terminal.write(data)?;
        Ok(())
    }

    fn get_output(&self) -> Vec<String> {
        self.output_buffer.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

struct App {
    panes: Vec<Pane>,
    split_horizontal: bool,
    focused_pane: usize,
}

impl App {
    fn new() -> Result<Self> {
        Ok(Self {
            panes: vec![
                Pane::new("Terminal 1")?,
                Pane::new("Terminal 2")?,
            ],
            split_horizontal: false,
            focused_pane: 0,
        })
    }

    fn switch_focus(&mut self) {
        self.focused_pane = (self.focused_pane + 1) % self.panes.len();
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = RatatuiTerminal::new(backend)?;

    // Application state
    let mut app = App::new()?;

    let res = run_app(&mut terminal, &mut app);

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut RatatuiTerminal<B>,
    app: &mut App,
) -> io::Result<()> {
    // Start simple - just display output
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('h') => app.split_horizontal = true,
                    KeyCode::Char('v') => app.split_horizontal = false,
                    KeyCode::Tab => app.switch_focus(),
                    KeyCode::Char(c) => {
                        // Send character to focused terminal
                        let _ = app.panes[app.focused_pane].write(&[c as u8]);
                    }
                    KeyCode::Enter => {
                        let _ = app.panes[app.focused_pane].write(b"\n");
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    // Determine split direction
    let direction = if app.split_horizontal {
        Direction::Horizontal
    } else {
        Direction::Vertical
    };

    // Create layout (50% split)
    let chunks = Layout::default()
        .direction(direction)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    // Render each pane
    for (i, pane) in app.panes.iter().enumerate() {
        let is_focused = i == app.focused_pane;
        
        let block = Block::default()
            .title(format!("{} {}", 
                pane.title,
                if is_focused { "[FOCUSED]" } else { "" }
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(
                if is_focused { Color::Cyan } else { Color::White }
            ));

        // For now, show a simple message
        let text = vec![
            Line::from("ZSH Terminal"),
            Line::from(""),
            Line::from("Note: Full terminal emulation requires additional work."),
            Line::from("This is a simplified version for demonstration."),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(if i == 0 { Color::Green } else { Color::Yellow }))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, chunks[i]);
    }

    // Instructions
    let help_text = vec![
        Line::from("q: Quit | v: Vertical | h: Horizontal | Tab: Switch focus"),
        Line::from("Note: Full terminal emulation with ANSI escape sequences is complex"),
    ];
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(help, f.area());
}