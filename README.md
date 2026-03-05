# 🌱 AgroCLI Edge - Smart Farming IoT Platform

> Self-hosted intelligent garden management system with real-time monitoring and automation. Now fully rewritten in **Rust** for maximum performance and reliability.

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7-blue.svg)](https://github.com/tokio-rs/axum)
[![Websocket](https://img.shields.io/badge/WebSocket-Real--Time-orange.svg)](https://github.com/tokio-rs/axum)
[![Ratatui](https://img.shields.io/badge/Ratatui-0.26-green.svg)](https://ratatui.rs/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## ✨ Core Features

### 🌐 Premium Web Dashboard (v1.3.0)
- **Full Mobile Responsiveness** - Premium glassmorphic design that adapt to phone, tablet, and desktop.
- **AI Agent Interaction Feed** - Real-time stream of what your garden brain is thinking and doing.
- **Interactive Charts** - Moisture, Temp, and Humidity trends with high-performance rendering.
- **Sensor History & Export** - View historical data up to 7 days and export directly to CSV.
- **Manual Control** - Trigger watering and update plant thresholds remotely.
- **Secure Access** - Basic authentication protected endpoints.

### 🤖 AI Agent Mode
- **Multimodal AI Support** - Powered by Google Gemini (Flash/Pro) for intelligent decision making.
- **Natural Language Control** - Control your garden with natural language (Indonesian/English).
- **Tool-Calling Capabilities** - AI can directly query the database and trigger hardware via `water_plant_action`.
- **Simulation Mode** - Test AI logic safely without an API key.

### 🔌 IoT & Performance Layer
- **High-Performance Rust Core** - Optimized async engine using `tokio` and `axum`.
- **Direct Async Broadcasting** - Zero-latency internal communication between modules via specialized channels.
- **Daemon Mode** - 24/7 automated monitoring, weather checking, and failsafe protection.
- **SQLite Persistence** - Reliable local data storage with `sqlx`.
- **Ratatui TUI** - Polished terminal interface for quick management and monitoring.

## 🚀 Quick Start

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (v1.75+)
- [SQLite](https://www.sqlite.org/)

### Installation
```bash
# Clone repository
git clone https://github.com/yourusername/AgroCLI.git
cd AgroCLI

# Build the project
cargo build --release

# Initialize garden (first time only)
cargo run -- init
```

### Usage

**Terminal 1 - Web Dashboard & Logic Engine:**
```bash
# Starts the server and daemon logic
cargo run -- serve
```

**Terminal 2 - Interactive Interface (TUI):**
```bash
# Polished terminal dashboard
cargo run -- interactive
```

**Terminal 3 - AI Agent (Optional):**
```bash
# Direct natural language chat
cargo run -- ai-agent
```

## 📚 Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - 5-minute setup guide.
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Internal data flow and async patterns.
- **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)** - REST & WebSocket API details.
- **[AI_PROVIDER_GUIDE.md](AI_PROVIDER_GUIDE.md)** - Configuring Gemini and other AI models.

## 🔧 Configuration

All secrets and server settings are managed via `.env`:
```bash
PORT=8001
GEMINI_API_KEY=your_key_here
ADMIN_USERNAME=admin
ADMIN_PASSWORD=your_password
```

## 👨‍💻 Author

**AgroCLI Team**
Made with 💚 and **Rust** for smart farming enthusiasts.

---
**Happy Farming! 🌱**
