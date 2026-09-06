use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use crate::cli::Cli;
use crate::{config, distro, env_profile, hardware, history, ollama, safety};

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Idle,
    Thinking,
    ShowResult,
    Error(String),
}

struct App {
    input: String,
    cursor_pos: usize,
    result: Option<String>,
    result_safe: bool,
    explanation: Option<String>,
    history_display: Vec<(String, String)>,
    state: AppState,
    scroll: u16,
    cli: Cli,
    execute_requested: bool,
}

impl App {
    fn new(cli: Cli) -> Result<Self> {
        let hist = history::load()?;
        let history_display = hist
            .iter()
            .rev()
            .take(50)
            .map(|e| (e.query.clone(), e.command.clone()))
            .collect();

        Ok(Self {
            input: String::new(),
            cursor_pos: 0,
            result: None,
            result_safe: true,
            explanation: None,
            history_display,
            state: AppState::Idle,
            scroll: 0,
            cli,
            execute_requested: false,
        })
    }

    fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    fn delete_char_back(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.input[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input.drain(prev..self.cursor_pos);
            self.cursor_pos = prev;
        }
    }
}

pub async fn run_interactive(cli: &Cli) -> Result<(Option<String>, bool, Option<String>)> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let cli_clone = Cli {
        query: cli.query.clone(),
        model: cli.model.clone(),
        explain: cli.explain,
        ollama_url: cli.ollama_url.clone(),
        interactive: cli.interactive,
        execute: cli.execute,
        command: None,
        file: cli.file.clone(),
        role: cli.role.clone(),
    };

    let mut app = App::new(cli_clone)?;
    let _ = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if app.execute_requested {
        Ok((app.result.clone(), app.result_safe, app.explanation.clone()))
    } else {
        Ok((None, false, None))
    }
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Çıkış
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        if app.state != AppState::Idle {
                            app.state = AppState::Idle;
                            app.result = None;
                        } else {
                            return Ok(());
                        }
                    }

                    // Enter — sorgu gönder
                    KeyCode::Enter if app.state == AppState::Idle => {
                        if !app.input.trim().is_empty() {
                            let query = app.input.trim().to_string();
                            app.state = AppState::Thinking;
                            terminal.draw(|f| render(f, app))?;

                            // Sorguyu işle
                            match process_query(app, &query).await {
                                Ok((cmd, safe, expl)) => {
                                    // Geçmişe ekle
                                    let _ = history::append(&query, &cmd);
                                    app.history_display.insert(0, (query.clone(), cmd.clone()));
                                    app.history_display.truncate(50);
                                    app.result = Some(cmd);
                                    app.result_safe = safe;
                                    app.explanation = expl;
                                    app.state = AppState::ShowResult;
                                    app.input.clear();
                                    app.cursor_pos = 0;
                                }
                                Err(e) => {
                                    app.state = AppState::Error(e.to_string());
                                }
                            }
                        }
                    }

                    // Execute (Çalıştır) komutu
                    KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) && app.state == AppState::ShowResult => {
                        app.execute_requested = true;
                        return Ok(());
                    }

                    // Yeni sorgu (sonuç ekranından)
                    KeyCode::Enter if app.state == AppState::ShowResult => {
                        app.state = AppState::Idle;
                        app.result = None;
                    }

                    // Karakter girişi
                    KeyCode::Char(c) if app.state == AppState::Idle => {
                        app.insert_char(c);
                    }

                    // Backspace
                    KeyCode::Backspace if app.state == AppState::Idle => {
                        app.delete_char_back();
                    }

                    // Scroll
                    KeyCode::Up => app.scroll = app.scroll.saturating_sub(1),
                    KeyCode::Down => app.scroll = app.scroll.saturating_add(1),

                    _ => {}
                }
            }
        }
    }
}

async fn process_query(
    app: &App,
    query: &str,
) -> Result<(String, bool, Option<String>)> {
    let config = config::load()?;
    let distro = distro::detect();
    let hw = hardware::detect();
    let env = env_profile::load_or_detect()?;
    let hist = history::load()?;

    let model = app.cli.model
        .clone()
        .unwrap_or(config.model.default);

    let system_prompt = crate::build_system_prompt(&distro, &hw, &env, &hist, app.cli.explain);

    let result = ollama::generate(
        &app.cli.ollama_url,
        &model,
        query,
        &system_prompt,
        app.cli.explain,
    ).await?;

    let is_safe = !safety::is_dangerous(&result.command);
    Ok((result.command, is_safe, result.explanation))
}

fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Ana layout: header | body | footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Body
            Constraint::Length(3),  // Input
        ])
        .split(size);

    // ── Header ──────────────────────────────────────────────
    let title = Paragraph::new(
        Line::from(vec![
            Span::styled("👻 LocalGhost", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::DarkGray)),
            Span::styled("  |  Ctrl+C: çıkış  |  Enter: gönder  |  Esc: temizle",
                Style::default().fg(Color::DarkGray)),
        ])
    )
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // ── Body ─────────────────────────────────────────────────
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[1]);

    // Sol: Sonuç / Bekleme / Hoşgeldin
    let main_content = match &app.state {
        AppState::Idle => {
            let text = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Doğal dilde bir komut açıklaması yazın →",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Örnekler:",
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                )),
                Line::from(Span::styled(
                    "    sistemi güncelle",
                    Style::default().fg(Color::Green),
                )),
                Line::from(Span::styled(
                    "    500MB'dan büyük dosyaları bul",
                    Style::default().fg(Color::Green),
                )),
                Line::from(Span::styled(
                    "    disk kullanımını göster",
                    Style::default().fg(Color::Green),
                )),
                Line::from(Span::styled(
                    "    ekran kartı sıcaklığını göster",
                    Style::default().fg(Color::Green),
                )),
            ]);
            Paragraph::new(text)
                .block(Block::default()
                    .title(" Komut Üretici ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)))
                .wrap(Wrap { trim: true })
        }

        AppState::Thinking => {
            Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  ⠙ Düşünüyor...",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
            ]))
            .block(Block::default()
                .title(" Komut Üretici ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)))
        }

        AppState::ShowResult => {
            let cmd = app.result.as_deref().unwrap_or("");
            let (color, label) = if app.result_safe {
                (Color::Green, "✓ GÜVENLİ")
            } else {
                (Color::Red, "⚠ TEHLİKELİ")
            };

            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {} ", label),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {}", cmd),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            if let Some(expl) = &app.explanation {
                lines.push(Line::from(Span::styled(
                    "  📖 Açıklama:",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                for line in expl.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("     {}", line),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  [Enter] yeni sorgu  |  [Esc] temizle",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )));

            Paragraph::new(Text::from(lines))
                .block(Block::default()
                    .title(" Sonuç ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(color)))
                .wrap(Wrap { trim: true })
        }

        AppState::Error(msg) => {
            Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  ✗ Hata: {}", msg),
                    Style::default().fg(Color::Red),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Ollama çalışıyor mu? ollama serve",
                    Style::default().fg(Color::DarkGray),
                )),
            ]))
            .block(Block::default()
                .title(" Hata ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red)))
        }
    };
    f.render_widget(main_content, body_chunks[0]);

    // Sağ: Geçmiş
    let hist_items: Vec<ListItem> = app
        .history_display
        .iter()
        .take(20)
        .map(|(q, cmd)| {
            ListItem::new(vec![
                Line::from(Span::styled(
                    format!("  {}", q),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!("  → {}", cmd),
                    Style::default().fg(Color::Green),
                )),
                Line::from(""),
            ])
        })
        .collect();

    let hist_list = List::new(hist_items)
        .block(Block::default()
            .title(" Geçmiş ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(hist_list, body_chunks[1]);

    // ── Input ────────────────────────────────────────────────
    let input_style = match app.state {
        AppState::Idle => Style::default().fg(Color::White),
        AppState::Thinking => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::DarkGray),
    };

    let input_text = if app.state == AppState::Thinking {
        "  ⠙ Düşünüyor...".to_string()
    } else {
        format!("  {}", app.input)
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default()
            .title(" Sorgunuz ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(input_style));
    f.render_widget(input, chunks[2]);

    // Cursor göster
    if app.state == AppState::Idle {
        f.set_cursor_position((
            chunks[2].x + app.cursor_pos as u16 + 3,
            chunks[2].y + 1,
        ));
    }
}
