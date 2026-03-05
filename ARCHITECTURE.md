# 🏗️ AgroCLI System Architecture

## 📊 System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        AgroCLI System                            │
│                   Smart Farming IoT Platform                     │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────┐         ┌──────────────────┐
│   CLI Interface  │         │  Web Dashboard   │
│   (Terminal)     │         │   (Browser)      │
│                  │         │                  │
│  • Add Plants    │         │  • Live Monitor  │
│  • View Tasks    │         │  • Control Pump  │
│  • Statistics    │         │  • Real-time     │
│  • Interactive   │         │    Charts        │
└────────┬─────────┘         └────────┬─────────┘
         │                            │
         │                            │ HTTP/WebSocket
         ▼                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Core Application Layer                      │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Engine     │  │   Database   │  │   Weather    │         │
│  │              │  │              │  │              │         │
│  │ • Task Calc  │  │ • SQLite     │  │ • OpenWeather│         │
│  │ • Rules      │  │ • CRUD Ops   │  │   API        │         │
│  │ • Logic      │  │ • Logging    │  │ • Conditions │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌──────────────────────────────────────────────────────┐      │
│  │              Real-Time Manager                        │      │
│  │                                                        │      │
│  │  • WebSocket Hub                                      │      │
│  │  • Event Broadcasting                                 │      │
│  │  • Connection Management                              │      │
│  └──────────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────┘
         │                            │
         │                            │
         ▼                            ▼
┌──────────────────┐         ┌──────────────────┐
│  Hardware Layer  │         │   Daemon Mode    │
│                  │         │                  │
│  • Sensors       │◄────────┤  • Auto Monitor  │
│    - Moisture    │         │  • Auto Water    │
│    - Temp/Humid  │         │  • Failsafe      │
│  • Pump Control  │         │  • 24/7 Loop     │
└──────────────────┘         └──────────────────┘
```

## 🔄 Real-Time Data Flow

### Scenario 1: Daemon Mode Monitoring

```
1. Daemon Loop (Every 5 seconds)
   │
   ├─► Read Sensors (Mock/Real Hardware)
   │   └─► Moisture: 45.2%
   │   └─► Temperature: 28.5°C
   │   └─► Humidity: 65.3%
   │
   ├─► Broadcast via WebSocket
   │   └─► All connected browsers receive update
   │
   ├─► Check Watering Rules
   │   └─► If moisture < threshold
   │       ├─► Broadcast "Pump ON" event
   │       ├─► Activate pump (3 seconds)
   │       ├─► Update database
   │       └─► Broadcast "Pump OFF" event
   │
   └─► Log to Database (Every 60 seconds)
       └─► sensor_logs table
```

### Scenario 2: Manual Pump Trigger from Web

```
1. User clicks "Trigger Pump" button
   │
   ├─► POST /api/water/{plant_name}
   │
   ├─► Server broadcasts "Pump ON" event
   │   └─► All browsers show "Pumping..." status
   │
   ├─► Background task activates pump
   │   └─► 3 seconds duration
   │
   ├─► Update database (last_watered)
   │
   └─► Server broadcasts "Pump OFF" event
       └─► All browsers reset button state
```

## 📁 File Structure

```
agrocli/
│
├── main.rs                    # Entry point & CLI handles
│   ├── main()                 # Async entry point
│   ├── run_daemon()           # Automation loop
│   └── run_tui()              # Interactive TUI
│
├── src/                       # Rust source code
│   ├── ai/                    # Gemini/AI Agent logic
│   ├── db/                    # SQLx SQLite operations
│   ├── hardware/              # Sensor & Pump abstraction
│   ├── web/                   # Axum web server & WS
│   └── tui/                   # Ratatui interface
│
├── data/                      # Data storage
│   └── garden.db              # SQLite database
│
├── Cargo.toml                 # Rust dependencies
└── plants.yaml                # Plant care rules
```

## 🗄️ Database Schema

### Table: plants
```sql
CREATE TABLE plants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    plant_type TEXT NOT NULL,
    planted_date TEXT NOT NULL,
    last_watered TEXT NOT NULL,
    last_fertilized TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active'
);
```

### Table: sensor_logs
```sql
CREATE TABLE sensor_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    ambient_temp REAL NOT NULL,
    ambient_humidity REAL NOT NULL,
    plant_name TEXT NOT NULL,
    soil_moisture REAL NOT NULL
);
```

## 🔌 WebSocket Protocol

### Client → Server
```javascript
// Keep-alive ping (optional)
ws.send(JSON.stringify({ type: "ping" }));
```

### Server → Client

**Message Format:**
```typescript
interface WebSocketMessage {
  type: "sensor_update" | "pump_event" | "system_alert";
  timestamp: string;
  // ... type-specific fields
}
```

**Sensor Update:**
```json
{
  "type": "sensor_update",
  "timestamp": "14:23:45",
  "plant_name": "Tomato-1",
  "moisture": 45.2,
  "temperature": 28.5,
  "humidity": 65.3
}
```

**Pump Event:**
```json
{
  "type": "pump_event",
  "timestamp": "14:23:50",
  "plant_name": "Tomato-1",
  "status": "on",
  "duration": 3
}
```

**System Alert:**
```json
{
  "type": "system_alert",
  "timestamp": "14:24:00",
  "message": "Emergency: Pump locked",
  "level": "error"
}
```

## 🎯 Decision Logic

### Watering Decision Tree

```
Start
  │
  ├─► Has real-time moisture sensor?
  │   ├─► YES: Use sensor reading
  │   │   └─► moisture < min_moisture_level?
  │   │       ├─► YES: needs_water = true
  │   │       └─► NO: needs_water = false
  │   │
  │   └─► NO: Use date-based rule
  │       └─► days_since_watered >= water_interval_days?
  │           ├─► YES: needs_water = true
  │           └─► NO: needs_water = false
  │
  ├─► Is it raining?
  │   ├─► YES: skip_watering = true
  │   └─► NO: proceed
  │
  ├─► Pump triggered 5+ times consecutively?
  │   ├─► YES: LOCK PUMP (failsafe)
  │   └─► NO: proceed
  │
  └─► Execute watering
      ├─► Activate pump
      ├─► Update database
      └─► Broadcast event
```

## 🔐 Security Considerations

### Current Implementation (Development)
- ❌ No authentication
- ❌ No HTTPS/WSS
- ❌ No rate limiting
- ❌ No input validation

### Production Recommendations
- ✅ Add JWT authentication
- ✅ Enable HTTPS with SSL certificate
- ✅ Implement rate limiting
- ✅ Validate all user inputs
- ✅ Add CORS configuration
- ✅ Use environment variables for secrets

## 📈 Performance Metrics

### Current Specifications
- **Automation Cycle (Daemon):** 5 seconds
- **Database Write (Log) Interval:** 5 seconds (tied to sensor read)
- **TUI Refresh Rate:** 1s (Sensors), 2s (Tasks/Stats)
- **WebSocket Broadcast:** Real-time (< 100ms)
- **Max Concurrent Clients:** Unlimited (Tokio async)

### Resource Usage (Estimated)
- **CPU:** < 1% (idle), < 5% (active monitoring)
- **RAM:** ~15-20MB (Rust + Axum)
- **Database Size:** ~1MB per month (1 plant, 5s interval)
- **Network:** ~1KB per sensor update

## 🚀 Deployment Options

### Option 1: Raspberry Pi (Recommended)
```bash
# Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and Build
git clone https://github.com/yourusername/AgroCLI.git
cd AgroCLI
cargo build --release

# Run
./target/release/AgroCLI daemon
./target/release/AgroCLI serve
```

### Option 2: Docker Container
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/AgroCLI .
COPY --from=builder /app/plants.yaml .
CMD ["./AgroCLI", "daemon"]
```

### Option 3: Cloud VPS
- Deploy to DigitalOcean/AWS/Azure
- Use reverse proxy (Nginx)
- Enable HTTPS with Let's Encrypt
- Set up monitoring (Prometheus/Grafana)

## 🔄 Future Enhancements

### Phase 1: Hardware Integration
- [ ] Real DHT22 temperature/humidity sensor
- [ ] Real capacitive soil moisture sensor
- [ ] Real relay module for pump control
- [ ] Multiple pump support

### Phase 2: Advanced Features
- [ ] Machine learning for optimal watering
- [ ] Historical data analytics
- [ ] Weather forecast integration
- [ ] Mobile app (React Native)
- [ ] Push notifications

### Phase 3: Scalability
- [ ] Multi-garden support
- [ ] User authentication & roles
- [ ] Cloud sync & backup
- [ ] API for third-party integrations
- [ ] Marketplace for plant profiles

## 📞 Support & Contribution

Untuk pertanyaan atau kontribusi:
- GitHub Issues
- Pull Requests welcome
- Documentation improvements

---

**Built with ❤️ for smart farming enthusiasts**
