use std::env;
use std::time::Duration;
use std::fmt::Write; // Added for efficient string building
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::db::Database;
use crate::hardware::water_plant;
use anyhow::{Context, Result};
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "AgroCLI Premium Bot - Commands:"
)]
pub enum Command {
    #[command(description = "Open the main menu")]
    Start,
    #[command(description = "Quick garden status")]
    Status,
    #[command(description = "Weather info")]
    Weather,
    #[command(description = "Watering menu")]
    Water,
    #[command(description = "Show help information")]
    Help,
}

pub async fn run_bot(db: Arc<Database>, ct: CancellationToken) -> Result<()> {
    let token = env::var("TELEGRAM_BOT_TOKEN").context("TELEGRAM_BOT_TOKEN not set")?;
    let bot = Bot::new(token);

    println!("🤖 Premium Telegram Bot Listener Active.");

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(cmd_handler),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db])
        .build();

    tokio::select! {
        () = ct.cancelled() => {
            info!("Telegram bot shutting down...");
        }
        () = dispatcher.dispatch() => {}
    }

    Ok(())
}

async fn cmd_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    db: Arc<Database>,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Start => {
            let keyboard = InlineKeyboardMarkup::new(vec![
                vec![
                    InlineKeyboardButton::callback("📊 Status", "status_refresh"),
                    InlineKeyboardButton::callback("🚿 Water Now", "water_menu"),
                ],
                vec![
                    InlineKeyboardButton::callback("🌤 Weather", "weather_info"),
                    InlineKeyboardButton::callback("❓ Help", "help"),
                ],
            ]);

            bot.send_message(
                msg.chat.id,
                "🌿 <b>AgroCLI Premium Garden Console</b>\nWelcome! Use the buttons below to control your garden.",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Status => {
            send_status_report(&bot, msg.chat.id, &db).await?;
        }
        Command::Weather => {
            send_weather_info(&bot, msg.chat.id).await?;
        }
        Command::Water => {
            send_watering_menu(&bot, msg.chat.id, &db).await?;
        }
    }
    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    db: Arc<Database>,
) -> Result<(), teloxide::RequestError> {
    if let Some(data) = q.data {
        match data.as_str() {
            "status_refresh" => {
                send_status_report(&bot, q.from.id.into(), &db).await?;
            }
            "water_menu" => {
                send_watering_menu(&bot, q.from.id.into(), &db).await?;
            }
            "help" => {
                bot.send_message(q.from.id, Command::descriptions().to_string())
                    .await?;
            }
            "weather_info" => {
                send_weather_info(&bot, q.from.id.into()).await?;
            }
            d if d.starts_with("water:") => {
                let plant_name = &d[6..];
                water_plant(plant_name, 3).await;
                let _ = db.update_care(plant_name, crate::core::CareType::Water).await;

                bot.answer_callback_query(q.id.clone())
                    .text(format!("🚿 Watering {plant_name}..."))
                    .await?;
                bot.send_message(
                    q.from.id,
                    format!(
                        "✅ <b>Watering Success</b>\nTriggered pump for <b>{plant_name}</b>"
                    ),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
            _ => {}
        }
    }
    bot.answer_callback_query(q.id).await?;
    Ok(())
}

/// Sends an interactive garden report to a specific chat.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
async fn send_status_report(
    bot: &Bot,
    chat_id: ChatId,
    db: &Arc<Database>,
) -> Result<(), teloxide::RequestError> {
    let plants = db.active_plants().await.unwrap_or_default();
    if plants.is_empty() {
        bot.send_message(chat_id, "🪴 Your garden is empty.")
            .await?;
        return Ok(());
    }

    let mut report = String::from("📊 <b>AGRO GARDEN REPORT</b>\n\n");
    for p in plants {
        let moisture = crate::hardware::read_soil_moisture(&p.name);
        let bar_chars = (moisture / 10.0) as usize;
        let bar_filled = "█".repeat(bar_chars);
        let bar_empty = "░".repeat(10 - bar_chars);

        let status = if moisture < 30.0 {
            "THIRSTY"
        } else {
            "HEALTHY"
        };
        let _ = write!(
            report,
            "🌿 <b>{p_name}</b>\n<code>[{bar_filled}{bar_empty}]</code> {moisture:.1}% ({status})\n\n",
            p_name = p.name
        );
    }

    bot.send_message(chat_id, report)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

/// Fetches and sends real-time weather information to a specific chat.
async fn send_weather_info(bot: &Bot, chat_id: ChatId) -> Result<(), teloxide::RequestError> {
    let city = env::var("CITY").unwrap_or_else(|_| "Surabaya".to_string());
    let api_key = env::var("WEATHER_API_KEY").unwrap_or_else(|_| String::new());

    if api_key.is_empty() {
        bot.send_message(chat_id, "⚠️ Weather API Key not configured in .env")
            .await?;
        return Ok(());
    }

    match crate::core::weather(&city, &api_key).await {
        Some((cond, temp)) => {
            let icon = match cond.to_lowercase().as_str() {
                c if c.contains("rain") => "🌧",
                c if c.contains("cloud") => "☁️",
                c if c.contains("clear") => "☀️",
                _ => "🌤",
            };
            bot.send_message(
                chat_id,
                format!(
                    "{icon} <b>Weather in {city}</b>\nCondition: {cond}\nTemperature: {temp:.1}°C"
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        None => {
            bot.send_message(chat_id, "❌ Failed to fetch weather data.")
                .await?;
        }
    }
    Ok(())
}

async fn send_watering_menu(
    bot: &Bot,
    chat_id: ChatId,
    db: &Arc<Database>,
) -> Result<(), teloxide::RequestError> {
    let plants = db.active_plants().await.unwrap_or_default();
    if plants.is_empty() {
        bot.send_message(chat_id, "🪴 No plants to water.").await?;
        return Ok(());
    }

    let mut buttons = Vec::new();
    for p in plants {
        buttons.push(vec![InlineKeyboardButton::callback(
            format!("💧 Water {p_name}", p_name = p.name),
            format!("water:{p_name}", p_name = p.name),
        )]);
    }

    let keyboard = InlineKeyboardMarkup::new(buttons);
    bot.send_message(
        chat_id,
        "🚿 <b>Watering Control</b>\nSelect a plant to trigger the pump:",
    )
    .parse_mode(ParseMode::Html)
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

pub async fn send_telegram_alert(db: &Database, message: &str) -> Result<()> {
    let token = env::var("TELEGRAM_BOT_TOKEN");
    let chat_id = env::var("TELEGRAM_CHAT_ID");

    if let (Ok(t), Ok(c)) = (token, chat_id) {
        let bot = Bot::new(t);

        // If it's a critical alert, add a button
        let request = if message.contains("LOW") || message.contains("DUE") {
            let plant_name = message
                .split(':')
                .next()
                .unwrap_or("")
                .trim()
                .trim_start_matches(|c: char| !c.is_alphanumeric())
                .trim();

            let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "💧 Water 3s",
                format!("water:{plant_name}"),
            )]]);

            bot.send_message(c, message)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
        } else {
            bot.send_message(c, message)
                .parse_mode(ParseMode::Html)
        };

        // IoT Reliability: Offline-first queueing
        match request.await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Silenced to info to avoid TUI/Log clutter since it's queued anyway
                info!("Telegram delivery failed ({}), message queued locally.", e);
                db.queue_alert(message).await.map_err(|e| anyhow::anyhow!(e))
            }
        }
    } else {
        info!("Telegram config missing, alert queued locally.");
        db.queue_alert(message).await.map_err(|e| anyhow::anyhow!(e))
    }
}

/// Background task to process the pending alert queue.
pub async fn process_alert_queue(db: Arc<Database>) {
    loop {
        let alerts = db.pending_alerts().await.unwrap_or_default();
        if !alerts.is_empty() {
            info!("Processing {} pending alerts...", alerts.len());
            for (id, msg) in alerts {
                if send_direct(&msg).await.is_ok() {
                    let _ = db.delete_alert(id).await;
                } else {
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn send_direct(message: &str) -> Result<()> {
    let token = env::var("TELEGRAM_BOT_TOKEN").context("No token")?;
    let chat_id = env::var("TELEGRAM_CHAT_ID").context("No chat id")?;
    let bot = Bot::new(token);
    bot.send_message(chat_id, message).await?;
    Ok(())
}
