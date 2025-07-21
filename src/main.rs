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
    Frame, Terminal,
};
use std::{
    error::Error, 
    io,
    process::Command,
    time::{Duration, Instant},
};

struct Pane {
    title: String,
    command: String,
    output: String,
    last_update: Instant,
    scroll_offset: u16,
}

impl Pane {
    fn new(title: &str, command: &str) -> Self {
        Self {
            title: title.to_string(),
            command: command.to_string(),
            output: String::new(),
            last_update: Instant::now() - Duration::from_secs(60), // Force initial update
            scroll_offset: 0,
        }
    }

    fn update(&mut self) {
        // Update every 2 seconds
        if self.last_update.elapsed() > Duration::from_secs(2) {
            match Command::new("sh")
                .arg("-c")
                .arg(&self.command)
                .output()
            {
                Ok(output) => {
                    self.output = String::from_utf8_lossy(&output.stdout).to_string();
                    if !output.stderr.is_empty() {
                        self.output.push_str("\n--- STDERR ---\n");
                        self.output.push_str(&String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => {
                    self.output = format!("Error executing command: {}", e);
                }
            }
            self.last_update = Instant::now();
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_down(&mut self) {
        let line_count = self.output.lines().count() as u16;
        if self.scroll_offset < line_count.saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }
}

struct App {
    panes: Vec<Pane>,
    split_horizontal: bool,
    focused_pane: usize,
}

impl App {
    fn new() -> Self {
        Self {
            panes: vec![
                Pane::new("System Info", "date && echo && uname -a && echo && uptime"),
                Pane::new("Process List", "ps aux | head -20"),
            ],
            split_horizontal: false,
            focused_pane: 0,
        }
    }

    fn update(&mut self) {
        for pane in &mut self.panes {
            pane.update();
        }
    }

    fn switch_focus(&mut self) {
        self.focused_pane = (self.focused_pane + 1) % self.panes.len();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Application state
    let mut app = App::new();

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
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        app.update();
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('h') => app.split_horizontal = true,
                    KeyCode::Char('v') => app.split_horizontal = false,
                    KeyCode::Tab => app.switch_focus(),
                    KeyCode::Up => {
                        app.panes[app.focused_pane].scroll_up();
                    }
                    KeyCode::Down => {
                        app.panes[app.focused_pane].scroll_down();
                    }
                    KeyCode::Char('1') => {
                        app.panes[0] = Pane::new("Disk Usage", "df -h");
                    }
                    KeyCode::Char('2') => {
                        app.panes[1] = Pane::new("Network Info", "ifconfig");
                    }
                    KeyCode::Char('3') => {
                        app.panes[0] = Pane::new("Memory Info", "free -h");
                    }
                    KeyCode::Char('4') => {
                        app.panes[1] = Pane::new("CPU Info", "lscpu");
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
            .title(format!("{} | Command: {} {}", 
                pane.title, 
                pane.command,
                if is_focused { "[FOCUSED]" } else { "" }
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(
                if is_focused { Color::Cyan } else { Color::White }
            ));

        let lines: Vec<Line> = pane.output
            .lines()
            .skip(pane.scroll_offset as usize)
            .map(|line| Line::from(line.to_string()))
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(Style::default().fg(if i == 0 { Color::Green } else { Color::Yellow }))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, chunks[i]);
    }

    // Instructions
    let help_text = vec![
        Line::from("q: Quit | v: Vertical | h: Horizontal | Tab: Switch focus | ↑↓: Scroll"),
        Line::from("1: Disk Usage | 2: Network | 3: Memory | 4: CPU Info"),
    ];
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(help, f.area());
}