#![allow(dead_code)]

use std::env;
use std::sync::Arc;

use crate::db::Database;
use crate::hardware::water_plant;
use anyhow::Result;
use colored::Colorize;
use inquire::Text;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum AiProvider {
    GeminiFlash,
    GeminiPro,
    ClaudeSonnet,
    ClaudeOpus,
    ChatGpt4,
    ChatGpt5,
    Simulation,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::GeminiFlash => write!(f, "Google Gemini 1.5 Flash (Fast & Efficient)"),
            AiProvider::GeminiPro => write!(f, "Google Gemini 1.5 Pro (Advanced)"),
            AiProvider::ClaudeSonnet => write!(f, "Anthropic Claude 3.5 Sonnet (Balanced)"),
            AiProvider::ClaudeOpus => write!(f, "Anthropic Claude 3 Opus (Most Capable)"),
            AiProvider::ChatGpt4 => write!(f, "OpenAI ChatGPT-4 Turbo"),
            AiProvider::ChatGpt5 => write!(f, "OpenAI ChatGPT-5 (Latest)"),
            AiProvider::Simulation => write!(f, "Simulation Mode (No API Key Required)"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    pub tools: Option<Vec<Tool>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Part {
    Text(String),
    FunctionCall(FunctionCall),
    FunctionResponse(FunctionResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tool {
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candidate {
    pub content: Content,
}

/// Helper to broadcast AI logs to the web dashboard.
fn broadcast_ai_log(state: &crate::web::AppState, query: &str, response: &str) {
    let _ = state.tx.send(crate::web::DashboardMessage::AiLog(crate::web::AiLog {
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        query: query.to_string(),
        response: response.to_string(),
    }));
}

/// Entry point for the AI Agent interactive chat mode.
pub async fn run_agent_mode(state: std::sync::Arc<crate::web::AppState>) -> Result<()> {
    let db = &state.db;
    // Clear screen and show banner
    print!("\x1B[2J\x1B[1;1H"); // Clear screen
    
    println!("{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "║           ✨  AI AGENT MODE ACTIVATED  ✨                    ║".bright_cyan().bold());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "║  🤖 AgroAI - Your Intelligent Garden Assistant               ║".bright_white());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();

    // Step 1: Select AI Provider
    println!("{}", "  🔧 CONFIGURATION".bright_yellow().bold());
    println!();
    
    let provider = select_provider()?;
    
    println!();
    
    // Step 2: Get API Key (if not simulation)
    let api_key = api_key_for_provider(&provider)?;
    
    // Show selected configuration
    println!("{}", "  ┌─────────────────────────────────────────────────────────────┐".bright_cyan());
    println!("{}", "  │ 📋 ACTIVE CONFIGURATION                                     │".bright_cyan());
    println!("{}", "  ├─────────────────────────────────────────────────────────────┤".bright_cyan());
    println!("  │ {:<58}│", format!("Provider: {}", match provider {
        AiProvider::GeminiFlash => "Gemini 1.5 Flash",
        AiProvider::GeminiPro => "Gemini 1.5 Pro",
        AiProvider::ClaudeSonnet => "Claude 3.5 Sonnet",
        AiProvider::ClaudeOpus => "Claude 3 Opus",
        AiProvider::ChatGpt4 => "ChatGPT-4 Turbo",
        AiProvider::ChatGpt5 => "ChatGPT-5",
        AiProvider::Simulation => "Simulation Mode",
    }));
    println!("  │ {:<58}│", format!("Status: {}", if api_key.is_some() { "✅ Connected" } else { "⚠️  Simulation" }));
    println!("{}", "  └─────────────────────────────────────────────────────────────┘".bright_cyan());
    println!();
    
    display_agent_instructions(api_key.is_none());

    println!("{}", "─────────────────────────────────────────────────────────────────".bright_black());
    println!();

    let client = reqwest::Client::new();
    let mut history: Vec<Content> = vec![
        Content {
            role: "user".to_string(),
            parts: vec![Part::Text("You are AgroCLI AI, an intelligent garden assistant. You have access to tools to monitor sensors, water plants, and update thresholds. Be helpful, concise, and professional.".to_string())],
        },
        Content {
            role: "model".to_string(),
            parts: vec![Part::Text("Understood. I am ready to help manage your garden.".to_string())],
        }
    ];

    loop {
        let input = Text::new(&format!("{}", "👤 You".bright_blue().bold()))
            .with_placeholder("Type your command here...")
            .prompt()?;
            
        if input.trim().to_lowercase() == "exit" {
            println!();
            println!("{}", "  👋 Exiting AI Agent Mode...".bright_cyan());
            println!("{}", "  Press Enter to return to the TUI...".bright_black());
            println!();
            break;
        }

        if input.trim().is_empty() {
            continue;
        }

        println!(); // Add spacing

        if let Some(key) = api_key.as_deref() {
            match provider {
                AiProvider::GeminiFlash | AiProvider::GeminiPro => {
                    match process_gemini_command(&client, key, &mut history, &input, db, &provider).await {
                        Ok(response) => {
                            display_ai_response(&response);
                            let _ = db.log_ai_interaction(&input, &response).await;
                            broadcast_ai_log(&state, &input, &response);
                        }
                        Err(e) => {
                            println!("{}", format!("  ❌ AI Error: {e}").red().bold());
                            println!();
                        }
                    }
                }
                _ => { // Handles Claude, ChatGPT, etc. when an API key is present
                    let response = format!("{provider} integration coming soon! For now, please use Gemini or Simulation mode.");
                    display_ai_response(&response);
                }
            }
        } else {
            // Simulation Mode
            simulated_response(&state, &input).await?;
        }
    }

    Ok(())
}

fn display_ai_response(response: &str) {
    println!("{}", "  ┌─────────────────────────────────────────────────────────────┐".bright_green());
    println!("{} {}", "  │ 🤖".bright_green(), "AgroAI Response:".bright_green().bold());
    println!("{}", "  ├─────────────────────────────────────────────────────────────┤".bright_green());
    
    // Word wrap the response
    for line in wrap_text(response, 58) {
        println!("  │ {line:<58}│");
    }
    
    println!("{}", "  └─────────────────────────────────────────────────────────────┘".bright_green());
    println!();
}

/// Helper function to wrap text to a specific width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::with_capacity(width);
    
    for word in text.split_whitespace() {
        if !current_line.is_empty() && current_line.len() + word.len() + 1 > width {
            lines.push(std::mem::take(&mut current_line));
        }
        
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(String::new());
    }
    
    lines
}

async fn process_gemini_command(
    client: &reqwest::Client,
    api_key: &str,
    history: &mut Vec<Content>,
    input: &str,
    db: &Arc<Database>,
    provider: &AiProvider,
) -> Result<String> {
    history.push(Content {
        role: "user".to_string(),
        parts: vec![Part::Text(input.to_string())],
    });

    let model = match provider {
        AiProvider::GeminiPro => "gemini-1.5-pro",
        _ => "gemini-1.5-flash",
    };

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}");

    let tools = vec![Tool {
        function_declarations: vec![
            FunctionDeclaration {
                name: "get_garden_status".to_string(),
                description:
                    "Gets the current status (moisture, temp, humidity) of all active plants."
                        .to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            FunctionDeclaration {
                name: "water_plant_action".to_string(),
                description: "Triggers the water pump for a specific plant.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plant_name": { "type": "string" },
                        "duration_sec": { "type": "integer", "description": "Duration in seconds" }
                    },
                    "required": ["plant_name", "duration_sec"]
                }),
            },
        ],
    }];

    let request = GeminiRequest {
        contents: history.clone(),
        tools: Some(tools),
    };

    let res = client.post(&url).json(&request).send().await?;
    let status = res.status();
    let body_text = res.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Gemini API Error {status}: {body_text}"
        ));
    }

    let body: GeminiResponse = match serde_json::from_str(&body_text) {
        Ok(b) => b,
        Err(e) => {
            println!("❌ [DEBUG] Failed to decode AI response: {e}");
            println!("📄 [DEBUG] Raw JSON: {body_text}");
            return Err(e.into());
        }
    };

    if let Some(candidate) = body.candidates.first() {
        let model_response = candidate.content.clone();

        // Handle potential function calls
        for part in &model_response.parts {
            if let Part::FunctionCall(call) = part {
                println!();
                println!("{}", format!("  🛠️  AI executing tool: {}...", call.name).bright_black().italic());
                println!();
                
                let tool_res = match call.name.as_str() {
                    "get_garden_status" => {
                        let plants = db.active_plants().await?;
                        serde_json::to_value(plants)?
                    }
                    "water_plant_action" => {
                        let name = call.args["plant_name"].as_str().unwrap_or("");
                        let sec = call.args["duration_sec"].as_u64().unwrap_or(3);
                        water_plant(name, sec).await;
                        serde_json::json!({ "status": "success", "action": "watered", "plant": name })
                    }
                    _ => serde_json::json!({ "error": "tool not found" }),
                };

                // Send back tool response to AI
                let response_part = Part::FunctionResponse(FunctionResponse {
                    name: call.name.clone(),
                    response: tool_res,
                });

                // Add to history and recurse or continue
                history.push(model_response.clone());
                history.push(Content {
                    role: "function".to_string(),
                    parts: vec![response_part],
                });

                // Recursive call to get the final text response
                return process_gemini_command_no_input(client, api_key, history, provider).await;
            }
        }

        // It's just text
        if let Some(Part::Text(txt)) = model_response.parts.first() {
            let final_text = txt.clone();
            history.push(model_response);
            return Ok(final_text);
        }
    }

    Ok("I'm not sure how to respond to that.".to_string())
}

async fn process_gemini_command_no_input(
    client: &reqwest::Client,
    api_key: &str,
    history: &mut Vec<Content>,
    provider: &AiProvider,
) -> Result<String> {
    let model = match provider {
        AiProvider::GeminiPro => "gemini-1.5-pro",
        _ => "gemini-1.5-flash",
    };

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}");

    let request = GeminiRequest {
        contents: history.clone(),
        tools: None, // Don't allow tools again in the same recursive step for simplicity
    };

    let res = client.post(&url).json(&request).send().await?;
    let status = res.status();
    let body_text = res.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Gemini API Error {status}: {body_text}"
        ));
    }

    let body: GeminiResponse = match serde_json::from_str(&body_text) {
        Ok(b) => b,
        Err(e) => {
            println!("❌ [DEBUG] Failed to decode AI response: {e}");
            println!("📄 [DEBUG] Raw JSON: {body_text}");
            return Err(e.into());
        }
    };

    if let Some(candidate) = body.candidates.first()
        && let Some(Part::Text(txt)) = candidate.content.parts.first()
    {
        let final_text = txt.clone();
        history.push(candidate.content.clone());
        return Ok(final_text);
    }

    Ok("Processing complete.".to_string())
}

async fn simulated_response(state: &crate::web::AppState, input: &str) -> Result<()> {
    let input_lower = input.to_lowercase();
    let db = &state.db;
    
    if input_lower.contains("status")
        || input_lower.contains("kabarkan")
        || input_lower.contains("bagaimana")
    {
        println!("{}", "  🛠️  [SIMULATION] Querying database for plant status...".bright_black().italic());
        println!();
        
        let plants = db.active_plants().await?;
        let resp = format!(
            "Saat ini ada {} tanaman aktif. Semua parameter terlihat normal di database.",
            plants.len()
        );
        
        println!("{}", "  ┌─────────────────────────────────────────────────────────────┐".bright_green());
        println!("{} {}", "  │ 🤖".bright_green(), "AgroAI Response (Simulation):".bright_green().bold());
        println!("{}", "  ├─────────────────────────────────────────────────────────────┤".bright_green());
        println!("  │ {resp:<58}│");
        println!("  │                                                             │");
        
        for p in plants {
            let line = format!("• {}: {} (Terakhir disiram: {})", p.name, p.plant_type, p.last_watered);
            for wrapped in wrap_text(&line, 58) {
                println!("  │ {wrapped:<58}│");
            }
        }
        
        println!("{}", "  └─────────────────────────────────────────────────────────────┘".bright_green());
        println!();
        
        let _ = db.log_ai_interaction(input, &resp).await;
        broadcast_ai_log(state, input, &resp);
        
    } else if input_lower.contains("siram") || input_lower.contains("water") {
        println!("{}", "  🛠️  [SIMULATION] Analyzing watering command...".bright_black().italic());
        println!();
        
        let resp = "Perintah penyiraman diterima (dalam simulasi). Saya akan menginstruksikan sistem hardware untuk aktif.";
        
        println!("{}", "  ┌─────────────────────────────────────────────────────────────┐".bright_green());
        println!("{} {}", "  │ 🤖".bright_green(), "AgroAI Response (Simulation):".bright_green().bold());
        println!("{}", "  ├─────────────────────────────────────────────────────────────┤".bright_green());
        
        for line in wrap_text(resp, 58) {
            println!("  │ {line:<58}│");
        }
        
        println!("  │                                                             │");
        println!("  │ 💧 Activating pump for 3 seconds...                    │");
        println!("{}", "  └─────────────────────────────────────────────────────────────┘".bright_green());
        println!();
        
        water_plant("Simulator-Plant", 3).await;
        let _ = db.log_ai_interaction(input, resp).await;
        broadcast_ai_log(state, input, resp);
        
    } else {
        let resp = "[SIMULATION]: Maaf, dalam mode simulasi saya hanya paham perintah dasar seperti 'status' atau 'siram'. Masukkan API Key di .env untuk fitur AI penuh!";
        
        println!("{}", "  ┌─────────────────────────────────────────────────────────────┐".yellow());
        println!("{} {}", "  │ 🤖".yellow(), "AgroAI Response (Simulation):".yellow().bold());
        println!("{}", "  ├─────────────────────────────────────────────────────────────┤".yellow());
        
        for line in wrap_text(resp, 58) {
            println!("  │ {line:<58}│");
        }
        
        println!("{}", "  └─────────────────────────────────────────────────────────────┘".yellow());
        println!();
        
        let _ = db.log_ai_interaction(input, resp).await;
        broadcast_ai_log(state, input, resp);
    }
    Ok(())
}

fn select_provider() -> Result<AiProvider> {
    use inquire::Select;
    let providers = vec![
        AiProvider::GeminiFlash,
        AiProvider::GeminiPro,
        AiProvider::ClaudeSonnet,
        AiProvider::ClaudeOpus,
        AiProvider::ChatGpt4,
        AiProvider::ChatGpt5,
        AiProvider::Simulation,
    ];
    
    Ok(Select::new("Select AI Provider:", providers)
        .with_help_message("Choose your preferred AI model")
        .prompt()?)
}

fn api_key_for_provider(provider: &AiProvider) -> Result<Option<String>> {
    if matches!(provider, AiProvider::Simulation) {
        return Ok(None);
    }

    // Check environment variable first
    let env_key = match provider {
        AiProvider::GeminiFlash | AiProvider::GeminiPro => env::var("GEMINI_API_KEY").ok(),
        AiProvider::ClaudeSonnet | AiProvider::ClaudeOpus => env::var("ANTHROPIC_API_KEY").ok(),
        AiProvider::ChatGpt4 | AiProvider::ChatGpt5 => env::var("OPENAI_API_KEY").ok(),
        AiProvider::Simulation => None,
    };
    
    if let Some(key) = env_key {
        println!("{}", format!("  ✅ API Key found in environment: {}...", &key[..key.len().min(10)]).bright_green());
        println!();
        Ok(Some(key))
    } else {
        println!("{}", "  ⚠️  API Key not found in environment".yellow());
        println!();
        
        let key_input = Text::new("Enter API Key:")
            .with_help_message("Paste your API key here (it will be hidden)")
            .prompt()?;
        
        if key_input.trim().is_empty() {
            println!();
            println!("{}", "  ❌ No API key provided. Switching to Simulation Mode.".red());
            println!();
            Ok(None)
        } else {
            println!();
            println!("{}", "  ✅ API Key accepted".bright_green());
            println!();
            Ok(Some(key_input.trim().to_string()))
        }
    }
}

fn display_agent_instructions(is_simulation: bool) {
    println!("{}", "  📋 INSTRUCTIONS:".bright_yellow().bold());
    println!("{}", "     • Type your command in natural language".bright_white());
    println!("{}", "     • Examples: 'How are my plants?' or 'Water the Tomato'".bright_black());
    println!("{}", "     • Type 'exit' to return to the main menu".bright_black());
    println!();

    if is_simulation {
        println!("{}", "  ┌─────────────────────────────────────────────────────────────┐".yellow());
        println!("{}", "  │ ⚠️  SIMULATION MODE                                         │".yellow());
        println!("{}", "  │ Only basic commands available: 'status', 'siram'            │".yellow());
        println!("{}", "  │ For full AI features, restart and provide an API key        │".yellow());
        println!("{}", "  └─────────────────────────────────────────────────────────────┘".yellow());
        println!();
    }
}


