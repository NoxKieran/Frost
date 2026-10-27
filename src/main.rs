
use std::{
    error::Error,
    io,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};

use sysinfo::System;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let mut system = System::new_all();

    loop {
        system.refresh_cpu_all();
        system.refresh_memory();

        let cpu = system.global_cpu_usage();
        let used_ram = system.used_memory();
        let total_ram = system.total_memory();

        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(7),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            // Header
            let header = Paragraph::new(
                " FROST  /  SYSTEM MONITOR"
            )
            .style(
                Style::default().fg(Color::Cyan)
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
            );

            frame.render_widget(header, layout[0]);

            // CPU and RAM cards
            let cards = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(layout[1]);

            let cpu_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(" CPU ")
                        .borders(Borders::ALL)
                )
                .gauge_style(
                    Style::default()
                    .fg(Color::Gray)
                    .bg(Color::Rgb(30, 41, 59))
                )
                .ratio(
                    (cpu as f64 / 100.0)
                        .clamp(0.0, 1.0)
                )
                .label(format!("{:.1}%", cpu));

            frame.render_widget(cpu_gauge, cards[0]);

            let ram_ratio = if total_ram > 0 {
                used_ram as f64 / total_ram as f64
            } else {
                0.0
            };

            let ram_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(" MEMORY ")
                        .borders(Borders::ALL)
                )
                .gauge_style(
                    Style::default()
                    .fg(Color::Rgb(74, 222, 128))
                    .bg(Color::Rgb(30, 41, 59))
                )
                .ratio(ram_ratio.clamp(0.0, 1.0))
                .label(format!(
                    "{:.1} / {:.1} GiB",
                    used_ram as f64 / 1_073_741_824.0,
                    total_ram as f64 / 1_073_741_824.0
                ));

            frame.render_widget(ram_gauge, cards[1]);

            // System overview
            let overview_text = format!(
                " CPU usage       : {:>5.1}%\n\
                 Memory usage    : {:>5.1}%\n\
                 Memory available: {:.2} GiB",
                cpu,
                ram_ratio * 100.0,
                total_ram.saturating_sub(used_ram)
                    as f64 / 1_073_741_824.0,
            );

            let overview = Paragraph::new(overview_text)
                .block(
                    Block::default()
                        .title(" SYSTEM OVERVIEW ")
                        .borders(Borders::ALL)
                )
                .style(
                    Style::default().fg(Color::Gray)
                );

            frame.render_widget(overview, layout[2]);

            // Footer
            let footer = Paragraph::new(
                " Q: Quit | R: Refresh"
            )
            .style(
                Style::default()
                .fg(Color::Rgb(148, 168, 184))
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
            );

            frame.render_widget(footer, layout[3]);
        })?;

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {
                        system.refresh_cpu_all();
                        system.refresh_memory();
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
