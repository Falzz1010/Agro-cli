# 🌱 AgroCLI - Smart Farming IoT Platform

> Self-hosted intelligent garden management system with real-time monitoring and automation

[![Python](https://img.shields.io/badge/Python-3.8+-blue.svg)](https://www.python.org/)
[![FastAPI](https://img.shields.io/badge/FastAPI-0.109-green.svg)](https://fastapi.tiangolo.com/)
[![WebSocket](https://img.shields.io/badge/WebSocket-Real--Time-orange.svg)](https://websockets.readthedocs.io/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## ✨ Features

### 🤖 AI Agent Mode (NEW!)
- **Natural Language Control** - Control your garden with Indonesian commands
- **Smart Command Parsing** - Understands context and intent
- **Automated Actions** - Water plants, check status, get recommendations
- **Interactive Chat** - Conversational interface for easy control
- **No API Key Required** - Rule-based AI, works offline

### 🌐 Real-Time Web Dashboard
- **Live Sensor Monitoring** - Moisture, temperature, humidity updates every 5 seconds
- **Auto-Updating Charts** - Chart.js with rolling window visualization
- **WebSocket Integration** - Instant updates across all connected devices
- **Mobile Responsive** - Access from phone, tablet, or desktop
- **Multi-Client Sync** - All browsers stay synchronized
- **Manual Pump Control** - Trigger watering from web interface

### 🤖 Intelligent Automation
- **24/7 Daemon Mode** - Continuous monitoring and auto-watering
- **Weather-Aware Logic** - Skip watering when raining
- **Sensor-Based Decisions** - Real-time moisture threshold triggers
- **Failsafe Protection** - Pump lock after 5 consecutive triggers
- **Event Broadcasting** - Real-time alerts and notifications
- **Optimized Sensor Reading** - Smart caching to reduce hardware load

### 💻 CLI Interface
- **Interactive Menu** - Beautiful terminal UI with questionary
- **Live Task Monitor** - Real-time auto-refreshing task list (2s refresh)
- **Live Stats Monitor** - Real-time garden statistics (1s refresh)
- **Live Sensor Monitor** - Real-time sensor readings via WebSocket
- **Task Management** - Daily watering and fertilizing schedules
- **Plant Lifecycle** - Add, monitor, harvest plants
- **Statistics Export** - CSV export for data analysis
- **Rich Formatting** - Colorful terminal output with emojis

### 🔌 IoT Ready
- **Hardware Abstraction** - Easy integration with real sensors
- **Mock Sensors** - Test without hardware (random data generation)
- **Raspberry Pi Compatible** - Ready for GPIO/I2C
- **ESP32 Support** - Can integrate with microcontrollers
- **HTTP API** - RESTful endpoints for external integrations

## 🚀 Quick Start

### Installation
```bash
# Clone repository
git clone https://github.com/yourusername/agrocli.git
cd agrocli

# Install dependencies
pip install -r requirements.txt

# Initialize database
python main.py init
```

### Add Plants
```bash
python main.py add tomato "My-Tomato"
python main.py add chili "My-Chili"
```

### Start Real-Time System

**Terminal 1 - Web Dashboard:**
```bash
cargo run -- serve
```

**Terminal 2 - Daemon Mode:**
```bash
cargo run -- daemon
```

**Terminal 3 - CLI Monitor (Optional):**
```bash
cargo run -- interactive
```

**Browser:**
```
http://localhost:8000
```

🎉 **Done!** You now have a fully functional real-time smart farming system.

### Use AI Agent Mode

Control your garden with natural language (Indonesian):

```bash
python main.py
# Select: 🤖 AI Agent Mode
```

**Example Commands:**
```
🤖 Perintah: Cek status kebun
🤖 Perintah: Siram tanaman Tomat-Saya
🤖 Perintah: Siram semua tanaman yang kering
🤖 Perintah: Berikan rekomendasi
🤖 Perintah: Tambah tanaman tomat bernama Test-Plant
```

Type `help` for more commands, `exit` to quit.

## 📚 Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - 5-minute setup guide
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design & architecture
- **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)** - Backend API details

## 🎯 Use Cases

### Home Gardening
- Monitor indoor plants
- Automate watering schedule
- Track plant health over time

### Small Farm
- Manage multiple plant types
- Weather-aware irrigation
- Export data for analysis

### Education
- Learn IoT development
- Understand sensor integration
- Practice full-stack development

### Prototyping
- Test smart farming concepts
- Develop custom plant profiles
- Integrate with other systems

## 🛠️ Technology Stack

- **Backend:** Python 3.8+, FastAPI, SQLite
- **Real-Time:** WebSockets, Asyncio
- **Frontend:** HTML5, JavaScript, Chart.js
- **CLI:** Rich, Questionary
- **Hardware:** GPIO (Raspberry Pi), I2C sensors

## 📊 System Architecture

```
CLI Interface ──┐
                ├──► Core Engine ──► Database (SQLite)
Web Dashboard ──┘         │
                          ├──► Weather API
                          ├──► Hardware Layer
                          └──► WebSocket Hub
                                    │
                                    └──► Real-Time Updates
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed diagrams.

## 🔧 Configuration

### Plant Rules (`plants.yaml`)
```yaml
tomato:
  water_interval_days: 1
  min_moisture_level: 40.0
  water_ml: 250
  fertilizer_interval_days: 7
  sun_hours: 6
```

### Weather API (`data/config.json`)
```json
{
  "city": "Surabaya",
  "api_key": "your_openweathermap_key"
}
```

## 🧪 Testing

```bash
# Run basic tests
python main.py stats

# Test real-time system
python main.py daemon  # Terminal 1
python main.py serve   # Terminal 2

# Export data
python main.py stats --export data.csv
```

## 🌟 Screenshots

### Web Dashboard
- Real-time sensor monitoring
- Live charts with auto-scroll
- Mobile-responsive design

### CLI Interface
- Interactive menu system
- Colorful terminal output
- Task management

### Daemon Mode
- 24/7 monitoring logs
- Auto-watering events
- System alerts

## 🤝 Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create feature branch
3. Commit changes
4. Push to branch
5. Open pull request

## 📝 License

MIT License - see [LICENSE](LICENSE) file

## 👨‍💻 Author

**Naufal Rizky**
- Made with 💚 for smart farming enthusiasts

## 🙏 Acknowledgments

- OpenWeatherMap for weather API
- FastAPI for amazing web framework
- Chart.js for beautiful charts
- Rich for terminal formatting

## 📞 Support

- GitHub Issues for bugs
- Discussions for questions
- Pull Requests for contributions

---

**Happy Farming! 🌱**
