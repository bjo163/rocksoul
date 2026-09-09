use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use rocksoul_life::LifeState;

const BRAND: &str = include_str!("../../../public/brand.env");

fn brand_value(key: &str) -> Option<&'static str> {
    BRAND.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        (name == key).then_some(value)
    })
}

fn main() -> Result<()> {
    let life = LifeState::born_now();
    ratatui::run(|terminal| run(terminal, life))
}

fn run(terminal: &mut DefaultTerminal, life: LifeState) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, &life))?;

        if event::poll(Duration::from_millis(150))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Ok(());
        }
    }
}

fn render(frame: &mut Frame, life: &LifeState) {
    let [header_area, body_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    let display_name = brand_value("NAME").unwrap_or(&life.identity.name);
    let world = brand_value("WORLD").unwrap_or("Internet");
    let phase = brand_value("PHASE").unwrap_or("Phase A / Birth");
    let tagline = brand_value("TAGLINE").unwrap_or("Local-first digital cognitive runtime");

    let title = format!(
        " {} • GEN {} • AGE {} • LV {} ",
        display_name.to_uppercase(),
        life.identity.generation,
        life.cognitive_age,
        life.level
    );

    let header = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, header_area);

    let label = |text: &'static str| {
        Span::styled(
            format!("{text:<18}"),
            Style::default().add_modifier(Modifier::BOLD),
        )
    };

    let body_lines = vec![
        Line::from(vec![label("Name"), Span::raw(display_name)]),
        Line::from(vec![
            label("Born"),
            Span::raw(
                life.identity
                    .born_at
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            ),
        ]),
        Line::from(vec![
            label("Cognitive Age"),
            Span::raw(life.cognitive_age.to_string()),
        ]),
        Line::from(vec![label("Level"), Span::raw(life.level.to_string())]),
        Line::from(vec![label("XP"), Span::raw(life.xp.to_string())]),
        Line::from(vec![label("Trust"), Span::raw(life.trust.to_string())]),
        Line::from(vec![label("World"), Span::raw(world)]),
        Line::from(vec![label("State"), Span::raw(life.status.as_str())]),
        Line::from(""),
        Line::from(format!("{phase} — brain not connected yet.")),
    ];

    let body =
        Paragraph::new(body_lines).block(Block::default().title(" HOME ").borders(Borders::ALL));
    frame.render_widget(body, body_area);

    let footer = Paragraph::new(format!("{tagline} • q / Esc: exit"))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, footer_area);
}
