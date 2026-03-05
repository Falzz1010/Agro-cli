use std::net::SocketAddr;
use std::sync::Arc;
use std::fmt::Write; // Added for efficient string building
use tokio_util::sync::CancellationToken;

use axum::{
    Json, Router,
    extract::{Path, Query, ws::{Message, WebSocket, WebSocketUpgrade}},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// Data structure for broadcasting sensor readings.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SensorData {
    pub plant_name: String,
    pub moisture: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub timestamp: String,
    pub min_moisture: Option<f32>,
    pub water_ml: Option<i32>,
}

/// Data structure for AI interactions.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiLog {
    pub timestamp: String,
    pub query: String,
    pub response: String,
}

/// A single historical sensor data point.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SensorHistoryPoint {
    pub timestamp: String,
    pub moisture: f32,
    pub temperature: f32,
    pub humidity: f32,
}

/// Unified message type for WebSocket updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum DashboardMessage {
    SensorUpdate(SensorData),
    AiLog(AiLog),
}

/// Shared state for the web server.
#[derive(Clone)]
pub struct AppState {
    /// Broadcast channel sender for live sensor data and AI logs.
    pub tx: broadcast::Sender<DashboardMessage>,
    /// Database handle for persistence.
    pub db: std::sync::Arc<crate::db::Database>,
}

/// Command structure for remote actions.
#[derive(Deserialize)]
pub struct SettingsCommand {
    pub plant_name: String,
    pub min_moisture: f32,
    pub water_ml: i32,
}

/// Command structure for remote watering.
#[derive(Deserialize)]
pub struct WaterCommand {
    pub plant_name: String,
}

/// Command structure for deleting a plant.
#[derive(Deserialize)]
pub struct DeleteCommand {
    pub plant_name: String,
}

/// Unified error type for the web module.
pub enum AppError {
    NotFound(String),
    Database(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_msg) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::Database(msg) | Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": true,
            "message": error_msg,
        }));

        (status, body).into_response()
    }
}

// Convert any error to AppError::Internal
impl<E> From<E> for AppError
where
    E: std::error::Error,
{
    fn from(err: E) -> Self {
        Self::Internal(err.to_string())
    }
}

type WebResult<T> = Result<T, AppError>;

/// Endpoint for the Daemon to POST live sensor updates.
async fn broadcast_sensor(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<SensorData>,
) -> impl IntoResponse {
    let _ = state.tx.send(DashboardMessage::SensorUpdate(payload.clone()));
    Json(serde_json::json!({ "status": "broadcasted", "received": payload.plant_name }))
}

/// Endpoint for the AI Agent to POST live interaction logs.
async fn broadcast_ai(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<AiLog>,
) -> impl IntoResponse {
    let _ = state.tx.send(DashboardMessage::AiLog(payload));
    Json(serde_json::json!({ "status": "broadcasted", "type": "ai_log" }))
}

/// Endpoint for the Dashboard to trigger manual watering.
async fn manual_water(Json(payload): Json<WaterCommand>) -> WebResult<impl IntoResponse> {
    tracing::info!(plant = %payload.plant_name, "Remote watering triggered via web");
    crate::hardware::water_plant(&payload.plant_name, 3).await;
    Ok(Json(serde_json::json!({ "status": "executed", "plant": payload.plant_name })))
}

/// Returns historical sensor data for a plant.
async fn get_history(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(plant_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> WebResult<impl IntoResponse> {
    let hours: i32 = params
        .get("hours")
        .and_then(|h| h.parse().ok())
        .unwrap_or(24);

    let data = state.db.sensor_history(&plant_name, hours).await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
    Ok(Json(serde_json::json!({ "plant_name": plant_name, "hours": hours, "data": data })))
}

/// Exports historical sensor data as a CSV file.
async fn export_history(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(plant_name): Path<String>,
) -> impl IntoResponse {
    // Fetch a large window of history for export (e.g., last 30 days = 720 hours)
    let db = Arc::clone(&state.db);
    let p_name = plant_name.clone();
    
    match db.sensor_history(&p_name, 720).await {
        Ok(data) => {
            // Move string building to a blocking task to avoid stalling the async executor
            let csv_res = tokio::task::spawn_blocking(move || {
                // Pre-allocate capacity: ~60 bytes per row * data.len() + header
                let mut csv_string = String::with_capacity(data.len() * 60 + 60);
                csv_string.push_str("Timestamp,Moisture (%),Temperature (C),Humidity (%)\n");
                
                for point in data {
                    let _ = writeln!(
                        csv_string,
                        "{timestamp},{moisture:.1},{temperature:.1},{humidity:.1}",
                        timestamp = point.timestamp,
                        moisture = point.moisture,
                        temperature = point.temperature,
                        humidity = point.humidity
                    );
                }
                csv_string
            }).await;

            match csv_res {
                Ok(csv_data) => {
                    let headers = [(
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{plant_name}_sensor_logs.csv\""),
                    )];
                    (StatusCode::OK, headers, csv_data).into_response()
                }
                Err(e) => {
                    tracing::error!(error = %e, "Background CSV generation failed");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate CSV").into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!(plant = %plant_name, error = %e, "Error fetching export data");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error fetching data: {e}"),
            ).into_response()
        }
    }
}

/// Permanently deletes a plant and its sensor data.
async fn delete_plant(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<DeleteCommand>,
) -> WebResult<impl IntoResponse> {
    tracing::info!(plant = %payload.plant_name, "Delete requested via web");
    if state.db.delete_plant(&payload.plant_name).await? {
        Ok(Json(serde_json::json!({ "status": "deleted", "plant": payload.plant_name })))
    } else {
        Err(AppError::NotFound(format!("Plant '{}' not found", payload.plant_name)))
    }
}

/// Endpoint for the Dashboard to update plant-specific health thresholds.
async fn update_settings(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<SettingsCommand>,
) -> WebResult<impl IntoResponse> {
    tracing::info!(plant = %payload.plant_name, "Updating settings via web");
    if state
        .db
        .update_plant_settings(&payload.plant_name, payload.min_moisture, payload.water_ml)
        .await?
    {
        Ok(Json(serde_json::json!({ "status": "updated", "plant": payload.plant_name })))
    } else {
        Err(AppError::NotFound(format!("Plant '{}' not found", payload.plant_name)))
    }
}

/// Upgrades a connection to a WebSocket for real-time streaming.
async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handles a single WebSocket stream, sending sensor data as it arrives.
#[allow(clippy::collapsible_if)]
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // 1. Send initial sensor state
    if let Ok(latest_data) = state.db.latest_sensor_data().await {
        for data in latest_data {
            let msg_obj = DashboardMessage::SensorUpdate(data);
            if let Ok(msg) = serde_json::to_string(&msg_obj) {
                if socket.send(Message::Text(msg)).await.is_err() {
                    return;
                }
            }
        }
    }

    // 2. Send initial AI logs
    if let Ok(latest_logs) = state.db.recent_ai_logs(10).await {
        for log in latest_logs {
            let msg_obj = DashboardMessage::AiLog(log);
            if let Ok(msg) = serde_json::to_string(&msg_obj) {
                if socket.send(Message::Text(msg)).await.is_err() {
                    return;
                }
            }
        }
    }

    // 3. Subscribe to real-time updates (both sensors and AI)
    let mut rx = state.tx.subscribe();
    while let Ok(data) = rx.recv().await {
        let Ok(msg) = serde_json::to_string(&data) else {
            continue;
        };
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

/// Starts the Axum web server and Real-Time Hub.
///
/// # Errors
/// Returns an error if the address is invalid or the server fails to bind/start.
pub async fn serve(
    state: AppState,
    token: CancellationToken,
) -> anyhow::Result<()> {

    // Standard Axum authentication middleware

    let app = Router::new()
        // Serve static files from the "static" directory
        .fallback_service(ServeDir::new("static"))
        .route("/ws", get(ws_handler))
        .route("/api/broadcast/sensor", post(broadcast_sensor))
        .route("/api/broadcast/ai", post(broadcast_ai))
        .route("/api/command/water", post(manual_water))
        .route("/api/command/delete", post(delete_plant))
        .route("/api/command/settings", post(update_settings))
        .route("/api/history/:plant_name", get(get_history))
        .route("/api/export/:plant_name", get(export_history))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr_str = format!("{host}:{port}");

    let addr: SocketAddr = addr_str
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid HOST or PORT in .env: {e}"))?;
    
    // Display user-friendly message
    let display_host = if host == "0.0.0.0" { "127.0.0.1" } else { host.as_str() };
    println!("🌐 [WEB] Real-Time Dashboard running");
    println!("      Server listening on: {addr}");
    println!("      Access in browser:   http://{display_host}:{port}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {addr}: {e}"))?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            token.cancelled().await;
            println!("🌐 [WEB] Shutdown signal received, closing server...");
        })
        .await
        .map_err(|e| anyhow::anyhow!("Web server error: {e}"))?;

    Ok(())
}
