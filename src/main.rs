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
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Application state
    let mut split_horizontal = false;

    let res = run_app(&mut terminal, &mut split_horizontal);

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
    split_horizontal: &mut bool,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, *split_horizontal))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') => *split_horizontal = true,
                KeyCode::Char('v') => *split_horizontal = false,
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, split_horizontal: bool) {
    // Determine split direction
    let direction = if split_horizontal {
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

    // Left/Top pane
    let block1 = Block::default()
        .title("Pane 1")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    let text1 = vec![
        Line::from("This is the first pane"),
        Line::from(""),
        Line::from("Controls:"),
        Line::from("'v' - Vertical split"),
        Line::from("'h' - Horizontal split"),
        Line::from("'q' - Quit"),
    ];
    let paragraph1 = Paragraph::new(text1)
        .block(block1)
        .style(Style::default().fg(Color::Green));
    f.render_widget(paragraph1, chunks[0]);

    // Right/Bottom pane
    let block2 = Block::default()
        .title("Pane 2")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    let text2 = vec![
        Line::from("This is the second pane"),
        Line::from(""),
        Line::from(format!("Current split: {}", 
            if split_horizontal { "Horizontal" } else { "Vertical" }
        )),
    ];
    let paragraph2 = Paragraph::new(text2)
        .block(block2)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(paragraph2, chunks[1]);
}