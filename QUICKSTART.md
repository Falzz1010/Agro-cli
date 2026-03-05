# ⚡ Quick Start Guide - AgroCLI Edge

## 🎯 Goal
In 5 minutes, you'll have a high-performance smart farming system running:
- **Rust Core**: Ultra-fast sensor processing.
- **Axum Web**: Premium real-time dashboard.
- **AI Agent**: Intelligent garden brain.

## 📋 Prerequisites
- [Rust & Cargo](https://rustup.rs/) (v1.75+)
- Git installed.

## 🚀 5-Minute Setup

### 1️⃣ Clone & Build
```bash
git clone https://github.com/yourusername/AgroCLI.git
cd AgroCLI
cargo build --release
```

### 2️⃣ Initialize Garden
```bash
cargo run -- init
```
*This creates your local `data/garden.db` and prepares the schema.*

### 3️⃣ Add Your First Plant
```bash
cargo run -- add tomato "My-Tomato"
```

### 4️⃣ Launch the Engine (Terminal 1)
```bash
cargo run -- serve
```
*This starts the **Web Server**, **WebSocket Hub**, and **Automation Daemon** all at once.*

**✅ Checkpoint:** Open `http://localhost:8001` in your browser.

### 5️⃣ Launch the Interactive TUI (Terminal 2)
```bash
cargo run -- interactive
```
*Beautiful terminal interface for local monitoring.*

---

## 🧪 Quick Test

1. **Dashboard Check**: Go to the web dashboard. You should see "My-Tomato" appearing.
2. **AI Inquiry**: Type `ai-agent` command or use the AI feed on the dashboard to ask: "Bagaimana kondisi tomat saya?"
3. **Manual Water**: Click the "💧 Trigger Pump" button on the dashboard. Watch the logs in Terminal 1 for activation.

## 🔧 Pro Configuration
- **Care Rules**: Edit `plants.yaml` to change moisture thresholds.
- **AI Brain**: Add your `GEMINI_API_KEY` to `.env` for full intelligence.
- **Access**: Access from your phone via `http://[your-ip]:8001`.

---
**Happy Farming! 🌱**
