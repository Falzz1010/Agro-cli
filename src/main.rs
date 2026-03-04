mod ai;
mod core;
mod db;
mod hardware;
mod telegram;
mod tui;
mod web;

use std::env;
use std::sync::Arc;

use anyhow::{Context, Result};
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode};
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{info, error, instrument};
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

use crate::core::{CareType, calculate_today_tasks, get_weather};
use crate::db::Database;
use crate::hardware::{read_humidity, read_soil_moisture, read_temperature, water_plant};
use crate::telegram::{run_bot as start_telegram_bot, send_telegram_alert};
use crate::web::serve as start_web_server;

/// Checks if the 'q' key has been pressed in a non-blocking way.
fn should_exit() -> bool {
    if event::poll(Duration::from_millis(10)).unwrap_or(false) {
        match event::read() {
            Ok(Event::Key(key)) => {
                return key.code == KeyCode::Char('q') || key.code == KeyCode::Esc;
            }
            _ => return false,
        }
    }
    false
}

const VERSION: &str = "1.2.0";
const BANNER: &str = r"
    █████╗  ██████╗ ██████╗  ██████╗  ██████╗██╗     ██╗
   ██╔══██╗██╔════╝ ██╔══██╗██╔═══██╗██╔════╝██║     ██║
   ███████║██║  ███╗██████╔╝██║   ██║██║     ██║     ██║
   ██╔══██║██║   ██║██╔══██╗██║   ██║██║     ██║     ██║
   ██║  ██║╚██████╔╝██║  ██║╚██████╔╝╚██████╗███████╗██║
   ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚══════╝╚═╝
";

/// Displays the `AgroCLI` ASCII banner and version info.
fn display_banner() {
    println!("{}", BANNER.bright_green());
    println!(
        "{}",
        "   === THE INTELLIGENT GARDEN BRAIN (RUST) ===".bright_cyan()
    );
    println!(
        "   v{} | {}",
        VERSION.yellow(),
        "Local Network Node Active".bright_blue()
    );
    println!(
        "   Made with {} by {}",
        "💚".green(),
        "Naufal Rizky".bright_white().bold()
    );
    println!();
}

#[derive(clap::Parser)]
#[command(name = "AgroCLI")]
#[command(about = "The Intelligent Garden Brain - Rust Edition", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize the garden database
    Init,
    /// Add a new plant
    Add {
        /// Type of plant (e.g., tomato, basil)
        #[arg(short, long)]
        plant_type: String,
        /// Unique nickname for the plant
        #[arg(short, long)]
        name: String,
    },
    /// Check tasks for today
    Today {
        /// City for weather data
        #[arg(short, long)]
        city: Option<String>,
        /// Automatically mark tasks as done
        #[arg(short, long)]
        mark_done: bool,
    },
    /// Interactive menu mode
    Interactive,
    /// Run the automation daemon
    Daemon,
    /// Start the web dashboard
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("AgroCLI starting up...");

    // Load environment variables from .env file
    let _ = dotenvy::dotenv();

    let cli = <Cli as clap::Parser>::parse();
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "data/garden.db".to_string());
    let db_url = format!("sqlite://{db_path}");

    // Auto-init if DB doesn't exist
    if !std::path::Path::new(&db_path).exists() {
        let db = Database::new(&db_url)
            .await
            .context("Failed to connect to database")?;
        db.init().await.context("Failed to initialize database")?;
    }

    let db = Arc::new(
        Database::new(&db_url)
            .await
            .context("Failed to connect to database")?,
    );

    // Ensure schema is up to date on every run
    db.init().await.context("Failed to initialize database schema")?;

    let global_token = CancellationToken::new();

    match cli.command {
        Some(Commands::Init) => {
            db.init().await.context("Failed to initialize database")?;
            println!("🌱 AgroCLI Initialized!");
        }
        Some(Commands::Add { plant_type, name }) => {
            let success = db
                .add_plant(&plant_type, &name)
                .await
                .context("Failed to add plant")?;
            if success {
                println!("✅ Added {name} ({plant_type}) to your garden.");
            } else {
                println!("❌ A plant named '{name}' already exists.");
            }
        }
        Some(Commands::Today { city: _, mark_done }) => {
            let active_plants = db
                .get_all_active_plants()
                .await
                .context("Failed to get plants")?;
            let tasks = calculate_today_tasks(&active_plants, None, None);

            if tasks.is_empty() {
                println!("✨ All caught up! No tasks needed today.");
                return Ok(());
            }

            for task in &tasks {
                let water_str = if task.needs_water {
                    format!("💧 {}ml", task.water_ml)
                } else {
                    "OK".to_string()
                };
                let fert_str = if task.needs_fertilizer {
                    "🌾 Yes"
                } else {
                    "OK"
                };
                println!(
                    "{}: Water: {}, Fertilize: {}",
                    task.name, water_str, fert_str
                );
            }

            if mark_done {
                for task in tasks {
                    if task.needs_water {
                        db.update_care(&task.name, CareType::Water)
                            .await
                            .context("Failed to update water record")?;
                    }
                    if task.needs_fertilizer {
                        db.update_care(&task.name, CareType::Fertilizer)
                            .await
                            .context("Failed to update fertilizer record")?;
                    }
                }
                println!("✅ All tasks marked as completed!");
            }
        }
        Some(Commands::Interactive) | None => {
            display_banner();
            run_tui_loop(Arc::clone(&db), &global_token).await?;
        }
        Some(Commands::Daemon) => {
            run_daemon_direct(Arc::clone(&db), &global_token).await?;
        }
        Some(Commands::Serve) => {
            run_web_direct(Arc::clone(&db), &global_token, false).await?;
        }
    }
    Ok(())
}

/// Runs the TUI in a loop, re-entering after external operations.
async fn run_tui_loop(db: Arc<Database>, cancel_token: &CancellationToken) -> Result<()> {
    match tui::run_tui(Arc::clone(&db), cancel_token.clone()).await? {
        tui::ExitSignal::Quit => {
            cancel_token.cancel();
            println!("Happy Farming! Goodbye. 👋");
        }
    }
    Ok(())
}

/// Helper function to run the daemon loop directly.
async fn run_daemon_direct(db_arc: Arc<Database>, cancel_token: &CancellationToken) -> Result<()> {
    
    info!("AgroCLI Daemon Activated.");
    println!(
        "{}",
        "🔌 AgroCLI Daemon Activated. (Press 'q' to return to menu/exit)"
            .bright_green()
            .bold()
    );

    // Start Telegram Bot in background if token exists
    if env::var("TELEGRAM_BOT_TOKEN").is_ok() {
        let bot_db = Arc::clone(&db_arc);
        let ct = cancel_token.clone();
        tokio::spawn(async move {
            tokio::select! {
                () = ct.cancelled() => info!("Telegram bot task cancelled"),
                res = start_telegram_bot(bot_db, ct.clone()) => {
                    if let Err(e) = res {
                        error!("Telegram Bot Error: {}", e);
                    }
                }
            }
        });
    }

    let api_key = env::var("WEATHER_API_KEY").unwrap_or_else(|_| "default_key".to_string());
    let city = env::var("CITY").unwrap_or_else(|_| "Surabaya".to_string());
    let client = reqwest::Client::new();
    
    loop {
        if should_exit() {
            cancel_token.cancel();
            break;
        }
        
        println!("\nCycle Check: {}", chrono::Local::now().format("%H:%M:%S"));
        info!("Starting automation cycle...");
        let weather_info = get_weather(&city, &api_key).await;
        let weather_cond = weather_info.as_ref().map(|(cond, _)| cond.as_str());
        
        let active_plants = db_arc
            .get_all_active_plants()
            .await
            .context("Failed to get plants")?;
            
        let mut set = JoinSet::new();
        let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
        let broadcast_url = format!("http://127.0.0.1:{port}/api/broadcast/sensor");

        for plant in active_plants {
            let db_inner = Arc::clone(&db_arc);
            let client_inner = client.clone();
            let b_url = broadcast_url.clone();
            let w_cond = weather_cond.map(std::string::ToString::to_string);
            
            set.spawn(async move {
                process_plant_automation(plant, db_inner, client_inner, b_url, w_cond).await
            });
        }

        while let Some(res) = set.join_next().await {
            if let Err(e) = res {
                error!("Automation task panicked: {}", e);
            } else if let Ok(Err(e)) = res {
                error!("Automation task failed: {}", e);
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
    Ok(())
}

/// Helper function to run the web server directly.
async fn run_web_direct(
    db_arc: Arc<Database>,
    cancel_token: &CancellationToken,
    background: bool,
) -> Result<()> {
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    
    // Display user-friendly URL (convert 0.0.0.0 to 127.0.0.1)
    let display_host = if host == "0.0.0.0" { "127.0.0.1" } else { &host };
    
    info!(host = %host, port = %port, "Starting web dashboard");
    println!("🌐 Starting Web Dashboard...");
    println!("   Server binding to: {host}:{port}");
    println!("   Access in browser: http://{display_host}:{port}");
    
    if host == "0.0.0.0" {
        println!("   💡 Note: Use 127.0.0.1 or localhost in your browser, not 0.0.0.0");
    }
    
    let ct = cancel_token.clone();

    if background {
        tokio::spawn(async move {
            if let Err(e) = start_web_server(db_arc, ct).await {
                error!("Web server error: {}", e);
            }
        });
        println!("✅ Dashboard is now running in the background.");
    } else {
        start_web_server(db_arc, ct).await?;
    }

    Ok(())
}

#[instrument(skip(db, client, broadcast_url))]
async fn process_plant_automation(
    plant: crate::core::Plant,
    db: Arc<Database>,
    client: reqwest::Client,
    broadcast_url: String,
    weather_cond: Option<String>,
) -> Result<()> {
    let name = &plant.name;
    let moisture = read_soil_moisture(name);
    let temp = read_temperature();
    let humidity = read_humidity();

    db.log_sensor_data(name, moisture, temp, humidity).await?;

    // Broadcast to Web Dashboard with timeout
    let _ = client
        .post(&broadcast_url)
        .timeout(Duration::from_secs(1))
        .json(&crate::web::SensorData {
            plant_name: name.clone(),
            moisture,
            temperature: temp,
            humidity,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            min_moisture: plant.min_moisture,
            water_ml: plant.water_ml,
        })
        .send()
        .await;

    let tasks = calculate_today_tasks(
        std::slice::from_ref(&plant),
        weather_cond.as_deref(),
        Some(moisture),
    );

    if !tasks.is_empty() && tasks[0].needs_water {
        let alert_msg = format!("🚨 *{name}*: Moisture LOW ({moisture:.1}%). Triggering pump!");
        println!(
            "{} {}: Moisture {} ({:.1}%). {}",
            "🚨".red(),
            name.bright_white().bold(),
            "LOW".red().bold(),
            moisture.to_string().red(),
            "Triggering pump!".bright_blue()
        );
        info!(plant = %name, moisture = %moisture, "Moisture LOW. Triggering pump!");
        
        water_plant(name, 3).await;
        db.update_care(name, CareType::Water).await?;

        // Send Telegram Alert
        let _ = send_telegram_alert(&alert_msg).await;
    } else {
        println!(
            "{} {}: Moisture {:.1}% ({})",
            "✅".green(),
            name.bright_white(),
            moisture,
            "OK".green().bold()
        );
        info!(plant = %name, moisture = %moisture, "Moisture OK");
    }

    // Fertilizer alert check
    if !tasks.is_empty() && tasks[0].needs_fertilizer {
        let fert_msg = format!("🌾 *{name}*: Fertilizer is due! Please fertilize your plant.");
        println!(
            "{} {}: {} {}",
            "🌾".yellow(),
            name.bright_white().bold(),
            "Fertilizer DUE".bright_yellow().bold(),
            "— please fertilize!".bright_white()
        );
        info!(plant = %name, "Fertilizer is due");
        let _ = send_telegram_alert(&fert_msg).await;
    }

    Ok(())
}
