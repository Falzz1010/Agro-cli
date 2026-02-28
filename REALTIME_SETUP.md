# 🌐 AgroCLI Real-Time System Setup

## ✨ Fitur Real-Time yang Sudah Diimplementasi

### 1. WebSocket Integration
- ✅ Live sensor data streaming ke web dashboard
- ✅ Real-time chart updates (moisture, temperature, humidity)
- ✅ Pump status notifications
- ✅ System alerts & warnings
- ✅ Auto-reconnect jika koneksi terputus

### 2. Daemon Mode Enhancement
- ✅ Broadcast sensor readings via WebSocket
- ✅ Push pump events ke semua connected clients
- ✅ System alerts untuk emergency situations
- ✅ Fallback ke sync mode jika WebSocket tidak tersedia

### 3. Web Dashboard Features
- ✅ Live connection status indicator
- ✅ Real-time sensor readings per plant
- ✅ Auto-updating Chart.js graphs
- ✅ Toast notifications untuk events
- ✅ Responsive design untuk mobile

## 📦 Installation

### 1. Install Dependencies
```bash
pip install -r requirements.txt
```

Dependencies baru yang ditambahkan:
- `fastapi==0.109.0` - Web framework
- `uvicorn[standard]==0.27.0` - ASGI server
- `websockets==12.0` - WebSocket support

### 2. Initialize Database
```bash
python main.py init
```

### 3. Add Test Plants
```bash
python main.py add tomato "Tomato-1"
python main.py add chili "Chili-1"
```

## 🚀 Running Real-Time System

### Mode 1: Web Dashboard Only
```bash
python main.py serve
```
- Buka browser: `http://localhost:8000`
- Dashboard akan menampilkan historical data
- Bisa trigger pump manual dari UI

### Mode 2: Daemon + Web Dashboard (RECOMMENDED)

**Terminal 1 - Daemon Mode:**
```bash
python main.py daemon
```
Output:
```
🤖 AgroCLI Daemon Mode Activated
✓ WebSocket broadcasting enabled

🕰️  Cycle Check: 14:23:45
Sensor Tomato-1 | Moisture: 45.2% | Temp: 28.5°C | Hum: 65.3%
```

**Terminal 2 - Web Server:**
```bash
python main.py serve
```
Output:
```
🌐 Starting AgroCLI Web Server...
Open http://localhost:8000 in your browser
```

Sekarang:
- Daemon akan broadcast sensor data setiap 5 detik
- Web dashboard akan update real-time
- Chart akan auto-scroll dengan data baru
- Pump events akan muncul sebagai notifications

## 🔌 How It Works

### Architecture Flow:
```
┌─────────────────┐
│  Daemon Mode    │
│  (Terminal 1)   │
│                 │
│  Read Sensors   │
│  Check Rules    │
│  Control Pump   │
└────────┬────────┘
         │
         │ WebSocket Broadcast
         ▼
┌─────────────────┐
│  FastAPI Server │
│  (Terminal 2)   │
│                 │
│  WebSocket Hub  │
└────────┬────────┘
         │
         │ WebSocket Push
         ▼
┌─────────────────┐
│  Web Dashboard  │
│  (Browser)      │
│                 │
│  Live Updates   │
│  Chart.js       │
└─────────────────┘
```

### WebSocket Message Types:

**1. Sensor Update**
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

**2. Pump Event**
```json
{
  "type": "pump_event",
  "timestamp": "14:23:50",
  "plant_name": "Tomato-1",
  "status": "on",
  "duration": 3
}
```

**3. System Alert**
```json
{
  "type": "system_alert",
  "timestamp": "14:24:00",
  "message": "🚨 Tomato-1: Minimum moisture threshold breached!",
  "level": "warning"
}
```

## 🧪 Testing Real-Time Features

### Test 1: Sensor Data Streaming
1. Start daemon mode
2. Open web dashboard
3. Watch sensor readings update every 5 seconds
4. Chart should auto-scroll with new data

### Test 2: Manual Pump Trigger
1. Open web dashboard
2. Click "💧 Trigger Pump" button
3. Button should change to "💧 Pumping..."
4. Toast notification should appear
5. After 3 seconds, button returns to normal

### Test 3: Auto-Reconnect
1. Stop daemon mode (Ctrl+C)
2. Dashboard should show "🔴 Disconnected"
3. Restart daemon mode
4. Dashboard should auto-reconnect "🟢 Real-Time Connected"

### Test 4: Multiple Clients
1. Open dashboard in 2 different browsers
2. Trigger pump from browser 1
3. Browser 2 should also see the pump event
4. Both should receive same sensor updates

## 🔧 Configuration

### Change WebSocket Update Interval
Edit `main.py` daemon_mode():
```python
await asyncio.sleep(5)  # Change to 10 for slower updates
```

### Change Chart Data Points
Edit `web/server.py`:
```javascript
const MAX_POINTS = 30;  // Change to 50 for more history
```

### Change Database Write Interval
Edit `main.py` daemon_mode():
```python
DB_WRITE_INTERVAL_SECONDS = 60  # Change to 30 for more frequent logging
```

## 📱 Mobile Access

Dashboard dapat diakses dari device lain di network yang sama:

1. Cek IP address komputer:
   ```bash
   # Windows
   ipconfig
   
   # Linux/Mac
   ifconfig
   ```

2. Buka dari smartphone/tablet:
   ```
   http://192.168.1.XXX:8000
   ```

## 🐛 Troubleshooting

### WebSocket tidak connect
- Pastikan web server running di port 8000
- Check firewall tidak block port 8000
- Coba akses via `http://localhost:8000` dulu

### Sensor data tidak update
- Pastikan daemon mode running
- Check terminal daemon untuk error messages
- Pastikan ada active plants di database

### Chart tidak muncul
- Check browser console untuk JavaScript errors
- Pastikan Chart.js CDN loaded
- Clear browser cache dan reload

### Pump tidak trigger
- Check plant exists di database
- Pastikan moisture threshold configured di plants.yaml
- Check daemon terminal untuk failsafe locks

## 🎯 Next Steps

Untuk production deployment:

1. **Replace Mock Sensors** - Edit `hardware/sensors.py`:
   ```python
   import board
   import adafruit_dht
   
   def read_temperature():
       sensor = adafruit_dht.DHT22(board.D4)
       return sensor.temperature
   ```

2. **Replace Mock Pump** - Edit `hardware/pump.py`:
   ```python
   import RPi.GPIO as GPIO
   
   def water_plant(plant_name, duration_seconds):
       GPIO.output(PUMP_PIN, GPIO.HIGH)
       time.sleep(duration_seconds)
       GPIO.output(PUMP_PIN, GPIO.LOW)
   ```

3. **Add Authentication** - Protect web dashboard dengan login

4. **SSL/HTTPS** - Untuk secure WebSocket (wss://)

5. **Database Optimization** - Add indexes untuk faster queries

6. **Monitoring** - Add Prometheus metrics untuk system health

## 📚 API Endpoints

### REST API
- `GET /` - Web dashboard HTML
- `GET /api/telemetry` - Historical sensor data (JSON)
- `POST /api/water/{plant_name}` - Manual pump trigger

### WebSocket
- `WS /ws` - Real-time bidirectional communication

## 🎉 Selamat!

Sistem real-time sudah fully functional. Daemon mode dan web dashboard sekarang terhubung via WebSocket untuk monitoring dan control yang seamless.
