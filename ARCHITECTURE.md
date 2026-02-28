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
├── main.py                    # Entry point & CLI commands
│   ├── init()                 # Database initialization
│   ├── add()                  # Add new plant
│   ├── today()                # Show today's tasks
│   ├── harvest()              # Archive plant
│   ├── stats()                # Garden statistics
│   ├── daemon_mode()          # 24/7 automation loop
│   └── interactive_mode()     # Menu-driven interface
│
├── core/                      # Business logic
│   ├── database.py            # SQLite operations
│   │   ├── init_db()
│   │   ├── add_plant()
│   │   ├── get_all_active_plants()
│   │   ├── update_care()
│   │   ├── log_sensor_data()
│   │   └── get_recent_sensor_logs()
│   │
│   ├── engine.py              # Task calculation engine
│   │   ├── load_rules()
│   │   └── calculate_today_tasks()
│   │
│   ├── weather.py             # Weather API integration
│   │   └── get_weather()
│   │
│   └── realtime.py            # WebSocket manager (NEW)
│       ├── ConnectionManager
│       ├── send_sensor_update()
│       ├── send_pump_event()
│       └── send_system_alert()
│
├── hardware/                  # IoT abstraction layer
│   ├── sensors.py             # Sensor readings
│   │   ├── read_soil_moisture()
│   │   ├── read_temperature()
│   │   └── read_humidity()
│   │
│   └── pump.py                # Pump control
│       └── water_plant()
│
├── web/                       # Web interface
│   └── server.py              # FastAPI application
│       ├── websocket_endpoint()
│       ├── read_root()        # Dashboard HTML
│       ├── api_get_telemetry()
│       ├── api_water_plant()
│       └── serve()
│
├── data/                      # Data storage
│   ├── garden.db              # SQLite database
│   └── config.json            # Weather API config
│
├── plants.yaml                # Plant care rules
├── requirements.txt           # Python dependencies
└── REALTIME_SETUP.md         # Setup documentation
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
- **Sensor Read Interval:** 5 seconds
- **Database Write Interval:** 60 seconds
- **WebSocket Broadcast:** Real-time (< 100ms)
- **Chart Data Points:** 30 (rolling window)
- **Max Concurrent Clients:** Unlimited (FastAPI async)

### Resource Usage (Estimated)
- **CPU:** < 5% (idle), < 15% (active monitoring)
- **RAM:** ~50MB (Python + FastAPI)
- **Database Size:** ~1MB per month (1 plant, 5s interval)
- **Network:** ~1KB per sensor update

## 🚀 Deployment Options

### Option 1: Raspberry Pi (Recommended)
```bash
# Install on Raspberry Pi OS
sudo apt update
sudo apt install python3-pip
pip3 install -r requirements.txt

# Run as systemd service
sudo systemctl enable agrocli-daemon
sudo systemctl enable agrocli-web
```

### Option 2: Docker Container
```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY . .
RUN pip install -r requirements.txt
CMD ["python", "main.py", "serve"]
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
