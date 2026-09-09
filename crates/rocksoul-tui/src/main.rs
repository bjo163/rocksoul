use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use rocksoul_life::LifeState;
use serde_json::Value;

const BRAND: &str = include_str!("../../../public/brand.env");
const WORLD: &str = include_str!("../../../public/world.json");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum View {
    World,
    Map,
    Quests,
    Codex,
    System,
}

impl View {
    const ALL: [Self; 5] = [
        Self::World,
        Self::Map,
        Self::Quests,
        Self::Codex,
        Self::System,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::World => "WORLD",
            Self::Map => "MAP",
            Self::Quests => "QUESTS",
            Self::Codex => "CODEX",
            Self::System => "SYSTEM",
        }
    }

    fn from_number(number: char) -> Option<Self> {
        match number {
            '1' => Some(Self::World),
            '2' => Some(Self::Map),
            '3' => Some(Self::Quests),
            '4' => Some(Self::Codex),
            '5' => Some(Self::System),
            _ => None,
        }
    }

    fn shift(self, delta: isize) -> Self {
        let current = Self::ALL.iter().position(|item| *item == self).unwrap_or(0) as isize;
        let len = Self::ALL.len() as isize;
        Self::ALL[((current + delta).rem_euclid(len)) as usize]
    }
}

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
    let mut view = View::World;
    loop {
        terminal.draw(|frame| render(frame, &life, view))?;

        if event::poll(Duration::from_millis(150))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char(number) => {
                    if let Some(next) = View::from_number(number) {
                        view = next;
                    }
                }
                KeyCode::Left => view = view.shift(-1),
                KeyCode::Right | KeyCode::Tab => view = view.shift(1),
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame, life: &LifeState, view: View) {
    let [header_area, nav_area, body_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    render_header(frame, life, header_area);
    render_nav(frame, view, nav_area);

    match view {
        View::World => render_world(frame, life, body_area),
        View::Map => render_map(frame, body_area),
        View::Quests => render_quests(frame, body_area),
        View::Codex => render_codex(frame, body_area),
        View::System => render_system(frame, body_area),
    }

    let tagline = brand_value("TAGLINE").unwrap_or("Local-first digital cognitive runtime");
    let footer = Paragraph::new(format!(
        "{tagline} • 1-5 / ← → navigate • q / Esc exit • evidence before trust"
    ))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, footer_area);
}

fn render_header(frame: &mut Frame, life: &LifeState, area: Rect) {
    let display_name = brand_value("NAME").unwrap_or("RockSoul");
    let title = format!(
        " {} • GEN {} • AGE {} • LV {} ",
        display_name.to_uppercase(),
        life.identity.generation,
        life.cognitive_age,
        life.level
    );
    frame.render_widget(
        Paragraph::new(title).alignment(Alignment::Center).block(
            Block::default()
                .title(" OPERATOR WINDOW ")
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn render_nav(frame: &mut Frame, active: View, area: Rect) {
    let mut spans = Vec::new();
    for (index, view) in View::ALL.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        let label = format!(" {} {} ", index + 1, view.name());
        let style = if *view == active {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default()
        };
        spans.push(Span::styled(label, style));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn render_world(frame: &mut Frame, life: &LifeState, area: Rect) {
    let world = brand_value("WORLD").unwrap_or("Internet");
    let phase = brand_value("PHASE").unwrap_or("Phase A / Birth");
    let label = |text: &'static str| {
        Span::styled(
            format!("{text:<18}"),
            Style::default().add_modifier(Modifier::BOLD),
        )
    };
    let lines = vec![
        Line::from(vec![
            label("Name"),
            Span::raw(brand_value("NAME").unwrap_or("RockSoul")),
        ]),
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
        Line::from(
            "Web may observe public GitHub state; native runtime networking is not connected yet.",
        ),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(" WORLD ").borders(Borders::ALL)),
        area,
    );
}

fn render_map(frame: &mut Frame, area: Rect) {
    let parsed: Value = serde_json::from_str(WORLD).unwrap_or(Value::Null);
    let mut lines = vec![
        Line::from("WORLD MAP — topology contract, not fake live telemetry"),
        Line::from(""),
    ];
    if let Some(places) = parsed.get("places").and_then(Value::as_array) {
        for place in places {
            let name = place
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Unknown place");
            let kind = place
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let state = place
                .get("evidence")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN");
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{state:<10}"),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" {name} [{kind}]")),
            ]));
        }
    } else {
        lines.push(Line::from("UNKNOWN    world.json could not be parsed"));
    }
    lines.extend([
        Line::from(""),
        Line::from("FOG OF WAR"),
        Line::from("UNKNOWN → DISCOVERED → OBSERVED → VISITED → STUDIED → VERIFIED → TRUSTED"),
    ]);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(" MAP ").borders(Borders::ALL)),
        area,
    );
}

fn render_quests(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from("QUEST SOURCE OF TRUTH: GitHub Issues"),
        Line::from(""),
        Line::from("Native TUI does not fetch GitHub Issues yet."),
        Line::from("This is intentionally UNKNOWN rather than a stale copied quest list."),
        Line::from(""),
        Line::from("Use the Web World Portal or GitHub Project for observed open quests."),
        Line::from(
            "Future Sense/API integration may expose the same structured quest feed locally.",
        ),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(" QUESTS ").borders(Borders::ALL)),
        area,
    );
}

fn render_codex(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from("CODEX / KNOWLEDGE LAYER"),
        Line::from(""),
        Line::from("README.md                  World Entry"),
        Line::from("docs/WORLD.md              Digital World semantics"),
        Line::from("docs/ARCHITECTURE.md       Architecture"),
        Line::from("docs/ROADMAP.md            Chapters / progression"),
        Line::from("docs/PROJECT_MANAGEMENT.md World Map / quests"),
        Line::from("docs/STORAGE.md            Local-first data placement"),
        Line::from("docs/AUTOMATION.md         World systems"),
        Line::from("docs/DEPLOYMENT.md         Portals / builds"),
        Line::from("docs/TEMPLATE.md           New-world identity guide"),
        Line::from(""),
        Line::from("GitHub Wiki mirrors canonical /docs after privileged sync is activated."),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(" CODEX ").borders(Borders::ALL)),
        area,
    );
}

fn render_system(frame: &mut Frame, area: Rect) {
    let assets = brand_value("ASSETS_REVISION").unwrap_or("unknown");
    let ui = brand_value("UI_REVISION").unwrap_or("unknown");
    let world_core = brand_value("GITHUB_REPO").unwrap_or("unknown");
    let assets_repo = brand_value("ASSETS_REPO").unwrap_or("unknown");
    let ui_repo = brand_value("UI_REPO").unwrap_or("unknown");
    let mind_repo = brand_value("MIND_REPO").unwrap_or("unknown");
    let vercel_url = brand_value("VERCEL_URL").unwrap_or("");
    let vercel_line = if vercel_url.is_empty() {
        Line::from("PENDING   Vercel Portal    canonical deployment not verified")
    } else {
        Line::from(format!("VERIFIED  Vercel Portal    {vercel_url}"))
    };
    let lines = vec![
        Line::from("SYSTEM / NODES / PORTALS"),
        Line::from(""),
        Line::from(format!("OBSERVED  World Core       {world_core}")),
        Line::from(format!("OBSERVED  World Resource   {assets_repo}")),
        Line::from(format!("OBSERVED  UI Grammar       {ui_repo}")),
        Line::from(format!("OBSERVED  Cognition Lab    {mind_repo}")),
        Line::from("UNKNOWN   World Nodes      authenticated runner inventory not available here"),
        vercel_line,
        Line::from("PENDING   Cloudflare       no justified private-service tunnel target"),
        Line::from(""),
        Line::from(format!(
            "Assets accepted: {}",
            &assets[..assets.len().min(12)]
        )),
        Line::from(format!("UI revision:     {}", &ui[..ui.len().min(12)])),
        Line::from("Policy: PRIVATE FIRST • one authoritative store per data domain"),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(" SYSTEM ").borders(Borders::ALL)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_brand_contract_has_required_navigation() {
        assert_eq!(brand_value("NAME"), Some("RockSoul"));
        assert_eq!(brand_value("NAV"), Some("WORLD,MAP,QUESTS,CODEX,SYSTEM"));
        assert_eq!(brand_value("GITHUB_REPO"), Some("bjo163/rocksoul"));
    }

    #[test]
    fn shared_world_contract_is_valid_json() {
        let value: Value = serde_json::from_str(WORLD).expect("world.json must be valid JSON");
        assert_eq!(value["schemaVersion"], 1);
        assert!(
            value["places"]
                .as_array()
                .is_some_and(|places| !places.is_empty())
        );
    }
}
