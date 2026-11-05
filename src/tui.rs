use std::{
    io::{self, IsTerminal},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    memory::{MemoryClaimStatus, MemoryKind, MemorySearchFilters, MemoryStore},
    model::ModelConfig,
    paths::AppPaths,
    rid::{RuhProfile, StatId},
    rid_catalogue::{CatalogueEntry, RidCatalogue},
    rid_store, task_trace,
    tui_preferences::{DensityMode, LocaleMode, PREFERENCES_FILE, ThemeMode, TuiPreferences},
};

#[derive(Clone, Copy)]
struct Palette {
    background: Color,
    panel: Color,
    muted: Color,
    text: Color,
    accent: Color,
    success: Color,
    error: Color,
    border: Color,
}

impl Palette {
    fn for_theme(theme: ThemeMode) -> Self {
        match theme {
            ThemeMode::Dark => Self {
                background: Color::Rgb(15, 17, 18),
                panel: Color::Rgb(25, 29, 30),
                muted: Color::Rgb(129, 139, 140),
                text: Color::Rgb(226, 224, 215),
                accent: Color::Rgb(219, 166, 74),
                success: Color::Rgb(98, 190, 165),
                error: Color::Rgb(214, 104, 93),
                border: Color::Rgb(66, 75, 76),
            },
            ThemeMode::Light => Self {
                background: Color::Rgb(242, 240, 232),
                panel: Color::Rgb(255, 253, 246),
                muted: Color::Rgb(94, 102, 100),
                text: Color::Rgb(34, 39, 39),
                accent: Color::Rgb(154, 101, 27),
                success: Color::Rgb(25, 112, 86),
                error: Color::Rgb(174, 49, 43),
                border: Color::Rgb(188, 182, 167),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Screen {
    Overview,
    Brains,
    Tasks,
    Memory,
    Rid,
    Tools,
    Diagnostics,
    Settings,
}

impl Screen {
    const ALL: [Self; 8] = [
        Self::Overview,
        Self::Brains,
        Self::Tasks,
        Self::Memory,
        Self::Rid,
        Self::Tools,
        Self::Diagnostics,
        Self::Settings,
    ];

    fn index(self) -> usize {
        Self::ALL.iter().position(|item| *item == self).unwrap_or(0)
    }

    fn from_index(index: usize) -> Self {
        Self::ALL[index % Self::ALL.len()]
    }

    fn label(self, locale: LocaleMode) -> &'static str {
        match (self, locale) {
            (Self::Overview, LocaleMode::English) => "OVERVIEW",
            (Self::Brains, LocaleMode::English) => "BRAINS",
            (Self::Tasks, LocaleMode::English) => "TASKS",
            (Self::Memory, LocaleMode::English) => "MEMORY",
            (Self::Rid, LocaleMode::English) => "RID",
            (Self::Tools, LocaleMode::English) => "TOOLS",
            (Self::Diagnostics, LocaleMode::English) => "DIAGNOSTICS",
            (Self::Settings, LocaleMode::English) => "SETTINGS",
            (Self::Overview, LocaleMode::Indonesian) => "RINGKASAN",
            (Self::Brains, LocaleMode::Indonesian) => "OTAK",
            (Self::Tasks, LocaleMode::Indonesian) => "TUGAS",
            (Self::Memory, LocaleMode::Indonesian) => "MEMORI",
            (Self::Rid, LocaleMode::Indonesian) => "RID",
            (Self::Tools, LocaleMode::Indonesian) => "ALAT",
            (Self::Diagnostics, LocaleMode::Indonesian) => "DIAGNOSTIK",
            (Self::Settings, LocaleMode::Indonesian) => "PENGATURAN",
        }
    }

    fn title(self, locale: LocaleMode) -> &'static str {
        match (self, locale) {
            (Self::Overview, LocaleMode::English) => "Command Deck",
            (Self::Brains, LocaleMode::English) => "Brain Registry",
            (Self::Tasks, LocaleMode::English) => "Turn Monitor",
            (Self::Memory, LocaleMode::English) => "Memory & Experience",
            (Self::Rid, LocaleMode::English) => "Ruh Information",
            (Self::Tools, LocaleMode::English) => "Tools & Permissions",
            (Self::Diagnostics, LocaleMode::English) => "Runtime Diagnostics",
            (Self::Settings, LocaleMode::English) => "Presentation Settings",
            (Self::Overview, LocaleMode::Indonesian) => "Pusat Kendali",
            (Self::Brains, LocaleMode::Indonesian) => "Registri Otak",
            (Self::Tasks, LocaleMode::Indonesian) => "Monitor Tugas",
            (Self::Memory, LocaleMode::Indonesian) => "Memori & Pengalaman",
            (Self::Rid, LocaleMode::Indonesian) => "Informasi Ruh",
            (Self::Tools, LocaleMode::Indonesian) => "Alat & Izin",
            (Self::Diagnostics, LocaleMode::Indonesian) => "Diagnostik Runtime",
            (Self::Settings, LocaleMode::Indonesian) => "Pengaturan Tampilan",
        }
    }
}

struct MemoryStoreHealth {
    schema_version: u16,
    active_records: usize,
    total_records: usize,
}

struct MemoryView {
    store: MemoryStore,
    health: MemoryStoreHealth,
}

struct TaskTraceView {
    event: String,
    operation_id: String,
    target_name: String,
    timestamp_unix_ms: Option<u64>,
}

impl TaskTraceView {
    fn from_value(value: &serde_json::Value) -> Self {
        let target = safe_trace_text(value.get("target"), 96);
        Self {
            event: safe_trace_text(value.get("event"), 24),
            operation_id: safe_trace_text(value.get("operation_id"), 64),
            target_name: target.rsplit('/').next().unwrap_or("target").to_owned(),
            timestamp_unix_ms: value
                .get("timestamp_unix_ms")
                .and_then(serde_json::Value::as_u64),
        }
    }
}

struct Snapshot {
    paths: AppPaths,
    model_configured: bool,
    rid_profile: Option<RuhProfile>,
    rid_error: Option<String>,
    catalogue: Option<RidCatalogue>,
    evaluation: std::result::Result<Option<crate::eval::EvaluationReport>, String>,
    memory_evaluation:
        std::result::Result<Option<crate::eval::MemoryRetrievalEvaluationReport>, String>,
    memory: std::result::Result<MemoryView, String>,
    task_history: std::result::Result<Vec<TaskTraceView>, String>,
    preferences: std::result::Result<TuiPreferences, String>,
}

impl Snapshot {
    fn load() -> Result<Self> {
        let paths = AppPaths::discover()?;
        let (rid_profile, rid_error) = if paths.rid_file.is_file() {
            match rid_store::load(&paths.rid_file) {
                Ok(profile) => (Some(profile), None),
                Err(error) => (None, Some(format!("{error:#}"))),
            }
        } else {
            (None, None)
        };
        let preferences = TuiPreferences::load(&paths.config_dir.join(PREFERENCES_FILE))
            .map_err(|_| "preferences are invalid or unsupported".to_owned());
        let memory = MemoryStore::load(&paths.memory_dir)
            .map(|store| MemoryView {
                health: MemoryStoreHealth {
                    schema_version: store.schema_version,
                    active_records: store.active_records(now_unix()).len(),
                    total_records: store.records.len(),
                },
                store,
            })
            .map_err(|_| "memory store is unreadable".to_owned());
        let task_history = task_trace::read_recent(
            &paths.logs_dir.join("task-trace.v1.jsonl"),
            262_144,
            task_event_limit(preferences.as_ref().map(|value| value.density).unwrap_or_default()),
        )
        .map(|records| records.iter().map(TaskTraceView::from_value).collect())
        .map_err(|_| "task trace is malformed, oversized or unavailable".to_owned());
        Ok(Self {
            paths,
            model_configured: ModelConfig::from_env().is_some(),
            rid_profile,
            rid_error,
            catalogue: RidCatalogue::embedded().ok(),
            evaluation: crate::eval::latest()
                .map_err(|_| "evaluation report is unreadable".to_owned()),
            memory_evaluation: crate::eval::latest_memory_retrieval()
                .map_err(|_| "memory retrieval report is unreadable".to_owned()),
            memory,
            task_history,
            preferences,
        })
    }

    fn rated_stats(&self) -> usize {
        self.rid_profile
            .as_ref()
            .map(|profile| {
                profile
                    .stats()
                    .iter()
                    .filter(|stat| stat.score.is_some())
                    .count()
            })
            .unwrap_or(0)
    }
}

struct App {
    screen: Screen,
    selected_stat: usize,
    rid_state: ListState,
    palette_open: bool,
    help_open: bool,
    quit: bool,
    snapshot: Snapshot,
    preferences: TuiPreferences,
    preferences_error: bool,
    palette: Palette,
    memory_query: String,
    memory_search_active: bool,
    memory_detail_open: bool,
    memory_selected: usize,
    task_selected: usize,
    memory_kind_filter: Option<MemoryKind>,
    memory_claim_filter: Option<MemoryClaimStatus>,
}

impl App {
    fn new(snapshot: Snapshot) -> Self {
        let preferences = snapshot.preferences.clone().unwrap_or_default();
        let preferences_error = snapshot.preferences.is_err();
        let palette = Palette::for_theme(preferences.theme);
        let mut rid_state = ListState::default();
        rid_state.select(Some(0));
        Self {
            screen: Screen::Overview,
            selected_stat: 0,
            rid_state,
            palette_open: false,
            help_open: false,
            quit: false,
            snapshot,
            preferences,
            preferences_error,
            palette,
            memory_query: String::new(),
            memory_search_active: false,
            memory_detail_open: false,
            memory_selected: 0,
            task_selected: 0,
            memory_kind_filter: None,
            memory_claim_filter: None,
        }
    }

    fn select_screen(&mut self, screen: Screen) {
        self.screen = screen;
        self.palette_open = false;
        self.help_open = false;
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.memory_search_active {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => self.memory_search_active = false,
                KeyCode::Backspace => {
                    self.memory_query.pop();
                }
                KeyCode::Char(character)
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && self.memory_query.chars().count() < 64 =>
                {
                    self.memory_query.push(character);
                }
                _ => {}
            }
            return;
        }
        if self.palette_open || self.help_open {
            match key.code {
                KeyCode::Esc => {
                    self.palette_open = false;
                    self.help_open = false;
                }
                KeyCode::Char(c @ '1'..='8') => {
                    self.select_screen(Screen::from_index((c as u8 - b'1') as usize))
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc if self.screen == Screen::Memory && self.memory_detail_open => {
                self.memory_detail_open = false
            }
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Char(c @ '1'..='8') => {
                self.select_screen(Screen::from_index((c as u8 - b'1') as usize))
            }
            KeyCode::Tab => self.select_screen(Screen::from_index(self.screen.index() + 1)),
            KeyCode::BackTab => self.select_screen(Screen::from_index(self.screen.index() + 7)),
            KeyCode::Up if self.screen == Screen::Rid => self.move_stat(-1),
            KeyCode::Down if self.screen == Screen::Rid => self.move_stat(1),
            KeyCode::Up if self.screen == Screen::Memory => {
                self.memory_detail_open = false;
                self.memory_selected = self.memory_selected.saturating_sub(1)
            }
            KeyCode::Up if self.screen == Screen::Tasks => {
                self.task_selected = self.task_selected.saturating_sub(1)
            }
            KeyCode::Down if self.screen == Screen::Tasks => {
                self.task_selected = self.task_selected.saturating_add(1).min(11)
            }
            KeyCode::Down if self.screen == Screen::Memory => {
                self.memory_detail_open = false;
                self.memory_selected = self.memory_selected.saturating_add(1).min(99)
            }
            KeyCode::Char('/') if self.screen == Screen::Memory => self.memory_search_active = true,
            KeyCode::Enter if self.screen == Screen::Memory => {
                self.memory_detail_open = !self.memory_detail_open
            }
            KeyCode::Char('f') if self.screen == Screen::Memory => {
                self.memory_kind_filter = next_memory_kind(self.memory_kind_filter)
            }
            KeyCode::Char('t') if self.screen == Screen::Settings => {
                self.preferences.theme = match self.preferences.theme {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                self.persist_preferences();
            }
            KeyCode::Char('d') if self.screen == Screen::Settings => {
                self.preferences.density = match self.preferences.density {
                    DensityMode::Compact => DensityMode::Comfortable,
                    DensityMode::Comfortable => DensityMode::Compact,
                };
                self.persist_preferences();
            }
            KeyCode::Char('l') if self.screen == Screen::Settings => {
                self.preferences.locale = match self.preferences.locale {
                    LocaleMode::English => LocaleMode::Indonesian,
                    LocaleMode::Indonesian => LocaleMode::English,
                };
                self.persist_preferences();
            }
            KeyCode::Char('c') if self.screen == Screen::Memory => {
                self.memory_claim_filter = next_memory_claim_status(self.memory_claim_filter)
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.palette_open = true
            }
            KeyCode::Char('?') => self.help_open = true,
            KeyCode::Char('r') => {
                if let Ok(snapshot) = Snapshot::load() {
                    self.preferences = snapshot.preferences.clone().unwrap_or_default();
                    self.preferences_error = snapshot.preferences.is_err();
                    self.palette = Palette::for_theme(self.preferences.theme);
                    self.snapshot = snapshot;
                }
            }
            _ => {}
        }
    }

    fn persist_preferences(&mut self) {
        let path = self.snapshot.paths.config_dir.join(PREFERENCES_FILE);
        self.preferences_error = self.preferences.save(&path).is_err();
        self.palette = Palette::for_theme(self.preferences.theme);
    }

    fn move_stat(&mut self, delta: isize) {
        let count = self
            .snapshot
            .catalogue
            .as_ref()
            .map(|catalogue| catalogue.entries.len())
            .unwrap_or(0);
        if count == 0 {
            return;
        }
        self.selected_stat =
            (self.selected_stat as isize + delta).rem_euclid(count as isize) as usize;
        self.rid_state.select(Some(self.selected_stat));
    }
}

pub fn run() -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!(
            "rocksoul tui requires an interactive terminal; use CLI commands for scripts and pipes"
        );
    }

    let snapshot = Snapshot::load().context("could not load the cockpit snapshot")?;
    ratatui::run(|terminal| -> io::Result<()> {
        let mut app = App::new(snapshot);
        loop {
            terminal.draw(|frame| render(frame, &mut app))?;
            if app.quit {
                break;
            }
            if event::poll(Duration::from_millis(180))? {
                if let Event::Key(key) = event::read()? {
                    app.handle_key(key);
                }
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn render(frame: &mut Frame<'_>, app: &mut App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(app.palette.background)), area);
    if area.width < 80 || area.height < 24 {
        render_small(frame, area, app);
        return;
    }

    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(area);
    render_header(frame, shell[0], app);
    render_navigation(frame, shell[1], app);
    render_body(frame, shell[2], app);
    render_footer(frame, shell[3], app);
    if app.palette_open {
        render_palette(frame, shell[2], app);
    }
    if app.help_open {
        render_help(frame, shell[2], app);
    }
}

fn panel(title: &'static str, palette: &Palette) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(palette.accent).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.panel).fg(palette.text))
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let english = app.preferences.locale == LocaleMode::English;
    let rid = app
        .snapshot
        .rid_profile
        .as_ref()
        .map(|profile| format!("RID L{} · {} XP", profile.level(), profile.total_xp()))
        .unwrap_or_else(|| {
            if english {
                "RID NOT INITIALIZED".to_owned()
            } else {
                "RID BELUM DIINISIALISASI".to_owned()
            }
        });
    let status = if app.snapshot.rid_error.is_some() {
        (if english { "DEGRADED" } else { "TERGANGGU" }, app.palette.error)
    } else {
        (if english { "LOCAL RUNTIME" } else { "RUNTIME LOKAL" }, app.palette.success)
    };
    let line = Line::from(vec![
        Span::styled(
            " ROCKSOUL ",
            Style::default().fg(app.palette.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if english { "/ COGNITIVE COCKPIT " } else { "/ KOKPIT KOGNITIF " },
            Style::default()
                .fg(app.palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(status.0, Style::default().fg(status.1)),
        Span::raw("   "),
        Span::styled(rid, Style::default().fg(app.palette.muted)),
    ]);
    frame.render_widget(
        Paragraph::new(line)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(app.palette.accent)),
            )
            .style(Style::default().bg(app.palette.panel)),
        area,
    );
}

fn render_navigation(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let tabs = Screen::ALL
        .iter()
        .enumerate()
        .map(|(index, screen)| {
            let label = format!(" {} {} ", index + 1, screen.label(app.preferences.locale));
            if *screen == app.screen {
                Span::styled(
                    label,
                    Style::default()
                        .fg(app.palette.background)
                        .bg(app.palette.accent)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(label, Style::default().fg(app.palette.muted))
            }
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(Line::from(tabs)).block(Block::default().borders(Borders::BOTTOM)),
        area,
    );
}

fn render_body(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if area.width >= 116 {
            vec![Constraint::Percentage(68), Constraint::Percentage(32)]
        } else {
            vec![Constraint::Percentage(100)]
        })
        .split(area);
    match app.screen {
        Screen::Overview => render_overview(frame, columns[0], app),
        Screen::Brains => render_brains(frame, columns[0], app),
        Screen::Tasks => render_tasks(frame, columns[0], app),
        Screen::Memory => render_memory(frame, columns[0], app),
        Screen::Rid => render_rid(frame, columns[0], app),
        Screen::Tools => render_tools(frame, columns[0], app),
        Screen::Diagnostics => render_diagnostics(frame, columns[0], app),
        Screen::Settings => render_settings(frame, columns[0], app),
    }
    if columns.len() > 1 {
        render_inspector(frame, columns[1], app);
    }
}

fn render_overview(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(6)])
        .margin(1)
        .split(area);
    let model = if app.snapshot.model_configured {
        ("CONFIGURED", app.palette.success)
    } else {
        ("NOT CONFIGURED", app.palette.accent)
    };
    let rid = if app.snapshot.rid_profile.is_some() {
        ("INITIALIZED", app.palette.success)
    } else {
        ("NOT INITIALIZED", app.palette.accent)
    };
    let summary = vec![
        Line::from(vec![
            Span::styled("MODEL ", Style::default().fg(app.palette.muted)),
            Span::styled(model.0, Style::default().fg(model.1)),
            Span::raw("    "),
            Span::styled("RID ", Style::default().fg(app.palette.muted)),
            Span::styled(rid.0, Style::default().fg(rid.1)),
        ]),
        Line::from(""),
        Line::from("KERNEL  TURN STATE MACHINE AVAILABLE"),
        Line::from("ACTIVE TASK  NONE PERSISTED"),
    ];
    frame.render_widget(
        Paragraph::new(summary).block(panel("SYSTEM SNAPSHOT", &app.palette)),
        chunks[0],
    );
    let lines = vec![
        Line::from(Span::styled(
            "RUNTIME MAP",
            Style::default().fg(app.palette.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from("• One local-first runtime; CLI, REPL and TUI share application services."),
        Line::from("• Verified task lifecycle: Planning → Executing → Verifying → Completed."),
        Line::from("• RID identity, XP ledger and 99 evidence-based capability slots."),
        Line::from(""),
        Line::from(Span::styled(
            "NOT IMPLEMENTED YET",
            Style::default().fg(app.palette.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(
            "• Persistent task queue, Memory OS, tool registry and specialist model routing.",
        ),
        Line::from("• Native LLM API and controlled learning/evolution pipeline."),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel("SYSTEM MAP", &app.palette)),
        chunks[1],
    );
}

fn render_brains(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let adapter = if app.snapshot.model_configured {
        "configured"
    } else {
        "not configured"
    };
    let lines = vec![
        Line::from(Span::styled(
            "AUTOMATIC SPECIALIST REGISTRY",
            Style::default().fg(app.palette.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(
            "Local-first is fixed runtime policy; backend choice is not a top-level user mode.",
        ),
        Line::from(""),
        Line::from(format!(
            "Language / Action     legacy chat adapter: {adapter}"
        )),
        Line::from("Reflex                not assigned"),
        Line::from("Reasoning / Planning  not assigned"),
        Line::from("Decision              not assigned"),
        Line::from("Perception            not assigned"),
        Line::from("Embedding / Rerank    not assigned"),
        Line::from(""),
        Line::from("Hardware-aware specialist routing is not implemented yet."),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel("BRAINS / RUNTIME", &app.palette)),
        area,
    );
}

fn render_tasks(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let lines = match &app.snapshot.task_history {
        Ok(records) if records.is_empty() => {
            app.task_selected = 0;
            vec![
                Line::from(Span::styled(
                    "NO TASK EVENTS RECORDED",
                    Style::default().fg(app.palette.muted).add_modifier(Modifier::BOLD),
                )),
                Line::from("Verified workspace operations will appear here after execution."),
                Line::from("No queue or active task is inferred from an empty trace."),
            ]
        }
        Ok(records) => {
            app.task_selected = app.task_selected.min(records.len().saturating_sub(1));
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("PERSISTED TASK TRACE · {} RECENT EVENTS", records.len()),
                    Style::default().fg(app.palette.accent).add_modifier(Modifier::BOLD),
                )),
                Line::from("Newest events first · ↑/↓ select · content-free metadata only"),
                Line::from(""),
            ];
            for (index, record) in records.iter().enumerate() {
                let timestamp = record
                    .timestamp_unix_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "time n/a".to_owned());
                lines.push(Line::from(format!(
                    "{} {timestamp}  {:<18}  {:<12}  {}",
                    if index == app.task_selected { "▸" } else { " " },
                    record.event,
                    clip_text(&record.operation_id, 12),
                    record.target_name
                )));
            }
            let selected = &records[app.task_selected];
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "SELECTED EVENT",
                Style::default().fg(app.palette.accent).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!("Event       {}", selected.event)));
            lines.push(Line::from(format!("Operation   {}", selected.operation_id)));
            lines.push(Line::from(format!("Target      {}", selected.target_name)));
            lines.push(Line::from(format!(
                "Timestamp   {}",
                selected
                    .timestamp_unix_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "n/a".to_owned())
            )));
            lines
        }
        Err(_) => vec![
            Line::from(Span::styled(
                "TASK TRACE UNAVAILABLE",
                Style::default().fg(app.palette.error).add_modifier(Modifier::BOLD),
            )),
            Line::from("The trace is malformed, unsupported, oversized or inaccessible."),
            Line::from("No task state is inferred from invalid records."),
        ],
    };
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel("TASK / TURN MONITOR", &app.palette)),
        area,
    );
}

fn safe_trace_text(value: Option<&serde_json::Value>, limit: usize) -> String {
    let Some(value) = value.and_then(serde_json::Value::as_str) else {
        return "n/a".to_owned();
    };
    let cleaned = value
        .chars()
        .filter(|character| !character.is_control())
        .take(limit)
        .collect::<String>();
    if cleaned.is_empty() {
        "n/a".to_owned()
    } else {
        cleaned
    }
}

fn render_memory(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let max_records = memory_record_limit(app.preferences.density);
    let lines = match &app.snapshot.memory {
        Ok(memory) => {
            let store = &memory.store;
            let active = store.active_records(now_unix());
            let mut lines = vec![
                Line::from(format!("Store schema         v{}", memory.health.schema_version)),
                Line::from(format!(
                    "Active records       {}/{}",
                    memory.health.active_records, memory.health.total_records
                )),
                Line::from(format!(
                    "Store                {}",
                    app.snapshot.paths.memory_dir.join("store.v4.json").display()
                )),
                Line::from(format!(
                    "Search               {}{}",
                    if app.memory_search_active { ">" } else { "" },
                    if app.memory_query.is_empty() {
                        "press / to search"
                    } else {
                        &app.memory_query
                    }
                )),
                Line::from(format!(
                    "Filters              kind={:?} · claim={:?} · source:<exact>",
                    app.memory_kind_filter, app.memory_claim_filter
                )),
                Line::from(""),
                Line::from("ID             KIND       CLAIM STATUS             SOURCE                       ACCESS"),
            ];
            let source_filter = app
                .memory_query
                .split_whitespace()
                .find_map(|token| token.strip_prefix("source:"));
            let text_query = app
                .memory_query
                .split_whitespace()
                .filter(|token| !token.starts_with("source:"))
                .collect::<Vec<_>>()
                .join(" ");
            if text_query.is_empty() {
                let mut records = active
                    .into_iter()
                    .filter(|record| {
                        app.memory_kind_filter.is_none_or(|kind| record.kind == kind)
                            && app
                                .memory_claim_filter
                                .is_none_or(|status| record.claim_status == status)
                    })
                    .collect::<Vec<_>>();
                if let Some(source) = source_filter {
                    records.retain(|record| record.source == source);
                }
                records.sort_by_key(|record| std::cmp::Reverse(record.created_at_unix));
                if records.is_empty() {
                    lines.push(Line::from("No active memory records match these filters."));
                }
                for (index, record) in records.iter().take(max_records).enumerate() {
                    lines.push(Line::from(format!(
                        "{} {:<12} {:<10} {:<24} {:<26} {}",
                        if index == app.memory_selected { "▸" } else { " " },
                        clip_text(&record.id, 12),
                        record.kind,
                        record.claim_status,
                        clip_text(&record.source, 26),
                        if record.user_approved { "approved" } else { "unapproved" }
                    )));
                    lines.push(Line::from(format!(
                        "  created={} · expires={}",
                        record.created_at_unix,
                        record
                            .expires_at_unix
                            .map(|timestamp| timestamp.to_string())
                            .unwrap_or_else(|| "never".to_owned())
                    )));
                }
                if app.memory_detail_open {
                    if let Some(record) =
                        records.get(app.memory_selected.min(records.len().saturating_sub(1)))
                    {
                        push_memory_detail(&mut lines, &record.content);
                    }
                } else if !records.is_empty() {
                    lines.push(Line::from(
                        "Press Enter to reveal the selected record after the privacy guard.",
                    ));
                }
                if records.len() > max_records {
                    lines.push(Line::from(format!(
                        "Showing newest {max_records} of {} matching records.",
                        records.len()
                    )));
                }
            } else {
                let filters = MemorySearchFilters {
                    kind: app.memory_kind_filter,
                    source: source_filter.map(str::to_owned),
                    ..MemorySearchFilters::default()
                };
                let results = store
                    .search_ranked(&text_query, &filters, now_unix(), max_records * 4)
                    .into_iter()
                    .filter(|record| {
                        app.memory_claim_filter
                            .is_none_or(|status| record.claim_status == status)
                    })
                    .take(max_records)
                    .collect::<Vec<_>>();
                if results.is_empty() {
                    lines.push(Line::from("No active records matched this local search."));
                }
                for (index, record) in results.iter().enumerate() {
                    lines.push(Line::from(format!(
                        "{} {:<12} {:<10} {:<24} {:<26} {}",
                        if index == app.memory_selected { "▸" } else { " " },
                        clip_text(&record.record_id, 12),
                        record.kind,
                        record.claim_status,
                        clip_text(&record.source, 26),
                        record
                            .verification_receipt_id
                            .as_deref()
                            .map(|_| "receipt")
                            .unwrap_or("no receipt")
                    )));
                    let expires = store
                        .records
                        .iter()
                        .find(|stored| stored.id == record.record_id)
                        .and_then(|stored| stored.expires_at_unix)
                        .map(|timestamp| timestamp.to_string())
                        .unwrap_or_else(|| "never / n/a".to_owned());
                    lines.push(Line::from(format!(
                        "  created={} · expires={expires}",
                        record.created_at_unix
                    )));
                }
                if app.memory_detail_open {
                    if let Some(record) = results
                        .get(app.memory_selected.min(results.len().saturating_sub(1)))
                    {
                        push_memory_detail(&mut lines, &record.content);
                    }
                } else if !results.is_empty() {
                    lines.push(Line::from(
                        "Press Enter to reveal the selected record after the privacy guard.",
                    ));
                }
                lines.push(Line::from(
                    "Search inspects local record content; content itself remains hidden by default.",
                ));
            }
            lines.push(Line::from(""));
            lines.push(Line::from("Read-only inspector · no memory writes or forgetting actions."));
            lines
        }
        Err(_) => vec![
            Line::from(Span::styled(
                "MEMORY STORE UNAVAILABLE",
                Style::default().fg(app.palette.error).add_modifier(Modifier::BOLD),
            )),
            Line::from("The store is missing, corrupt, unsupported, or inaccessible."),
            Line::from("No memory content is exposed in this error state."),
        ],
    };
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel(
                "MEMORY / EXPERIENCE · ↑/↓ SELECT · / SEARCH · f KIND · c CLAIM",
                &app.palette,
            )),
        area,
    );
}

fn push_memory_detail(lines: &mut Vec<Line<'static>>, content: &str) {
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "SELECTED MEMORY CONTENT",
        Style::default().fg(app.palette.accent).add_modifier(Modifier::BOLD),
    )));
    match memory_detail_display(content) {
        Ok(content) => {
            lines.push(Line::from(content));
            lines.push(Line::from(
                "Content truncated to 512 characters; control characters removed.",
            ));
        }
        Err(reason) => lines.push(Line::from(format!(
            "Content blocked by privacy guard: {reason}"
        ))),
    }
}

fn task_event_limit(density: DensityMode) -> usize {
    match density {
        DensityMode::Compact => 12,
        DensityMode::Comfortable => 8,
    }
}

fn memory_record_limit(density: DensityMode) -> usize {
    match density {
        DensityMode::Compact => 40,
        DensityMode::Comfortable => 20,
    }
}

fn memory_detail_display(content: &str) -> Result<String, &'static str> {
    if let Some(reason) = crate::memory_proposal::sensitive_content_reason(content) {
        Err(reason)
    } else {
        Ok(clip_text(content, 512))
    }
}

fn next_memory_kind(current: Option<MemoryKind>) -> Option<MemoryKind> {
    let values = [
        None,
        Some(MemoryKind::Episodic),
        Some(MemoryKind::Semantic),
        Some(MemoryKind::Procedural),
        Some(MemoryKind::Working),
        Some(MemoryKind::Derived),
    ];
    let index = values
        .iter()
        .position(|value| *value == current)
        .unwrap_or(0);
    values[(index + 1) % values.len()]
}

fn next_memory_claim_status(current: Option<MemoryClaimStatus>) -> Option<MemoryClaimStatus> {
    let values = [
        None,
        Some(MemoryClaimStatus::UserAssertion),
        Some(MemoryClaimStatus::ObservedEvidence),
        Some(MemoryClaimStatus::Inference),
        Some(MemoryClaimStatus::UnverifiedTaskOutcome),
        Some(MemoryClaimStatus::VerifiedTaskOutcome),
        Some(MemoryClaimStatus::ContradictedSuperseded),
    ];
    let index = values
        .iter()
        .position(|value| *value == current)
        .unwrap_or(0);
    values[(index + 1) % values.len()]
}

fn clip_text(value: &str, limit: usize) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(limit)
        .collect()
}

fn render_rid(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let Some(catalogue) = app.snapshot.catalogue.as_ref() else {
        frame.render_widget(
            Paragraph::new("RID catalogue unavailable. No score or label is displayed.")
                .block(panel("RID ERROR", &app.palette)),
            area,
        );
        return;
    };
    let metrics = if let Some(profile) = &app.snapshot.rid_profile {
        vec![
            Line::from(vec![
                Span::styled("AGE ", Style::default().fg(app.palette.muted)),
                Span::styled(
                    format_age(profile.age_seconds_at(now_unix())),
                    Style::default().fg(app.palette.text),
                ),
                Span::raw("   "),
                Span::styled("LEVEL ", Style::default().fg(app.palette.muted)),
                Span::styled(profile.level().to_string(), Style::default().fg(app.palette.accent)),
                Span::raw("   "),
                Span::styled("XP ", Style::default().fg(app.palette.muted)),
                Span::styled(profile.total_xp().to_string(), Style::default().fg(app.palette.text)),
            ]),
            Line::from(format!(
                "RATED {}/99 · CATALOGUE {} · UNRATED IS UNKNOWN, NOT ZERO",
                app.snapshot.rated_stats(),
                catalogue.catalogue_version
            )),
        ]
    } else {
        vec![Line::from(
            "RID profile not initialized. Run rocksoul init first.",
        )]
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(area);
    frame.render_widget(
        Paragraph::new(metrics).block(panel("RUH INFORMATION", &app.palette)),
        chunks[0],
    );

    let profile = app.snapshot.rid_profile.as_ref();
    let items = catalogue
        .entries
        .iter()
        .map(|entry| {
            let score = profile
                .and_then(|p| StatId::new(entry.slot_id).ok().and_then(|id| p.stat(id)))
                .and_then(|record| record.score)
                .map(|value| format!("{value:>3}/100"))
                .unwrap_or_else(|| "unrated".to_owned());
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:02} ", entry.slot_id), Style::default().fg(app.palette.muted)),
                Span::styled(
                    format!("{:<18}", entry.name.transliteration),
                    Style::default().fg(app.palette.text),
                ),
                Span::styled(
                    format!("{:<25}", entry.technical_stat.label),
                    Style::default().fg(app.palette.muted),
                ),
                Span::styled(score, Style::default().fg(app.palette.accent)),
            ]))
        })
        .collect::<Vec<_>>();
    let list = List::new(items)
        .block(panel("99 CAPABILITY DIMENSIONS", &app.palette))
        .highlight_symbol("▸ ")
        .highlight_style(
            Style::default()
                .fg(app.palette.text)
                .bg(Color::Rgb(51, 55, 52))
                .add_modifier(Modifier::BOLD),
        );
    frame.render_stateful_widget(list, chunks[1], &mut app.rid_state);
}

fn render_tools(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lines = vec![
        Line::from(Span::styled(
            "TOOL REGISTRY",
            Style::default().fg(app.palette.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from("No tools are registered for execution in this seed runtime."),
        Line::from("Shell execution     disabled"),
        Line::from("Generated code      not executable"),
        Line::from("Filesystem writes   no tool adapter"),
        Line::from("Network egress      no tool adapter"),
        Line::from(""),
        Line::from(
            "Future tools require least privilege, sandboxing, approval policy and audit evidence.",
        ),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel("TOOLS / PERMISSIONS", &app.palette)),
        area,
    );
}

fn render_diagnostics(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rid = if app.snapshot.rid_profile.is_some() {
        "valid"
    } else if app.snapshot.rid_error.is_some() {
        "invalid"
    } else {
        "not initialized"
    };
    let lines = vec![
        Line::from(format!("Version       {}", env!("CARGO_PKG_VERSION"))),
        Line::from(format!(
            "Platform      {} / {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )),
        Line::from(format!(
            "Config dir    {}",
            app.snapshot.paths.config_dir.display()
        )),
        Line::from(format!(
            "Data dir      {}",
            app.snapshot.paths.data_dir.display()
        )),
        Line::from(format!(
            "Model config  {}",
            if app.snapshot.model_configured {
                "configured (secrets masked)"
            } else {
                "not configured"
            }
        )),
        Line::from(format!("RID profile   {rid}")),
        Line::from(format!(
            "RID detail    {}",
            app.snapshot
                .rid_error
                .as_deref()
                .map(safe_rid_error)
                .unwrap_or("none")
        )),
        Line::from(format!(
            "RID catalogue {}",
            app.snapshot
                .catalogue
                .as_ref()
                .map(|c| c.catalogue_version.as_str())
                .unwrap_or("unavailable")
        )),
        Line::from(format!(
            "Latest eval   {}",
            match &app.snapshot.evaluation {
                Ok(Some(report)) => format!(
                    "{} [{}] · {} v{} · {}/{}",
                    if report.passed { "PASS" } else { "FAIL" },
                    report_state(&app.snapshot.evaluation),
                    report.suite_id,
                    report.suite_version,
                    report.passed_count(),
                    report.checks.len()
                ),
                Ok(None) => "not run".to_owned(),
                Err(_) => "corrupt / unavailable".to_owned(),
            }
        )),
        Line::from(""),
        Line::from(match &app.snapshot.evaluation {
            Ok(Some(report)) if !report.passed => format!(
                "Failed checks: {}",
                report
                    .checks
                    .iter()
                    .filter(|check| !check.passed)
                    .map(|check| check.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Ok(Some(_)) => "Failed checks: none".to_owned(),
            Ok(None) => "Evaluation report: not run".to_owned(),
            Err(_) => "Evaluation report: corrupt or unavailable".to_owned(),
        }),
        Line::from(format!(
            "Memory eval   {}",
            match &app.snapshot.memory_evaluation {
                Ok(Some(report)) => format!(
                    "{} [{}] · {} v{} · P@{} {:.3} · R@{} {:.3}",
                    if report.passed { "PASS" } else { "FAIL" },
                    report_state(&app.snapshot.memory_evaluation),
                    report.suite_id,
                    report.suite_version,
                    report.top_k,
                    report.precision_at_k,
                    report.top_k,
                    report.recall_at_k
                ),
                Ok(None) => "not run".to_owned(),
                Err(_) => "corrupt / unavailable".to_owned(),
            }
        )),
        Line::from(match &app.snapshot.memory_evaluation {
            Ok(Some(report)) => format!(
                "Retrieval leakage: stale={} expired={} deleted={}",
                report.stale_results_at_k, report.expired_leakage, report.deletion_leakage
            ),
            Ok(None) => "Memory retrieval report: not run".to_owned(),
            Err(_) => "Memory retrieval report: corrupt or unavailable".to_owned(),
        }),
        Line::from(match &app.snapshot.memory_store_health {
            Ok(health) => format!(
                "Memory store  v{} · active {}/{}",
                health.schema_version, health.active_records, health.total_records
            ),
            Err(_) => "Memory store  corrupt / unavailable".to_owned(),
        }),
        Line::from("No synthetic latency, token, GPU, RAM or activity telemetry is shown."),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel("DIAGNOSTICS / EVENTS", &app.palette)),
        area,
    );
}

fn render_settings(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let english = app.preferences.locale == LocaleMode::English;
    let theme_label = if english { "Theme" } else { "Tema" };
    let density_label = if english { "Density" } else { "Kepadatan" };
    let language_label = if english { "Language" } else { "Bahasa" };
    let lines = vec![
        Line::from(format!("{theme_label}  {:?}", app.preferences.theme)),
        Line::from(format!("{density_label}  {:?}", app.preferences.density)),
        Line::from(if english {
            "Keymap         keyboard-first"
        } else {
            "Tombol         keyboard-first"
        }),
        Line::from(if english {
            "Motion         reduced"
        } else {
            "Gerak          dikurangi"
        }),
        Line::from(format!("{language_label}  {:?}", app.preferences.locale)),
        Line::from(""),
        Line::from(if english {
            "t Theme · d Density · l Language · saved to portable config"
        } else {
            "t Tema · d Kepadatan · l Bahasa · tersimpan di konfigurasi portable"
        }),
        Line::from(if app.preferences_error {
            if english {
                "Preferences invalid/unsupported; existing file is preserved."
            } else {
                "Preferensi tidak valid/tidak didukung; file lama dipertahankan."
            }
        } else if english {
            "Preferences loaded and version-checked."
        } else {
            "Preferensi dimuat dan versi telah diperiksa."
        }),
        Line::from(if english {
            "Presentation settings cannot change routing, tool permissions or remote data-egress policy."
        } else {
            "Pengaturan tampilan tidak dapat mengubah routing, izin alat, atau kebijakan data-egress."
        }),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel("SETTINGS", &app.palette)),
        area,
    );
}

fn render_inspector(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let mut lines = vec![
        Line::from(Span::styled(
            "INSPECTOR",
            Style::default().fg(app.palette.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("SCREEN  {}", app.screen.label(app.preferences.locale))),
        Line::from(format!(
            "MODEL   {}",
            if app.snapshot.model_configured {
                "configured"
            } else {
                "not configured"
            }
        )),
    ];
    if app.screen == Screen::Rid {
        if let Some(entry) = app
            .snapshot
            .catalogue
            .as_ref()
            .and_then(|cat| cat.entries.get(app.selected_stat))
        {
            lines.extend(rid_detail(entry, app.snapshot.rid_profile.as_ref()));
        }
    } else {
        lines.push(Line::from("No active selection."));
        lines.push(Line::from("Views are read-only in this increment."));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel("SYSTEM INSPECTOR", &app.palette)),
        area,
    );
}

fn rid_detail(entry: &CatalogueEntry, profile: Option<&RuhProfile>) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            entry.name.transliteration.clone(),
            Style::default().fg(app.palette.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(entry.name.arabic.clone()),
        Line::from(entry.name.gloss.id.clone()),
        Line::from(""),
        Line::from(Span::styled(
            entry.technical_stat.label.clone(),
            Style::default().fg(app.palette.accent),
        )),
        Line::from(entry.technical_stat.key.clone()),
        Line::from(entry.technical_stat.definition.clone()),
        Line::from(""),
        Line::from(format!("Domain: {}", entry.technical_stat.domain_label)),
        Line::from(format!(
            "Samples required: {}",
            entry.technical_stat.minimum_samples
        )),
        Line::from(format!("Rubric: {}", entry.technical_stat.score_rubric)),
        Line::from(""),
        Line::from("Inspirational label only; this metric does not measure a divine attribute."),
    ];
    if let Some(profile) = profile {
        if let Ok(id) = StatId::new(entry.slot_id) {
            if let Some(stat) = profile.stat(id) {
                lines.push(Line::from(format!(
                    "Score: {}",
                    stat.score
                        .map(|v| format!("{v}/100"))
                        .unwrap_or_else(|| "unrated".into())
                )));
                lines.push(Line::from(format!(
                    "Confidence: {}",
                    stat.confidence
                        .map(|v| format!("{v}%"))
                        .unwrap_or_else(|| "n/a".into())
                )));
                lines.push(Line::from(format!(
                    "Evidence refs: {}",
                    stat.evidence_count()
                )));
            }
        }
    }
    lines
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let english = app.preferences.locale == LocaleMode::English;
    let text = if app.screen == Screen::Settings {
        if english {
            "t Theme   d Density   l Language   r Refresh   q Exit"
        } else {
            "t Tema   d Kepadatan   l Bahasa   r Segarkan   q Keluar"
        }
    } else if app.screen == Screen::Rid {
        if english {
            "↑/↓ Browse stats   1-8 Navigate   Ctrl+K Palette   ? Help   q Exit"
        } else {
            "↑/↓ Jelajah statistik   1-8 Navigasi   Ctrl+K Palet   ? Bantuan   q Keluar"
        }
    } else if english {
        "1-8 Navigate   / Search   f Kind   c Claim   ↑/↓ Select   Enter/Esc Close Search   q Exit"
    } else {
        "1-8 Navigasi   / Cari   f Jenis   c Klaim   ↑/↓ Pilih   Enter/Esc Tutup Pencarian   q Keluar"
    };
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(app.palette.muted))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(app.palette.border)),
            ),
        area,
    );
}

fn render_palette(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let popup = centered_rect(54, 68, area);
    frame.render_widget(Clear, popup);
    let rows = Screen::ALL
        .iter()
        .enumerate()
        .map(|(i, screen)| ListItem::new(format!("{}  {}", i + 1, screen.title(app.preferences.locale))))
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(rows).block(panel("COMMAND PALETTE · PRESS 1-8", &app.palette)),
        popup,
    );
}

fn render_help(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let popup = centered_rect(60, 62, area);
    frame.render_widget(Clear, popup);
    let rows = vec![
        Line::from("1..8       Open a screen"),
        Line::from("Tab       Next screen"),
        Line::from("Shift+Tab Previous screen"),
        Line::from("Up/Down   Browse RID slots"),
        Line::from("Ctrl+K    Command palette"),
        Line::from("r         Refresh local snapshot"),
        Line::from("Esc / q   Exit cockpit"),
        Line::from("Press Esc to close this panel."),
    ];
    frame.render_widget(
        Paragraph::new(rows)
            .block(panel("KEYBOARD HELP", &app.palette))
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_small(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let english = app.preferences.locale == LocaleMode::English;
    let rows = vec![
        Line::from(Span::styled(
            if english {
                "ROCKSOUL / COGNITIVE COCKPIT"
            } else {
                "ROCKSOUL / KOKPIT KOGNITIF"
            },
            Style::default()
                .fg(app.palette.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(if english {
            "Terminal is below the recommended 80×24 size."
        } else {
            "Terminal lebih kecil dari ukuran rekomendasi 80×24."
        }),
        Line::from(if english {
            "Resize terminal or use CLI commands."
        } else {
            "Perbesar terminal atau gunakan perintah CLI."
        }),
        Line::from(if english {
            "Press q or Esc to exit."
        } else {
            "Tekan q atau Esc untuk keluar."
        }),
    ];
    frame.render_widget(
        Paragraph::new(rows)
            .alignment(Alignment::Center)
            .block(panel("TERMINAL TOO SMALL", &app.palette)),
        area,
    );
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height) / 2),
            Constraint::Percentage(height),
            Constraint::Percentage((100 - height) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width) / 2),
            Constraint::Percentage(width),
            Constraint::Percentage((100 - width) / 2),
        ])
        .split(vertical[1])[1]
}

fn format_age(seconds: u64) -> String {
    let years = seconds / 31_557_600;
    let days = (seconds % 31_557_600) / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if years > 0 {
        format!("{years}y {days}d {hours}h")
    } else if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        format!("{seconds}s")
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{App, LocaleMode, Palette, Screen, Snapshot, ThemeMode};

    #[test]
    fn all_screens_render_at_standard_wide_and_narrow_sizes() {
        use ratatui::{Terminal, backend::TestBackend};

        let mut app = App::new(Snapshot::load().unwrap());
        for theme in [ThemeMode::Dark, ThemeMode::Light] {
            app.preferences.theme = theme;
            app.palette = Palette::for_theme(theme);
            for locale in [LocaleMode::English, LocaleMode::Indonesian] {
                app.preferences.locale = locale;
                for (width, height) in [(80, 24), (120, 40), (60, 20)] {
                    let backend = TestBackend::new(width, height);
                    let mut terminal = Terminal::new(backend).unwrap();
                    for index in 0..8 {
                        app.select_screen(Screen::from_index(index));
                        terminal
                            .draw(|frame| super::render(frame, &mut app))
                            .unwrap();
                    }
                    app.palette_open = true;
                    terminal
                        .draw(|frame| super::render(frame, &mut app))
                        .unwrap();
                    app.palette_open = false;
                    app.help_open = true;
                    terminal
                        .draw(|frame| super::render(frame, &mut app))
                        .unwrap();
                    app.help_open = false;
                }
            }
        }
    }

    #[test]
    fn rid_diagnostic_redacts_paths_and_raw_error_details() {
        let detail = super::safe_rid_error("invalid JSON at C:/Users/example/secret/rid.json");
        assert_eq!(detail, "invalid profile data");
        assert!(!detail.contains("secret"));
        assert_eq!(
            super::safe_rid_error("unsupported schema version 99"),
            "unsupported schema/version"
        );
    }

    #[test]
    fn report_state_distinguishes_missing_from_unreadable_without_leaking_errors() {
        assert_eq!(
            super::report_state(&Ok::<Option<u8>, String>(None)),
            "not run"
        );
        assert_eq!(
            super::report_state(&Err::<Option<u8>, _>(
                "C:/private/path/secret.json".to_owned()
            )),
            "corrupt / unavailable"
        );
        assert_eq!(
            super::report_state(&Ok::<Option<u8>, String>(Some(1))),
            "available"
        );
    }

    #[test]
    fn memory_detail_display_blocks_sensitive_content_and_clips_safe_text() {
        assert_eq!(
            super::memory_detail_display("api_key=example-secret"),
            Err("credential_or_secret")
        );
        let safe = super::memory_detail_display("safe\u{001b}[31m memory note").unwrap();
        assert!(!safe.chars().any(char::is_control));
        assert!(safe.chars().count() <= 512);
    }

    #[test]
    fn memory_detail_uses_shared_sensitive_content_guard() {
        assert_eq!(
            crate::memory_proposal::sensitive_content_reason("api_key=example-secret"),
            Some("credential_or_secret")
        );
        assert_eq!(
            crate::memory_proposal::sensitive_content_reason("my medical diagnosis is private"),
            Some("sensitive_personal_category")
        );
    }

    #[test]
    fn keyboard_focus_and_memory_detail_transitions_are_explicit() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::new(Snapshot::load().unwrap());
        app.screen = Screen::Tasks;
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.task_selected, 1);

        app.screen = Screen::Memory;
        app.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.memory_search_active);
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        assert_eq!(app.memory_query, "x");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!app.memory_search_active);

        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.memory_detail_open);
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.memory_detail_open);
        assert!(!app.quit);
    }

    #[test]
    fn navigation_labels_are_localized() {
        assert_eq!(Screen::Memory.label(LocaleMode::English), "MEMORY");
        assert_eq!(Screen::Memory.label(LocaleMode::Indonesian), "MEMORI");
        assert_eq!(
            Screen::Settings.title(LocaleMode::Indonesian),
            "Pengaturan Tampilan"
        );
    }

    #[test]
    fn light_and_dark_palettes_have_distinct_semantic_tokens() {
        let dark = Palette::for_theme(ThemeMode::Dark);
        let light = Palette::for_theme(ThemeMode::Light);
        assert_ne!(dark.background, light.background);
        assert_ne!(dark.text, light.text);
        assert_ne!(dark.accent, light.accent);
    }

    #[test]
    fn density_preferences_bound_task_and_memory_lists() {
        assert_eq!(super::task_event_limit(DensityMode::Compact), 12);
        assert_eq!(super::task_event_limit(DensityMode::Comfortable), 8);
        assert_eq!(super::memory_record_limit(DensityMode::Compact), 40);
        assert_eq!(super::memory_record_limit(DensityMode::Comfortable), 20);
    }

    #[test]
    fn memory_filters_cycle_through_known_values_and_reset() {
        assert_eq!(
            super::next_memory_kind(None),
            Some(crate::memory::MemoryKind::Episodic)
        );
        assert_eq!(
            super::next_memory_kind(Some(crate::memory::MemoryKind::Derived)),
            None
        );
        assert_eq!(
            super::next_memory_claim_status(None),
            Some(crate::memory::MemoryClaimStatus::UserAssertion)
        );
        assert_eq!(
            super::next_memory_claim_status(Some(
                crate::memory::MemoryClaimStatus::ContradictedSuperseded
            )),
            None
        );
    }

    #[test]
    fn memory_metadata_clipping_removes_controls_and_bounds_length() {
        let text = super::clip_text("source\u{001b}[31m-private", 10);
        assert!(!text.chars().any(char::is_control));
        assert!(text.chars().count() <= 10);
    }

    #[test]
    fn trace_metadata_is_control_free_and_bounded() {
        let value = serde_json::json!("safe\u{001b}[31m-secret");
        let text = super::safe_trace_text(Some(&value), 8);
        assert!(!text.chars().any(char::is_control));
        assert!(text.chars().count() <= 8);
    }

    #[test]
    fn navigation_selects_each_screen_and_resets_overlays() {
        let mut app = App::new(Snapshot::load().unwrap());
        app.palette_open = true;
        app.select_screen(Screen::Rid);
        assert_eq!(app.screen, Screen::Rid);
        assert!(!app.palette_open);
    }

    #[test]
    fn navigation_wraps_through_screen_set() {
        let mut app = App::new(Snapshot::load().unwrap());
        app.select_screen(Screen::Settings);
        assert_eq!(app.screen, Screen::Settings);
        app.select_screen(Screen::Overview);
        assert_eq!(app.screen, Screen::Overview);
    }
}
