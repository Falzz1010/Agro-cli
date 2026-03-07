use std::env;
use std::fmt::Write; // Added for efficient string building
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, BorderType, List, ListItem,
        Paragraph, Wrap,
    },
};
use tokio::time::Duration;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::core::{calculate_today_tasks, load_rules, weather, GardenTask};
use crate::hardware::{read_humidity, read_soil_moisture, read_temperature};

const VERSION: &str = "1.0.0";

const MENU_ITEMS: &[&str] = &[
    "🤖 AI Agent Mode",
    "🌱 Check Today's Tasks (Real-Time)",
    "➕ Add New Plant",
    "📊 View Garden Stats",
    "📡 Live Sensor Monitor",
    "✂️  Harvest a Plant",
    "🗑️  Delete a Plant",
    "⚙️  Configure Plant Settings",
    "☁️  Configure Weather API",
    "🔌 Run Daemon Automation",
    "🌐 Start Web Dashboard",
    "❌ Exit",
];

// ── Screen & Action Enums ──────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum Screen {
    MainMenu,
    LiveTasks,
    GardenStats,
    LiveSensor,
    SelectList,
    TextInput,
    Confirm,
    Message,
    WebDashboard,
    AiAgent,
    DaemonMonitor,
}

#[derive(Clone)]
enum Pending {
    None,
    AddType,
    AddName { plant_type: String },
    Harvest,
    DeleteSelect,
    DeleteConfirm { name: String },
    ConfigSelect,
    ConfigInput { name: String },
    WeatherInput,
    AiProviderSelect,
    AiApiKeyInput { provider: crate::ai::AiProvider },
}

struct Field {
    label: String,
    value: String,
    default: String,
}

struct SensorRow {
    name: String,
    moisture: f32,
    temperature: f32,
    humidity: f32,
}

#[derive(Clone)]
struct ChatMessage {
    role: String,  // "user" or "ai"
    text: String,
    timestamp: String,
}

#[derive(Clone)]
struct DaemonLog {
    timestamp: String,
    level: String,  // "info", "success", "warning", "error"
    message: String,
}

// ── Exit signals (to temporarily leave TUI) ────────────────────────

pub enum ExitSignal {
    Quit,
}

// ── App struct ─────────────────────────────────────────────────────

#[allow(clippy::struct_excessive_bools)]
struct App {
    state: crate::web::AppState,
    cancel_token: CancellationToken,
    screen: Screen,
    pending: Pending,
    exit_signal: Option<ExitSignal>,

    // Main menu
    menu_idx: usize,

    // Select list
    sel_items: Vec<String>,
    sel_title: String,
    sel_idx: usize,

    // Text input
    inp_fields: Vec<Field>,
    inp_focus: usize,
    inp_title: String,

    // Confirm
    cfm_msg: String,
    cfm_yes: bool,

    // Message
    msg_title: String,
    msg_body: String,

    // Live data
    tasks: Vec<GardenTask>,
    all_plants: Vec<crate::core::Plant>,  // Store all plants for display
    weather: Option<(String, f32)>,
    weather_last_fetch: Instant,
    stats: serde_json::Value,
    sensors: Vec<SensorRow>,
    last_tick: Instant,

    // Web Dashboard
    web_running: bool,
    web_url: String,
    web_server_started: bool,
    
    // AI Agent
    ai_input: String,
    ai_messages: Vec<ChatMessage>,
    ai_processing: bool,
    ai_provider: Option<crate::ai::AiProvider>,
    ai_api_key: Option<String>,
    
    // Daemon Monitor
    daemon_logs: Vec<DaemonLog>,
    daemon_running: bool,
    daemon_cycle_count: u32,
    
    // Performance: Only redraw when needed
    needs_redraw: bool,
}

impl App {
    fn new(state: crate::web::AppState, cancel_token: CancellationToken) -> Self {
        Self {
            state,
            cancel_token,
            screen: Screen::MainMenu,
            pending: Pending::None,
            exit_signal: None,
            menu_idx: 0,
            sel_items: vec![],
            sel_title: String::new(),
            sel_idx: 0,
            inp_fields: vec![],
            inp_focus: 0,
            inp_title: String::new(),
            cfm_msg: String::new(),
            cfm_yes: false,
            msg_title: String::new(),
            msg_body: String::new(),
            tasks: vec![],
            all_plants: vec![],
            weather: None,
            weather_last_fetch: Instant::now().checked_sub(Duration::from_secs(600)).unwrap_or_else(Instant::now), // Force initial fetch
            stats: serde_json::json!({}),
            sensors: vec![],
            last_tick: Instant::now(),
            web_running: false,
            web_url: String::new(),
            web_server_started: false,
            ai_input: String::new(),
            ai_messages: vec![],
            ai_processing: false,
            ai_provider: None,
            ai_api_key: None,
            daemon_logs: vec![],
            daemon_running: false,
            daemon_cycle_count: 0,
            needs_redraw: true, // Initial draw
        }
    }

    fn go_msg(&mut self, title: &str, body: &str) {
        self.screen = Screen::Message;
        self.msg_title = title.into();
        self.msg_body = body.into();
        self.needs_redraw = true;
    }

    fn go_sel(&mut self, title: &str, items: Vec<String>, action: Pending) {
        self.screen = Screen::SelectList;
        self.sel_title = title.into();
        self.sel_items = items;
        self.sel_idx = 0;
        self.pending = action;
        self.needs_redraw = true;
    }

    fn go_inp(&mut self, title: &str, fields: Vec<Field>, action: Pending) {
        self.screen = Screen::TextInput;
        self.inp_title = title.into();
        self.inp_fields = fields;
        self.inp_focus = 0;
        self.pending = action;
        self.needs_redraw = true;
    }

    fn go_cfm(&mut self, msg: &str, action: Pending) {
        self.screen = Screen::Confirm;
        self.cfm_msg = msg.into();
        self.cfm_yes = false;
        self.pending = action;
        self.needs_redraw = true;
    }

    fn back(&mut self) {
        self.screen = Screen::MainMenu;
        self.pending = Pending::None;
        self.needs_redraw = true;
    }

    // ── Tick – refresh live data ────────────────────────────────────

    async fn tick(&mut self) {
        let elapsed = self.last_tick.elapsed();
        match self.screen {
            Screen::LiveTasks if elapsed >= Duration::from_secs(2) => {
                self.last_tick = Instant::now();
                
                // Use timeout to prevent blocking
                let db = Arc::clone(&self.state.db);
                let key = env::var("WEATHER_API_KEY").unwrap_or_else(|_| "default_key".into());
                let city = env::var("CITY").unwrap_or_else(|_| "Surabaya".into());
                
                // Only fetch weather every 5 minutes to avoid blocking
                let should_fetch_weather = self.weather_last_fetch.elapsed() >= Duration::from_secs(300);
                
                // Spawn non-blocking task with timeout
                if let Ok(Some(plants)) = tokio::time::timeout(
                    Duration::from_millis(500),
                    async {
                        let plants = db.active_plants().await.ok()?;
                        Some(plants)
                    }
                ).await {
                    // Store all plants for display
                    self.all_plants.clone_from(&plants);
                    
                    // Fetch weather in background if needed
                    if should_fetch_weather {
                        let key_clone = key.clone();
                        let city_clone = city.clone();
                        tokio::spawn(async move {
                            match tokio::time::timeout(
                                Duration::from_millis(1000), // Increased timeout slightly
                                weather(&city_clone, &key_clone)
                            ).await {
                                Ok(Ok(_)) => {},
                                Ok(Err(e)) => tracing::warn!("Background weather fetch failed: {}", e),
                                Err(_) => tracing::warn!("Background weather fetch timed out"),
                            }
                        });
                        self.weather_last_fetch = Instant::now();
                    }
                    
                    let wc = self.weather.as_ref().map(|(c, _)| c.as_str());
                    self.tasks = calculate_today_tasks(&plants, wc, None).await;
                    self.needs_redraw = true;
                }
            }
            Screen::GardenStats if elapsed >= Duration::from_secs(2) => {
                self.last_tick = Instant::now();
                
                // Use timeout to prevent blocking
                let db = Arc::clone(&self.state.db);
                if let Ok(Some(s)) = tokio::time::timeout(
                    Duration::from_millis(500),
                    async { db.garden_stats().await.ok() }
                ).await {
                    self.stats = s;
                    self.needs_redraw = true;
                }
            }
            Screen::LiveSensor if elapsed >= Duration::from_secs(1) => {
                self.last_tick = Instant::now();
                
                // Use timeout to prevent blocking
                let db = Arc::clone(&self.state.db);
                if let Ok(Some(plants)) = tokio::time::timeout(
                    Duration::from_millis(500),
                    async { db.active_plants().await.ok() }
                ).await {
                    self.sensors = plants.iter().map(|p| SensorRow {
                        name: p.name.clone(),
                        moisture: read_soil_moisture(&p.name),
                        temperature: read_temperature(),
                        humidity: read_humidity(),
                    }).collect();
                    self.needs_redraw = true;
                }
            }
            Screen::DaemonMonitor if elapsed >= Duration::from_secs(5) && self.daemon_running => {
                self.last_tick = Instant::now();
                self.daemon_cycle_count += 1;
                self.run_daemon_cycle().await;
                self.needs_redraw = true;
            }
            _ => {}
        }
    }

    fn add_daemon_log(&mut self, level: &str, message: &str) {
        self.daemon_logs.push(DaemonLog {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            level: level.to_string(),
            message: message.to_string(),
        });
        
        // Keep only last 100 logs
        if self.daemon_logs.len() > 100 {
            self.daemon_logs.remove(0);
        }
    }

    fn start_web_server(&mut self) {
        if self.web_server_started {
            return;
        }
        
        let state = self.state.clone();
        let cancel_token = self.cancel_token.clone();
        
        // Spawn web server in background
        tokio::spawn(async move {
            if let Err(e) = crate::web::serve(state, cancel_token).await {
                eprintln!("Web server error: {e}");
            }
        });
        
        self.web_server_started = true;
    }

    async fn run_daemon_cycle(&mut self) {
        self.add_daemon_log("info", &format!("🔄 Cycle #{} - Checking plants...", self.daemon_cycle_count));
        
        let db = Arc::clone(&self.state.db);
        match db.active_plants().await {
            Ok(plants) if plants.is_empty() => {
                self.add_daemon_log("warning", "⚠️  No active plants to monitor");
            }
            Ok(plants) => {
                self.add_daemon_log("info", &format!("📊 Monitoring {} plant(s)", plants.len()));
                
                // Broadcast sensor data to web dashboard via Shared State
                for plant in &plants {
                    let moisture = crate::hardware::read_soil_moisture(&plant.name);
                    let temperature = crate::hardware::read_temperature();
                    let humidity = crate::hardware::read_humidity();
                    let min_threshold = plant.min_moisture.unwrap_or(40.0);

                    // Log sensor data to DB
                    if let Err(e) = db.log_sensor_data(&plant.name, moisture, temperature, humidity).await {
                        self.add_daemon_log("error", &format!("❌ Failed to log sensor data: {e}"));
                    }
                    
                    // Direct Broadcast (Internal Channel)
                    let _ = self.state.tx.send(crate::web::DashboardMessage::SensorUpdate(crate::web::SensorData {
                        plant_name: plant.name.clone(),
                        moisture,
                        temperature,
                        humidity,
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        min_moisture: plant.min_moisture,
                        water_ml: plant.water_ml,
                    }));

                    if self.daemon_cycle_count == 1 {
                        self.add_daemon_log("info", &format!("📡 Internal Broadcast: {}", plant.name));
                    }
                    
                    if moisture < min_threshold {
                        #[allow(clippy::cast_possible_truncation)]
                        let moisture_i = moisture as i32;
                        #[allow(clippy::cast_possible_truncation)]
                        let threshold_i = min_threshold as i32;
                        self.add_daemon_log("warning", &format!("💧 {} needs water! ({moisture_i}% < {threshold_i}%)", plant.name));
                        
                        // Simulate watering
                        let water_ml = plant.water_ml.unwrap_or(200);
                        self.add_daemon_log("success", &format!("✅ Watered {} with {water_ml}ml", plant.name));
                        
                        // Update database
                        let _ = db.update_care(&plant.name, crate::core::CareType::Water).await;

                        // Telegram Alert (v1.3.3)
                        let alert_msg = format!("🚨 *{p_name}*: Moisture LOW ({moisture:.1}%). Pump triggered from TUI Console!", p_name = plant.name);
                        let _ = crate::telegram::send_telegram_alert(&db, &alert_msg).await;
                    } else {
                        #[allow(clippy::cast_possible_truncation)]
                        let moisture_i = moisture as i32;
                        self.add_daemon_log("info", &format!("✅ {} is healthy ({moisture_i}%)", plant.name));
                    }
                }
                
                self.add_daemon_log("success", "✨ Cycle complete - Data sent to web dashboard");
            }
            Err(e) => {
                self.add_daemon_log("error", &format!("❌ Database error: {e}"));
            }
        }
    }

    // ── Key handling ────────────────────────────────────────────────

    #[allow(clippy::too_many_lines)]
    async fn handle_key(&mut self, key: event::KeyEvent) {
        // Only process Press and Repeat events, ignore Release
        if key.kind == KeyEventKind::Release { return; }

        match self.screen {
            Screen::MainMenu => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.menu_idx = self.menu_idx.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.menu_idx < MENU_ITEMS.len() - 1 { self.menu_idx += 1; }
                }
                KeyCode::Enter => self.on_menu_select().await,
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.exit_signal = Some(ExitSignal::Quit);
                }
                _ => {}
            },
            Screen::LiveTasks | Screen::GardenStats | Screen::LiveSensor => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.screen = Screen::MainMenu;
                        self.pending = Pending::None;
                    }
                    KeyCode::Char(' ') => {
                        // Force refresh
                        self.last_tick = Instant::now().checked_sub(Duration::from_secs(100)).unwrap_or_else(Instant::now);
                    }
                    _ => {}
                }
            }
            Screen::WebDashboard => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.screen = Screen::MainMenu;
                        self.pending = Pending::None;
                    }
                    KeyCode::Char('s') => {
                        // Start web server
                        if !self.web_server_started {
                            self.start_web_server();
                        }
                    }
                    _ => {}
                }
            }
            Screen::AiAgent => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.screen = Screen::MainMenu;
                        self.pending = Pending::None;
                        self.ai_input.clear();
                    }
                    KeyCode::Char(c) => {
                        if !self.ai_processing {
                            self.ai_input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if !self.ai_processing {
                            self.ai_input.pop();
                        }
                    }
                    KeyCode::Enter => {
                        if !self.ai_processing && !self.ai_input.trim().is_empty() {
                            self.on_ai_submit().await;
                        }
                    }
                    _ => {}
                }
            }
            Screen::DaemonMonitor => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.daemon_running = false;
                        self.screen = Screen::MainMenu;
                        self.pending = Pending::None;
                    }
                    KeyCode::Char('s') => {
                        // Toggle daemon start/stop
                        self.daemon_running = !self.daemon_running;
                        if self.daemon_running {
                            self.add_daemon_log("info", "🚀 Daemon started");
                        } else {
                            self.add_daemon_log("warning", "⏸️  Daemon paused");
                        }
                    }
                    KeyCode::Char('c') => {
                        // Clear logs
                        self.daemon_logs.clear();
                        self.add_daemon_log("info", "📋 Logs cleared");
                    }
                    _ => {}
                }
            }
            Screen::SelectList => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.sel_idx = self.sel_idx.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.sel_idx < self.sel_items.len().saturating_sub(1) { self.sel_idx += 1; }
                }
                KeyCode::Enter => self.on_sel_confirm().await,
                KeyCode::Esc => self.back(),
                _ => {}
            },
            Screen::TextInput => match key.code {
                KeyCode::Tab | KeyCode::Down => {
                    if self.inp_focus < self.inp_fields.len() - 1 { self.inp_focus += 1; }
                }
                KeyCode::BackTab | KeyCode::Up => {
                    self.inp_focus = self.inp_focus.saturating_sub(1);
                }
                KeyCode::Char(c) => self.inp_fields[self.inp_focus].value.push(c),
                KeyCode::Backspace => { self.inp_fields[self.inp_focus].value.pop(); }
                KeyCode::Enter => {
                    if self.inp_focus < self.inp_fields.len() - 1 {
                        self.inp_focus += 1;
                    } else {
                        self.on_inp_submit().await;
                    }
                }
                KeyCode::Esc => self.back(),
                _ => {}
            },
            Screen::Confirm => match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => self.cfm_yes = !self.cfm_yes,
                KeyCode::Char('y') => { self.cfm_yes = true; self.on_cfm_yes().await; }
                KeyCode::Char('n') | KeyCode::Esc => self.back(),
                KeyCode::Enter => {
                    if self.cfm_yes { self.on_cfm_yes().await; } else { self.back(); }
                }
                _ => {}
            },
            Screen::Message => self.back(),
        }
        self.needs_redraw = true;
    }

    // ── Menu select ─────────────────────────────────────────────────

    async fn on_menu_select(&mut self) {
        match self.menu_idx {
            0 => {
                // AI Agent Mode - show configuration flow
                let providers = vec![
                    "Google Gemini 1.5 Flash".to_string(),
                    "Google Gemini 1.5 Pro".to_string(),
                    "Anthropic Claude 3.5 Sonnet".to_string(),
                    "Anthropic Claude 3 Opus".to_string(),
                    "OpenAI ChatGPT-4 Turbo".to_string(),
                    "OpenAI ChatGPT-5".to_string(),
                    "Simulation Mode".to_string(),
                ];
                self.go_sel("🤖 Select AI Provider", providers, Pending::AiProviderSelect);
            }
            1 => { 
                self.screen = Screen::LiveTasks; 
                // Force immediate data load
                self.last_tick = Instant::now().checked_sub(Duration::from_secs(100)).unwrap_or_else(Instant::now);
            }
            2 => { // Add Plant
                match load_rules().await {
                    Ok(r) => {
                        let types: Vec<String> = r.keys().cloned().collect();
                        if types.is_empty() {
                            self.go_msg("❌ Error", "No plant types found in plants.yaml");
                        } else {
                            self.go_sel("Select plant type:", types, Pending::AddType);
                        }
                    }
                    Err(_) => self.go_msg("❌ Error", "Failed to load plants.yaml"),
                }
            }
            3 => { 
                self.screen = Screen::GardenStats; 
                // Force immediate data load
                self.last_tick = Instant::now().checked_sub(Duration::from_secs(100)).unwrap_or_else(Instant::now);
            }
            4 => { 
                self.screen = Screen::LiveSensor; 
                // Force immediate data load
                self.last_tick = Instant::now().checked_sub(Duration::from_secs(100)).unwrap_or_else(Instant::now);
            }
            5 => self.plant_list_for("Which plant do you want to harvest?", Pending::Harvest).await,
            6 => self.plant_list_for("Which plant do you want to permanently delete?", Pending::DeleteSelect).await,
            7 => self.plant_list_for("Which plant do you want to configure?", Pending::ConfigSelect).await,
            8 => {
                self.go_inp("☁️  Configure Weather API", vec![
                    Field { label: "City".into(), value: String::new(), default: "Surabaya".into() },
                    Field { label: "OpenWeatherMap API Key".into(), value: String::new(), default: String::new() },
                ], Pending::WeatherInput);
            }
            9 => {
                // Daemon Monitor - show in TUI
                self.screen = Screen::DaemonMonitor;
                self.daemon_logs.clear();
                self.daemon_running = false;
                self.daemon_cycle_count = 0;
                self.add_daemon_log("info", "🤖 Daemon Monitor initialized");
                self.add_daemon_log("info", "💡 Press 's' to start/stop, 'c' to clear logs");
                
                // Ensure web server is running so broadcasts don't fail
                if !self.web_server_started {
                    self.start_web_server();
                    self.add_daemon_log("info", "🌐 Web server started automatically");
                }
            }
            10 => {
                // Show Web Dashboard info screen
                let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
                let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
                // If host is 0.0.0.0, show localhost for browser access
                let browser_host = if host == "0.0.0.0" { "127.0.0.1" } else { &host };
                self.web_url = format!("http://{browser_host}:{port}");
                self.web_running = true;
                self.screen = Screen::WebDashboard;
                
                // Try to auto-start web server in background
                if !self.web_server_started {
                    self.start_web_server();
                }
            }
            11 => self.exit_signal = Some(ExitSignal::Quit),
            _ => {}
        }
    }

    async fn plant_list_for(&mut self, title: &str, action: Pending) {
        match self.state.db.active_plants().await {
            Ok(plants) if plants.is_empty() => self.go_msg("Info", "No active plants."),
            Ok(plants) => {
                let names = plants.into_iter().map(|p| p.name).collect();
                self.go_sel(title, names, action);
            }
            Err(e) => self.go_msg("❌ Error", &format!("Failed to get plants: {e}")),
        }
    }

    // ── Select confirm ──────────────────────────────────────────────

    async fn on_sel_confirm(&mut self) {
        if self.sel_idx >= self.sel_items.len() { return; }
        let selected = self.sel_items[self.sel_idx].clone();
        match &self.pending {
            Pending::AddType => {
                self.go_inp(
                    &format!("➕ Add New {selected} Plant"),
                    vec![Field { label: "Give your plant a unique nickname".into(), value: String::new(), default: String::new() }],
                    Pending::AddName { plant_type: selected },
                );
            }
            Pending::Harvest => {
                match self.state.db.harvest_plant(&selected).await {
                    Ok(()) => self.go_msg("🎉 Harvested!", &format!("{selected} has been archived.")),
                    Err(e) => self.go_msg("❌ Error", &format!("Failed: {e}")),
                }
            }
            Pending::DeleteSelect => {
                self.go_cfm(
                    &format!("⚠️  Are you sure you want to delete '{selected}' and all its data?"),
                    Pending::DeleteConfirm { name: selected },
                );
            }
            Pending::ConfigSelect => {
                self.go_inp(
                    &format!("⚙️  Configure {selected}"),
                    vec![
                        Field { label: "Minimum moisture threshold (%)".into(), value: String::new(), default: "40".into() },
                        Field { label: "Water volume (ml)".into(), value: String::new(), default: "200".into() },
                    ],
                    Pending::ConfigInput { name: selected },
                );
            }
            Pending::AiProviderSelect => {
                let provider = match selected.as_str() {
                    "Google Gemini 1.5 Flash" => crate::ai::AiProvider::GeminiFlash,
                    "Google Gemini 1.5 Pro" => crate::ai::AiProvider::GeminiPro,
                    "Anthropic Claude 3.5 Sonnet" => crate::ai::AiProvider::ClaudeSonnet,
                    "Anthropic Claude 3 Opus" => crate::ai::AiProvider::ClaudeOpus,
                    "OpenAI ChatGPT-4 Turbo" => crate::ai::AiProvider::ChatGpt4,
                    "OpenAI ChatGPT-5" => crate::ai::AiProvider::ChatGpt5,
                    _ => crate::ai::AiProvider::Simulation,
                };

                if matches!(provider, crate::ai::AiProvider::Simulation) {
                    self.ai_provider = Some(provider);
                    self.ai_api_key = None;
                    self.go_ai_agent();
                } else {
                    // Check if env key exists for hinting
                    let env_key_exists = match provider {
                        crate::ai::AiProvider::GeminiFlash | crate::ai::AiProvider::GeminiPro => env::var("GEMINI_API_KEY").is_ok(),
                        crate::ai::AiProvider::ClaudeSonnet | crate::ai::AiProvider::ClaudeOpus => env::var("ANTHROPIC_API_KEY").is_ok(),
                        crate::ai::AiProvider::ChatGpt4 | crate::ai::AiProvider::ChatGpt5 => env::var("OPENAI_API_KEY").is_ok(),
                        crate::ai::AiProvider::Simulation => false,
                    };

                    let hint = if env_key_exists {
                        " (Leave blank to use key from .env)"
                    } else {
                        ""
                    };

                    self.go_inp(
                        &format!("🔑 AI Key for {selected}{hint}"),
                        vec![Field { label: "API Key".into(), value: String::new(), default: String::new() }],
                        Pending::AiApiKeyInput { provider },
                    );
                }
            }
            _ => self.back(),
        }
    }

    fn go_ai_agent(&mut self) {
        self.screen = Screen::AiAgent;
        self.ai_input.clear();
        self.ai_messages.clear();
        
        let provider_name = match self.ai_provider {
            Some(crate::ai::AiProvider::GeminiFlash) => "Gemini Flash",
            Some(crate::ai::AiProvider::GeminiPro) => "Gemini Pro",
            Some(crate::ai::AiProvider::ClaudeSonnet) => "Claude Sonnet",
            Some(crate::ai::AiProvider::ClaudeOpus) => "Claude Opus",
            Some(crate::ai::AiProvider::ChatGpt4) => "ChatGPT-4",
            Some(crate::ai::AiProvider::ChatGpt5) => "ChatGPT-5",
            _ => "Simulation",
        };

        let status = if self.ai_api_key.is_some() { "● Connected" } else { "○ Simulation" };

        self.ai_messages.push(ChatMessage {
            role: "ai".to_string(),
            text: format!("👋 Hello! AI Agent initialized with {provider_name} ({status}). Ask me anything about your garden!"),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
    }

    // ── Input submit ────────────────────────────────────────────────

    fn field_val(&self, idx: usize) -> String {
        let f = &self.inp_fields[idx];
        if f.value.is_empty() { f.default.clone() } else { f.value.clone() }
    }

    async fn on_inp_submit(&mut self) {
        match &self.pending {
            Pending::AddName { plant_type } => {
                let name = self.field_val(0);
                if name.is_empty() {
                    self.go_msg("❌ Error", "Plant name cannot be empty.");
                    return;
                }
                let pt = plant_type.clone();
                match self.state.db.add_plant(&pt, &name).await {
                    Ok(true) => self.go_msg("✅ Success!", &format!("Added {name} ({pt}) to your garden.")),
                    Ok(false) => self.go_msg("❌ Error", &format!("A plant named '{name}' already exists.")),
                    Err(e) => self.go_msg("❌ Error", &format!("Failed: {e}")),
                }
            }
            Pending::ConfigInput { name } => {
                let mm: f32 = self.field_val(0).parse().unwrap_or(40.0);
                let wm: i32 = self.field_val(1).parse().unwrap_or(200);
                let pn = name.clone();
                match self.state.db.update_plant_settings(&pn, mm, wm).await {
                    Ok(()) => self.go_msg("⚙️  Updated!", &format!("{pn}: Min Moisture = {mm}%, Water = {wm}ml")),
                    Err(e) => self.go_msg("❌ Error", &format!("Failed: {e}")),
                }
            }
            Pending::WeatherInput => {
                let city = self.field_val(0);
                let key = self.field_val(1);
                self.go_msg("✅ Configured!", &format!("Weather configured for {city} with key {key}..."));
            }
            Pending::AiApiKeyInput { provider } => {
                let user_key = self.field_val(0).trim().to_string();
                
                if user_key.is_empty() {
                    // Fallback to env
                    let env_key = match provider {
                        crate::ai::AiProvider::GeminiFlash | crate::ai::AiProvider::GeminiPro => env::var("GEMINI_API_KEY").ok(),
                        crate::ai::AiProvider::ClaudeSonnet | crate::ai::AiProvider::ClaudeOpus => env::var("ANTHROPIC_API_KEY").ok(),
                        crate::ai::AiProvider::ChatGpt4 | crate::ai::AiProvider::ChatGpt5 => env::var("OPENAI_API_KEY").ok(),
                        crate::ai::AiProvider::Simulation => None,
                    };

                    if let Some(key) = env_key {
                        self.ai_provider = Some(provider.clone());
                        self.ai_api_key = Some(key);
                    } else {
                        // Truly empty, default to simulation or show error?
                        // For now, default to simulation to avoid crashing
                        self.ai_provider = Some(crate::ai::AiProvider::Simulation);
                        self.ai_api_key = None;
                    }
                } else {
                    self.ai_provider = Some(provider.clone());
                    self.ai_api_key = Some(user_key);
                }
                self.go_ai_agent();
            }
            _ => self.back(),
        }
    }

    // ── Confirm yes ─────────────────────────────────────────────────

    async fn on_cfm_yes(&mut self) {
        match &self.pending {
            Pending::DeleteConfirm { name } => {
                let n = name.clone();
                match self.state.db.delete_plant(&n).await {
                    Ok(()) => self.go_msg("🗑️  Deleted!", &format!("{n} and all sensor logs deleted.")),
                    Err(e) => self.go_msg("❌ Error", &format!("Failed: {e}")),
                }
            }
            _ => self.back(),
        }
    }

    // ── AI Agent submit ─────────────────────────────────────────

    async fn on_ai_submit(&mut self) {
        let user_input = self.ai_input.trim().to_string();
        if user_input.is_empty() {
            return;
        }

        // Add user message
        self.ai_messages.push(ChatMessage {
            role: "user".to_string(),
            text: user_input.clone(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });

        self.ai_input.clear();
        self.ai_processing = true;

        // Simple simulation mode responses
        let response = if user_input.to_lowercase().contains("status") || user_input.to_lowercase().contains("how") {
            match self.state.db.active_plants().await {
                Ok(plants) if plants.is_empty() => {
                    "🌱 You don't have any active plants yet. Add some plants from the main menu!".to_string()
                }
                Ok(plants) => {
                    let mut resp = format!("🌱 You have {} active plant(s):\n\n", plants.len());
                    for plant in plants.iter().take(5) {
                        let _ = writeln!(resp, "• {name} ({pt})", name = plant.name, pt = plant.plant_type);
                    }
                    resp.push_str("\nAll plants are being monitored! 🎉");
                    resp
                }
                Err(_) => "❌ Error accessing database.".to_string(),
            }
        } else if user_input.to_lowercase().contains("water") || user_input.to_lowercase().contains("siram") {
            "💧 Watering command received! In full AI mode, I can automatically water specific plants. For now, use the daemon or web dashboard for manual control.".to_string()
        } else if user_input.to_lowercase().contains("sensor") || user_input.to_lowercase().contains("temperature") || user_input.to_lowercase().contains("humidity") {
            "📡 Sensor readings are available in the 'Live Sensor Monitor' screen. Check it out from the main menu!".to_string()
        } else if user_input.to_lowercase().contains("help") {
            "💡 I can help you with:\n\n• Check plant status\n• Monitor sensors\n• Water plants (coming soon)\n• Answer garden questions\n\nTry asking: 'How are my plants?' or 'Show sensor status'".to_string()
        } else {
            "🤖 I understand your request! In full AI mode with API keys (Gemini/Claude/ChatGPT), I can provide detailed responses. For now, try: 'status', 'help', or 'sensor'.".to_string()
        };

        // Add AI response
        self.ai_messages.push(ChatMessage {
            role: "ai".to_string(),
            text: response,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });

        self.ai_processing = false;
    }


    // ── Render dispatch ─────────────────────────────────────────────

    fn render(&self, frame: &mut Frame) {
        // Dark background
        let bg = Block::default().style(Style::default().bg(Color::Rgb(15, 15, 25)));
        frame.render_widget(bg, frame.size());

        match self.screen {
            Screen::MainMenu => self.draw_menu(frame),
            Screen::LiveTasks => self.draw_tasks(frame),
            Screen::GardenStats => self.draw_stats(frame),
            Screen::LiveSensor => self.draw_sensor(frame),
            Screen::SelectList => self.draw_select(frame),
            Screen::TextInput => self.draw_input(frame),
            Screen::Confirm => self.draw_confirm(frame),
            Screen::Message => self.draw_message(frame),
            Screen::WebDashboard => self.draw_web_dashboard(frame),
            Screen::AiAgent => self.draw_ai_agent(frame),
            Screen::DaemonMonitor => self.draw_daemon_monitor(frame),
        }
    }

    // ── Render: Main Menu ───────────────────────────────────────────

    fn draw_menu(&self, f: &mut Frame) {
        let area = f.size();
        // Outer margin
        let outer = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(area);
        let inner = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]).split(outer[1]);
        let content = inner[1];

        let chunks = Layout::vertical([
            Constraint::Length(9),  // Banner
            Constraint::Length(1),  // Separator
            Constraint::Min(14),   // Menu
            Constraint::Length(3),  // Footer
        ]).split(content);

        // ── Banner ──
        let banner = vec![
            Line::styled("  █████╗  ██████╗ ██████╗  ██████╗  ██████╗██╗     ██╗", Style::default().fg(Color::Rgb(0, 200, 83))),
            Line::styled(" ██╔══██╗██╔════╝ ██╔══██╗██╔═══██╗██╔════╝██║     ██║", Style::default().fg(Color::Rgb(0, 200, 83))),
            Line::styled(" ███████║██║  ███╗██████╔╝██║   ██║██║     ██║     ██║", Style::default().fg(Color::Rgb(0, 230, 118))),
            Line::styled(" ██╔══██║██║   ██║██╔══██╗██║   ██║██║     ██║     ██║", Style::default().fg(Color::Rgb(0, 230, 118))),
            Line::styled(" ██║  ██║╚██████╔╝██║  ██║╚██████╔╝╚██████╗███████╗██║", Style::default().fg(Color::Rgb(100, 255, 218))),
            Line::styled(" ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚══════╝╚═╝", Style::default().fg(Color::Rgb(100, 255, 218))),
            Line::default(),
            Line::from(vec![
                Span::styled(" ═══ ", Style::default().fg(Color::Rgb(80, 80, 100))),
                Span::styled("THE INTELLIGENT GARDEN BRAIN", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                Span::styled(" (RUST) ", Style::default().fg(Color::Rgb(255, 183, 77)).bold()),
                Span::styled("═══", Style::default().fg(Color::Rgb(80, 80, 100))),
            ]),
            Line::from(vec![
                Span::styled(" v", Style::default().fg(Color::Rgb(120, 120, 140))),
                Span::styled(VERSION, Style::default().fg(Color::Rgb(255, 213, 79))),
                Span::styled(" │ ", Style::default().fg(Color::Rgb(60, 60, 80))),
                Span::styled("Local Network Node ● Active", Style::default().fg(Color::Rgb(0, 200, 83))),
                Span::styled(" │ ", Style::default().fg(Color::Rgb(60, 60, 80))),
                Span::styled("by Naufal Rizky 💚", Style::default().fg(Color::Rgb(160, 160, 180))),
            ]),
        ];
        f.render_widget(Paragraph::new(banner), chunks[0]);

        // ── Separator ──
        let sep = Paragraph::new(Line::styled(
            "─".repeat(content.width as usize),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(sep, chunks[1]);

        // ── Menu ──
        let items: Vec<ListItem> = MENU_ITEMS.iter().enumerate().map(|(i, item)| {
            if i == self.menu_idx {
                ListItem::new(Line::from(vec![
                    Span::styled("  ▸ ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                    Span::styled(item.to_string(), Style::default().fg(Color::White).bold()),
                ])).style(Style::default().bg(Color::Rgb(30, 50, 40)))
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(item.to_string(), Style::default().fg(Color::Rgb(160, 160, 180))),
                ]))
            }
        }).collect();

        let menu = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                .title(Span::styled(" 🌿 Menu ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
        );
        f.render_widget(menu, chunks[2]);

        // ── Footer ──
        f.render_widget(make_footer(&["↑↓/jk", "Navigate", "Enter", "Select", "q", "Quit"]), chunks[3]);
    }

    // ── Render: Live Tasks ──────────────────────────────────────────

    #[allow(clippy::too_many_lines)]
    fn draw_tasks(&self, f: &mut Frame) {
        let area = f.size();
        // Outer margin
        let outer = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(area);
        let inner = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]).split(outer[1]);
        let content = inner[1];

        let chunks = Layout::vertical([
            Constraint::Length(5),  // Header
            Constraint::Length(1),  // Separator
            Constraint::Min(5),     // Tasks
            Constraint::Length(3),  // Footer
        ]).split(content);

        // ── Header ──
        let now = chrono::Local::now().format("%H:%M:%S");
        let elapsed_secs = self.last_tick.elapsed().as_secs();
        let refresh_indicator = if elapsed_secs < 2 { "●" } else { "○" };
        
        let mut header_lines = vec![
            Line::default(),
            Line::from(vec![
                Span::styled("  🌱 ", Style::default()),
                Span::styled("LIVE TASK MONITOR", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                Span::styled(format!("  │  {now}", ), Style::default().fg(Color::Rgb(100, 100, 120))),
                Span::styled(format!("  {refresh_indicator}"), Style::default().fg(Color::Rgb(0, 200, 83))),
            ]),
        ];
        if let Some((ref c, t)) = self.weather {
            header_lines.push(Line::from(vec![
                Span::styled("  🌦️  ", Style::default()),
                Span::styled(format!("{c} ({t:.1}°C)"), Style::default().fg(Color::Rgb(255, 213, 79))),
            ]));
        } else {
            header_lines.push(Line::from(vec![
                Span::styled("  🌦️  ", Style::default()),
                Span::styled("Weather data unavailable", Style::default().fg(Color::Rgb(120, 120, 140))),
            ]));
        }
        f.render_widget(
            Paragraph::new(header_lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
            ),
            chunks[0],
        );

        // ── Separator ──
        let sep = Paragraph::new(Line::styled(
            "─".repeat(content.width as usize),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(sep, chunks[1]);

        // ── Tasks ──
        if self.all_plants.is_empty() {
            // No plants at all
            let empty_text = vec![
                Line::default(),
                Line::from(vec![
                    Span::styled("  📭 ", Style::default()),
                    Span::styled("No plants found in database.", Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  💡 ", Style::default()),
                    Span::styled("Tip: ", Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
                    Span::styled("Add plants from main menu (➕ Add New Plant)", Style::default().fg(Color::Rgb(160, 160, 180))),
                ]),
                Line::default(),
            ];
            f.render_widget(
                Paragraph::new(empty_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                        .title(Span::styled(" 📋 Today's Tasks ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
                ),
                chunks[2],
            );
        } else if self.tasks.is_empty() {
            // Plants exist but no tasks
            let mut info_lines = vec![
                Line::default(),
                Line::from(vec![
                    Span::styled("  ✨ ", Style::default()),
                    Span::styled("All caught up! No tasks needed today.", Style::default().fg(Color::Rgb(0, 200, 83)).bold()),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  🌱 ", Style::default()),
                    Span::styled(format!("Active Plants: {len}", len = self.all_plants.len()), Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                ]),
                Line::default(),
            ];
            
            // Show plant status
            for plant in &self.all_plants {
                let days_since_water = chrono::Local::now()
                    .signed_duration_since(
                        chrono::NaiveDate::parse_from_str(&plant.last_watered, "%Y-%m-%d")
                            .ok()
                            .and_then(|d| d.and_hms_opt(0, 0, 0))
                            .map_or_else(chrono::Local::now, |dt| {
                                chrono::TimeZone::from_local_datetime(&chrono::Local, &dt)
                                    .earliest()
                                    .unwrap_or_else(chrono::Local::now)
                            })
                    )
                    .num_days();
                
                info_lines.push(Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled(&plant.name, Style::default().fg(Color::White)),
                    Span::styled(format!(" ({pt})", pt = plant.plant_type), Style::default().fg(Color::Rgb(120, 120, 140))),
                ]));
                info_lines.push(Line::from(vec![
                    Span::styled("       ", Style::default()),
                    Span::styled(format!("Last watered: {days_since_water} days ago"), Style::default().fg(Color::Rgb(160, 160, 180))),
                ]));
            }
            
            info_lines.push(Line::default());
            info_lines.push(Line::from(vec![
                Span::styled("  💡 ", Style::default()),
                Span::styled("All plants are healthy! Check back tomorrow.", Style::default().fg(Color::Rgb(160, 160, 180))),
            ]));
            info_lines.push(Line::default());
            
            f.render_widget(
                Paragraph::new(info_lines).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                        .title(Span::styled(" 📋 Today's Tasks ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
                ),
                chunks[2],
            );
        } else {
            let items: Vec<ListItem> = self.tasks.iter().map(|t| {
                let w = if t.needs_water {
                    Span::styled(format!("💧 {}ml", t.water_ml), Style::default().fg(Color::Rgb(66, 165, 245)).bold())
                } else {
                    Span::styled("✓ OK", Style::default().fg(Color::Rgb(0, 200, 83)))
                };
                let fe = if t.needs_fertilizer {
                    Span::styled("🌾 Yes", Style::default().fg(Color::Rgb(255, 213, 79)).bold())
                } else {
                    Span::styled("✓ OK", Style::default().fg(Color::Rgb(0, 200, 83)))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {:<16}", t.name), Style::default().fg(Color::White).bold()),
                    Span::styled("Water: ", Style::default().fg(Color::Rgb(120, 120, 140))),
                    w,
                    Span::styled("  │  Fertilizer: ", Style::default().fg(Color::Rgb(120, 120, 140))),
                    fe,
                ]))
            }).collect();
            f.render_widget(
                List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                        .title(Span::styled(
                            format!(" 📋 Today's Tasks ({} items) ", self.tasks.len()), 
                            Style::default().fg(Color::Rgb(100, 255, 218)).bold()
                        ))
                ),
                chunks[2],
            );
        }
        f.render_widget(make_footer(&["Auto-refresh 2s", "", "q/Esc", "Back to Menu", "Space", "Force refresh"]), chunks[3]);
    }

    // ── Render: Garden Stats ────────────────────────────────────────

    fn draw_stats(&self, f: &mut Frame) {
        let area = f.size();
        // Outer margin
        let outer = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(area);
        let inner = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]).split(outer[1]);
        let content = inner[1];

        let chunks = Layout::vertical([
            Constraint::Length(4),  // Header
            Constraint::Length(1),  // Separator
            Constraint::Min(8),     // Stats
            Constraint::Length(3),  // Footer
        ]).split(content);

        // ── Header ──
        let now = chrono::Local::now().format("%H:%M:%S");
        let elapsed_secs = self.last_tick.elapsed().as_secs();
        let refresh_indicator = if elapsed_secs < 2 { "●" } else { "○" };
        
        let header = vec![
            Line::default(),
            Line::from(vec![
                Span::styled("  📊 ", Style::default()),
                Span::styled("GARDEN SYSTEM STATISTICS", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                Span::styled(format!("  │  {now}"), Style::default().fg(Color::Rgb(100, 100, 120))),
                Span::styled(format!("  {refresh_indicator}"), Style::default().fg(Color::Rgb(0, 200, 83))),
            ]),
        ];
        f.render_widget(
            Paragraph::new(header).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
            ),
            chunks[0],
        );

        // ── Separator ──
        let sep = Paragraph::new(Line::styled(
            "─".repeat(content.width as usize),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(sep, chunks[1]);

        // ── Stats ──
        let active = self.stats.get("active_plants").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let harvested = self.stats.get("harvested_plants").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let text = vec![
            Line::default(),
            Line::from(vec![
                Span::styled("      🌱 ACTIVE PLANTS      ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled("│", Style::default().fg(Color::Rgb(50, 50, 70))),
                Span::styled(format!("  {active}"), Style::default().fg(Color::Rgb(0, 230, 118)).bold()),
            ]),
            Line::from(Span::styled("     ─────────────────────────┼──────", Style::default().fg(Color::Rgb(40, 40, 60)))),
            Line::from(vec![
                Span::styled("      🎉 HARVESTED TOTAL    ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled("│", Style::default().fg(Color::Rgb(50, 50, 70))),
                Span::styled(format!("  {harvested}"), Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
            ]),
            Line::default(),
            Line::from(vec![
                Span::styled("      💾 DATABASE STATUS    ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled("│", Style::default().fg(Color::Rgb(50, 50, 70))),
                Span::styled("  ● ONLINE", Style::default().fg(Color::Rgb(0, 200, 83)).bold()),
            ]),
            Line::default(),
        ];
        f.render_widget(
            Paragraph::new(text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                    .title(Span::styled(" 📈 Overview ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
            ),
            chunks[2],
        );
        f.render_widget(make_footer(&["Auto-refresh 2s", "", "q/Esc", "Back to Menu", "Space", "Force refresh"]), chunks[3]);
    }

    // ── Render: Live Sensor ─────────────────────────────────────────

    #[allow(clippy::too_many_lines, clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
    fn draw_sensor(&self, f: &mut Frame) {
        let area = f.size();
        // Outer margin
        let outer = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(area);
        let inner = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]).split(outer[1]);
        let content = inner[1];

        let chunks = Layout::vertical([
            Constraint::Length(4),  // Header
            Constraint::Length(1),  // Separator
            Constraint::Min(5),     // Sensors
            Constraint::Length(3),  // Footer
        ]).split(content);

        // ── Header ──
        let now = chrono::Local::now().format("%H:%M:%S");
        let elapsed_secs = self.last_tick.elapsed().as_secs();
        let refresh_indicator = if elapsed_secs < 1 { "●" } else { "○" };
        
        let header = vec![
            Line::default(),
            Line::from(vec![
                Span::styled("  📡 ", Style::default()),
                Span::styled("LIVE SYSTEM TELEMETRY", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                Span::styled(format!("  │  {now}"), Style::default().fg(Color::Rgb(100, 100, 120))),
                Span::styled(format!("  {refresh_indicator}"), Style::default().fg(Color::Rgb(0, 200, 83))),
            ]),
        ];
        f.render_widget(
            Paragraph::new(header).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
            ),
            chunks[0],
        );

        // ── Separator ──
        let sep = Paragraph::new(Line::styled(
            "─".repeat(content.width as usize),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(sep, chunks[1]);

        // ── Sensors ──
        if self.sensors.is_empty() {
            let empty_text = vec![
                Line::default(),
                Line::from(vec![
                    Span::styled("  ⏳ ", Style::default()),
                    Span::styled("Waiting for sensor data...", Style::default().fg(Color::Rgb(100, 100, 120))),
                ]),
                Line::default(),
            ];
            f.render_widget(
                Paragraph::new(empty_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                        .title(Span::styled(" 🌡️  Real-Time Sensors ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
                ),
                chunks[2],
            );
        } else {
            let items: Vec<ListItem> = self.sensors.iter().map(|s| {
                let (clr, tag) = if s.moisture < 30.0 {
                    (Color::Rgb(244, 67, 54), "⚠ THIRSTY")
                } else if s.moisture < 50.0 {
                    (Color::Rgb(255, 213, 79), "● NORMAL ")
                } else {
                    (Color::Rgb(0, 200, 83), "● HEALTHY")
                };
                let bar_len = 20usize;
                let filled = ((s.moisture / 100.0 * bar_len as f32) as usize).min(bar_len);
                let bar = format!("{}{}",
                    "━".repeat(filled),
                    "╌".repeat(bar_len - filled),
                );
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {:<14}", s.name), Style::default().fg(Color::White).bold()),
                    Span::styled("▐", Style::default().fg(Color::Rgb(50, 50, 70))),
                    Span::styled(bar, Style::default().fg(clr)),
                    Span::styled("▌", Style::default().fg(Color::Rgb(50, 50, 70))),
                    Span::styled(format!(" {:>5.1}%", s.moisture), Style::default().fg(Color::White)),
                    Span::styled(format!("  {tag}"), Style::default().fg(clr).bold()),
                    Span::styled(format!("  {:.1}°C", s.temperature), Style::default().fg(Color::Rgb(100, 181, 246))),
                    Span::styled(format!("  {:.0}%H", s.humidity), Style::default().fg(Color::Rgb(77, 182, 172))),
                ]))
            }).collect();
            f.render_widget(
                List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                        .title(Span::styled(" 🌡️  Real-Time Sensors ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
                ),
                chunks[2],
            );
        }
        f.render_widget(make_footer(&["Live feed 1s", "", "q/Esc", "Back to Menu", "Any key", "Force refresh"]), chunks[3]);
    }

    // ── Render: Select List ─────────────────────────────────────────

    fn draw_select(&self, f: &mut Frame) {
        let chunks = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());
        let items: Vec<ListItem> = self.sel_items.iter().enumerate().map(|(i, item)| {
            if i == self.sel_idx {
                ListItem::new(Line::from(vec![
                    Span::styled("  ▸ ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                    Span::styled(item.clone(), Style::default().fg(Color::White).bold()),
                ])).style(Style::default().bg(Color::Rgb(30, 50, 40)))
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(item.clone(), Style::default().fg(Color::Rgb(160, 160, 180))),
                ]))
            }
        }).collect();
        f.render_widget(
            List::new(items).block(
                Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                    .title(Span::styled(format!(" {} ", self.sel_title), Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
            ),
            chunks[0],
        );
        f.render_widget(make_footer(&["↑↓/jk", "Navigate", "Enter", "Select", "Esc", "Cancel"]), chunks[1]);
    }

    // ── Render: Text Input ──────────────────────────────────────────

    fn draw_input(&self, f: &mut Frame) {
        let n = self.inp_fields.len();
        let mut constraints = vec![Constraint::Length(3)]; // title
        for _ in 0..n { constraints.push(Constraint::Length(3)); }
        constraints.push(Constraint::Min(0)); // filler
        constraints.push(Constraint::Length(3)); // footer
        let chunks = Layout::vertical(constraints).split(f.size());

        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!(" {} ", self.inp_title), Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
            ])).block(make_block("")),
            chunks[0],
        );

        for (i, field) in self.inp_fields.iter().enumerate() {
            let focused = i == self.inp_focus;
            let bc = if focused { Color::Rgb(0, 200, 83) } else { Color::Rgb(50, 50, 70) };
            let display = if field.value.is_empty() && !field.default.is_empty() {
                if focused { format!("▏ (default: {})", field.default) } else { format!("(default: {})", field.default) }
            } else if focused {
                format!("{}▏", field.value)
            } else {
                field.value.clone()
            };
            let clr = if field.value.is_empty() && !focused { Color::Rgb(80, 80, 100) } else { Color::White };
            f.render_widget(
                Paragraph::new(Span::styled(format!(" {display}"), Style::default().fg(clr)))
                    .block(
                        Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(bc))
                            .title(Span::styled(format!(" {} ", field.label), Style::default().fg(Color::Rgb(100, 255, 218))))
                    ),
                chunks[i + 1],
            );
        }
        f.render_widget(make_footer(&["Tab/↑↓", "Switch", "Enter", "Submit", "Esc", "Cancel"]), chunks[n + 2]);
    }

    // ── Render: Confirm ─────────────────────────────────────────────

    fn draw_confirm(&self, f: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Min(5), Constraint::Length(3), Constraint::Length(3),
        ]).split(f.size());

        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("\n  {}", self.cfm_msg), Style::default().fg(Color::Rgb(255, 213, 79))),
            ])).block(
                Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(255, 152, 0)))
                    .title(Span::styled(" ⚠️  Confirm ", Style::default().fg(Color::Rgb(255, 152, 0)).bold()))
            ),
            chunks[0],
        );

        let no_s = if self.cfm_yes {
            Style::default().fg(Color::Rgb(80, 80, 100))
        } else {
            Style::default().fg(Color::Rgb(0, 200, 83)).bold()
        };
        let yes_s = if self.cfm_yes {
            Style::default().fg(Color::Rgb(244, 67, 54)).bold()
        } else {
            Style::default().fg(Color::Rgb(80, 80, 100))
        };
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("     ", Style::default()),
                Span::styled(if self.cfm_yes { "  " } else { "▸ " }, no_s),
                Span::styled("[ No, cancel ]", no_s),
                Span::styled("       ", Style::default()),
                Span::styled(if self.cfm_yes { "▸ " } else { "  " }, yes_s),
                Span::styled("[ Yes, delete permanently ]", yes_s),
            ])).block(make_block("")),
            chunks[1],
        );
        f.render_widget(make_footer(&["←→/Tab", "Switch", "y/n", "Quick", "Enter", "Confirm"]), chunks[2]);
    }

    // ── Render: Message ─────────────────────────────────────────────

    fn draw_message(&self, f: &mut Frame) {
        let chunks = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());

        let icon_color = if self.msg_title.contains('✅') || self.msg_title.contains('🎉') || self.msg_title.contains("⚙") {
            Color::Rgb(0, 200, 83)
        } else if self.msg_title.contains('❌') {
            Color::Rgb(244, 67, 54)
        } else {
            Color::Rgb(100, 255, 218)
        };

        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("\n  {}", self.msg_body), Style::default().fg(Color::White)),
            ])).block(
                Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                    .title(Span::styled(format!(" {} ", self.msg_title), Style::default().fg(icon_color).bold()))
            ),
            chunks[0],
        );
        f.render_widget(make_footer(&["Press any key", "to continue"]), chunks[1]);
    }

    // ── Render: Web Dashboard ───────────────────────────────────────

    #[allow(clippy::too_many_lines)]
    fn draw_web_dashboard(&self, f: &mut Frame) {
        let area = f.size();
        // Outer margin
        let outer = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(area);
        let inner = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]).split(outer[1]);
        let content = inner[1];

        let chunks = Layout::vertical([
            Constraint::Length(8),  // Header with status
            Constraint::Length(1),  // Separator
            Constraint::Min(12),    // Info
            Constraint::Length(3),  // Footer
        ]).split(content);

        // Extract port from URL
        let port = self.web_url.split(':').next_back().unwrap_or("8000");
        
        // Server status
        let (status_text, status_color) = if self.web_server_started {
            ("● RUNNING", Color::Rgb(0, 200, 83))
        } else {
            ("○ NOT STARTED", Color::Rgb(255, 213, 79))
        };

        // ── Header ──
        let header = vec![
            Line::default(),
            Line::from(vec![
                Span::styled("  🌐 ", Style::default()),
                Span::styled("WEB DASHBOARD", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                Span::styled("  │  ", Style::default().fg(Color::Rgb(60, 60, 80))),
                Span::styled(status_text, Style::default().fg(status_color).bold()),
            ]),
            Line::default(),
            Line::from(vec![
                Span::styled("  URL:    ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled(&self.web_url, Style::default().fg(Color::Rgb(100, 181, 246)).bold()),
            ]),
            Line::from(vec![
                Span::styled("  Port:   ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled(port, Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
            ]),
            Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled(
                    if self.web_server_started { "Ready to access in browser" } else { "Press 's' to start server" },
                    Style::default().fg(if self.web_server_started { Color::Rgb(0, 200, 83) } else { Color::Rgb(255, 213, 79) })
                ),
            ]),
        ];
        f.render_widget(
            Paragraph::new(header).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
            ),
            chunks[0],
        );

        // ── Separator ──
        let sep = Paragraph::new(Line::styled(
            "─".repeat(content.width as usize),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(sep, chunks[1]);

        // ── Info ──
        let info_text = if self.web_server_started {
            vec![
                Line::default(),
                Line::from(vec![
                    Span::styled("  ✅ ", Style::default()),
                    Span::styled("WEB SERVER IS RUNNING!", Style::default().fg(Color::Rgb(0, 200, 83)).bold()),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  🌐 ", Style::default()),
                    Span::styled("Access the dashboard:", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("     ", Style::default()),
                    Span::styled(&self.web_url, Style::default().fg(Color::Rgb(100, 181, 246)).bold()),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  ✨ ", Style::default()),
                    Span::styled("AVAILABLE FEATURES:", Style::default().fg(Color::Rgb(160, 160, 180))),
                ]),
                Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled("Real-time sensor monitoring", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled("Live charts and graphs", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled("Manual pump control", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled("WebSocket real-time updates", Style::default().fg(Color::White)),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  💡 ", Style::default()),
                    Span::styled("TIP: ", Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
                    Span::styled("Server runs in background. You can return to menu.", Style::default().fg(Color::Rgb(160, 160, 180))),
                ]),
                Line::default(),
            ]
        } else {
            vec![
                Line::default(),
                Line::from(vec![
                    Span::styled("  🚀 ", Style::default()),
                    Span::styled("QUICK START:", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  1. ", Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
                    Span::styled("Press 's' to start the web server", Style::default().fg(Color::White)),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  2. ", Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
                    Span::styled("Open your browser and visit:", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("     ", Style::default()),
                    Span::styled(&self.web_url, Style::default().fg(Color::Rgb(100, 181, 246)).bold()),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  3. ", Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
                    Span::styled("Enjoy real-time monitoring!", Style::default().fg(Color::White)),
                ]),
                Line::default(),
                Line::default(),
                Line::from(vec![
                    Span::styled("  ✨ ", Style::default()),
                    Span::styled("FEATURES:", Style::default().fg(Color::Rgb(160, 160, 180))),
                ]),
                Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled("Real-time sensor monitoring", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled("Live charts and graphs", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled("Manual pump control", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("     • ", Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled("WebSocket real-time updates", Style::default().fg(Color::White)),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("  💡 ", Style::default()),
                    Span::styled("NOTE: ", Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
                    Span::styled("Use 127.0.0.1 or localhost, not 0.0.0.0", Style::default().fg(Color::Rgb(160, 160, 180))),
                ]),
                Line::default(),
            ]
        };
        f.render_widget(
            Paragraph::new(info_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                    .title(Span::styled(" 📖 Instructions ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
            ),
            chunks[2],
        );
        
        // ── Footer ──
        let footer_keys = if self.web_server_started {
            &["q/Esc", "Back to Menu"][..]
        } else {
            &["s", "Start Server", "q/Esc", "Back to Menu"][..]
        };
        f.render_widget(make_footer(footer_keys), chunks[3]);
    }

    // ── Render: AI Agent ────────────────────────────────────────

    #[allow(clippy::too_many_lines)]
    fn draw_ai_agent(&self, f: &mut Frame) {
        let area = f.size();
        // Outer margin
        let outer = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(area);
        let inner = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]).split(outer[1]);
        let content = inner[1];

        let chunks = Layout::vertical([
            Constraint::Length(7),  // Header
            Constraint::Length(1),  // Separator
            Constraint::Min(10),    // Chat history
            Constraint::Length(4),  // Input box
            Constraint::Length(3),  // Footer
        ]).split(content);

        // ── Header ──
        let header = vec![
            Line::default(),
            Line::from(vec![
                Span::styled("  🤖 ", Style::default()),
                Span::styled("AI AGENT MODE", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                Span::styled("  │  ", Style::default().fg(Color::Rgb(60, 60, 80))),
                Span::styled("AgroAI Assistant", Style::default().fg(Color::Rgb(160, 160, 180))),
            ]),
            Line::default(),
            Line::from(vec![
                Span::styled("  Mode:   ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled(format!("{:?}", self.ai_provider.as_ref().unwrap_or(&crate::ai::AiProvider::Simulation)), Style::default().fg(Color::Rgb(255, 213, 79)).bold()),
            ]),
            Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled("● Active", Style::default().fg(Color::Rgb(0, 200, 83)).bold()),
            ]),
        ];
        f.render_widget(
            Paragraph::new(header).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
            ),
            chunks[0],
        );

        // ── Separator ──
        let sep = Paragraph::new(Line::styled(
            "─".repeat(content.width as usize),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(sep, chunks[1]);

        // ── Chat History ──
        let mut chat_lines = vec![];
        
        if self.ai_messages.is_empty() {
            chat_lines.push(Line::default());
            chat_lines.push(Line::from(vec![
                Span::styled("  💡 ", Style::default()),
                Span::styled("Start chatting! Try: 'status', 'help', or 'how are my plants?'", Style::default().fg(Color::Rgb(160, 160, 180))),
            ]));
            chat_lines.push(Line::default());
        } else {
            // Show last messages (scrollable in future)
            let visible_count = ((chunks[2].height as usize).saturating_sub(2)) / 4; // Rough estimate
            let start_idx = self.ai_messages.len().saturating_sub(visible_count);
            
            for msg in &self.ai_messages[start_idx..] {
                chat_lines.push(Line::default());
                
                if msg.role == "user" {
                    chat_lines.push(Line::from(vec![
                        Span::styled("  👤 ", Style::default()),
                        Span::styled("You", Style::default().fg(Color::Rgb(100, 181, 246)).bold()),
                        Span::styled(format!("  [{}]", msg.timestamp), Style::default().fg(Color::Rgb(100, 100, 120))),
                    ]));
                    chat_lines.push(Line::from(vec![
                        Span::styled("     ", Style::default()),
                        Span::styled(&msg.text, Style::default().fg(Color::White)),
                    ]));
                } else {
                    chat_lines.push(Line::from(vec![
                        Span::styled("  🤖 ", Style::default()),
                        Span::styled("AgroAI", Style::default().fg(Color::Rgb(0, 200, 83)).bold()),
                        Span::styled(format!("  [{}]", msg.timestamp), Style::default().fg(Color::Rgb(100, 100, 120))),
                    ]));
                    
                    // Word wrap AI response
                    let max_width = (content.width as usize).saturating_sub(8);
                    for line in msg.text.lines() {
                        if line.is_empty() {
                            chat_lines.push(Line::default());
                        } else {
                            let mut current = String::new();
                            for word in line.split_whitespace() {
                                if current.len() + word.len() + 1 > max_width && !current.is_empty() {
                                    chat_lines.push(Line::from(vec![
                                            Span::styled("     ", Style::default()),
                                            Span::styled(current.clone(), Style::default().fg(Color::Rgb(200, 200, 200))),
                                        ]));
                                        current.clear();
                                }
                                if !current.is_empty() {
                                    current.push(' ');
                                }
                                current.push_str(word);
                            }
                            if !current.is_empty() {
                                chat_lines.push(Line::from(vec![
                                    Span::styled("     ", Style::default()),
                                    Span::styled(current, Style::default().fg(Color::Rgb(200, 200, 200))),
                                ]));
                            }
                        }
                    }
                }
            }
        }
        
        f.render_widget(
            Paragraph::new(chat_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                        .title(Span::styled(" 💬 Chat History ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
                )
                .wrap(Wrap { trim: false }),
            chunks[2],
        );

        // ── Input Box ──
        let input_display = if self.ai_processing {
            "⏳ Processing...".to_string()
        } else {
            format!("{}▏", self.ai_input)
        };
        
        let input_color = if self.ai_processing {
            Color::Rgb(255, 213, 79)
        } else {
            Color::White
        };
        
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(input_display, Style::default().fg(input_color)),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(if self.ai_processing { Color::Rgb(255, 213, 79) } else { Color::Rgb(0, 200, 83) }))
                    .title(Span::styled(" 💬 Your Message ", Style::default().fg(Color::Rgb(100, 255, 218))))
            ),
            chunks[3],
        );

        // ── Footer ──
        f.render_widget(make_footer(&["Enter", "Send", "q/Esc", "Back to Menu"]), chunks[4]);
    }

    // ── Render: Daemon Monitor ──────────────────────────────────

    fn draw_daemon_monitor(&self, f: &mut Frame) {
        let area = f.size();
        // Outer margin
        let outer = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(area);
        let inner = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]).split(outer[1]);
        let content = inner[1];

        let chunks = Layout::vertical([
            Constraint::Length(8),  // Header with stats
            Constraint::Length(1),  // Separator
            Constraint::Min(10),    // Logs
            Constraint::Length(3),  // Footer
        ]).split(content);

        // ── Header ──
        let status_text = if self.daemon_running { "● RUNNING" } else { "○ PAUSED" };
        let status_color = if self.daemon_running { Color::Rgb(0, 200, 83) } else { Color::Rgb(255, 213, 79) };
        
        let header = vec![
            Line::default(),
            Line::from(vec![
                Span::styled("  🤖 ", Style::default()),
                Span::styled("DAEMON AUTOMATION", Style::default().fg(Color::Rgb(100, 255, 218)).bold()),
                Span::styled("  │  ", Style::default().fg(Color::Rgb(60, 60, 80))),
                Span::styled(status_text, Style::default().fg(status_color).bold()),
            ]),
            Line::default(),
            Line::from(vec![
                Span::styled("  Cycle:     ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled(format!("#{}", self.daemon_cycle_count), Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::styled("  Interval:  ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled("5 seconds", Style::default().fg(Color::Rgb(255, 213, 79))),
            ]),
            Line::from(vec![
                Span::styled("  Logs:      ", Style::default().fg(Color::Rgb(160, 160, 180))),
                Span::styled(format!("{} entries", self.daemon_logs.len()), Style::default().fg(Color::Rgb(100, 181, 246))),
            ]),
        ];
        f.render_widget(
            Paragraph::new(header).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
            ),
            chunks[0],
        );

        // ── Separator ──
        let sep = Paragraph::new(Line::styled(
            "─".repeat(content.width as usize),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(sep, chunks[1]);

        // ── Logs ──
        let mut log_lines = vec![];
        
        if self.daemon_logs.is_empty() {
            log_lines.push(Line::default());
            log_lines.push(Line::from(vec![
                Span::styled("  📋 ", Style::default()),
                Span::styled("No logs yet. Press 's' to start daemon.", Style::default().fg(Color::Rgb(160, 160, 180))),
            ]));
            log_lines.push(Line::default());
        } else {
            // Show last logs (scrollable in future)
            let visible_count = (chunks[2].height as usize).saturating_sub(2);
            let start_idx = self.daemon_logs.len().saturating_sub(visible_count);
            
            for log in &self.daemon_logs[start_idx..] {
                let (icon, color) = match log.level.as_str() {
                    "success" => ("✅", Color::Rgb(0, 200, 83)),
                    "warning" => ("⚠️ ", Color::Rgb(255, 213, 79)),
                    "error" => ("❌", Color::Rgb(244, 67, 54)),
                    _ => ("ℹ️ ", Color::Rgb(100, 181, 246)),
                };
                
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  [{}] ", log.timestamp), Style::default().fg(Color::Rgb(100, 100, 120))),
                    Span::styled(icon, Style::default()),
                    Span::styled(" ", Style::default()),
                    Span::styled(&log.message, Style::default().fg(color)),
                ]));
            }
        }
        
        f.render_widget(
            Paragraph::new(log_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                        .title(Span::styled(" 📜 Activity Log ", Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
                )
                .wrap(Wrap { trim: false }),
            chunks[2],
        );

        // ── Footer ──
        f.render_widget(make_footer(&["s", "Start/Stop", "c", "Clear Logs", "q/Esc", "Back to Menu"]), chunks[3]);
    }
}

// ── Helper: themed block ───────────────────────────────────────────

fn make_block(title: &str) -> Block<'static> {
    let b = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)));
    if title.is_empty() {
        b
    } else {
        b.title(Span::styled(title.to_string(), Style::default().fg(Color::Rgb(100, 255, 218)).bold()))
    }
}

// ── Helper: footer bar ─────────────────────────────────────────────

fn make_footer(pairs: &[&str]) -> Paragraph<'static> {
    let mut spans: Vec<Span<'static>> = vec![Span::styled(" ", Style::default())];
    for chunk in pairs.chunks(2) {
        if !chunk[0].is_empty() {
            spans.push(Span::styled(
                chunk[0].to_string(),
                Style::default().fg(Color::Rgb(100, 255, 218)).bold(),
            ));
            spans.push(Span::styled(" ", Style::default()));
        }
        if chunk.len() > 1 && !chunk[1].is_empty() {
            spans.push(Span::styled(
                chunk[1].to_string(),
                Style::default().fg(Color::Rgb(120, 120, 140)),
            ));
            spans.push(Span::styled("  │  ", Style::default().fg(Color::Rgb(40, 40, 60))));
        }
    }
    Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
    )
}

// ── Public entry point ─────────────────────────────────────────────

/// Memulai antarmuka pengguna terminal (TUI).
///
/// # Errors
///
/// Mengembalikan kesalahan jika terminal tidak dapat diinisialisasi atau loop event gagal.
pub async fn run_tui(state: crate::web::AppState, cancel_token: CancellationToken) -> Result<ExitSignal> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(state, cancel_token);
    
    let mut event_stream = crossterm::event::EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(50));

    loop {
        if app.needs_redraw {
            terminal.draw(|f| app.render(f))?;
            app.needs_redraw = false;
        }

        if let Some(signal) = app.exit_signal.take() {
            // Restore terminal before exiting
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            return Ok(signal);
        }

        tokio::select! {
            _ = tick_interval.tick() => {
                app.tick().await;
            }
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        app.handle_key(key).await;
                        app.tick().await;
                    }
                    Some(Ok(_)) => {
                        // Ignore other events for now, but tick anyway to run background tasks
                        app.tick().await;
                    }
                    Some(Err(e)) => return Err(e.into()),
                    None => break,
                }
            }
        }
    }
    
    // In case the loop breaks, ensure terminal is restored
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(ExitSignal::Quit)
}

