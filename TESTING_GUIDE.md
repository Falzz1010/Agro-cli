# 🧪 Testing Guide - AgroCLI Real-Time System

## Quick Test Commands

### Test 1: Basic Setup
```bash
# Initialize
python main.py init

# Add plants
python main.py add tomato "Test-Tomato"
python main.py add chili "Test-Chili"

# Verify
python main.py stats
```

### Test 2: Real-Time System
```bash
# Terminal 1
python main.py serve

# Terminal 2
python main.py daemon

# Browser
http://localhost:8000
```

### Test 3: WebSocket Connection
Open browser console (F12) and check for:
```
WebSocket connected
Received: {type: "sensor_update", ...}
```

### Test 4: Multi-Client Sync
1. Open 2 browser windows
2. Trigger pump in window 1
3. Verify window 2 receives update

### Test 5: Auto-Reconnect
1. Stop daemon (Ctrl+C)
2. Check dashboard shows "Disconnected"
3. Restart daemon
4. Verify auto-reconnect

## Expected Behavior

### Daemon Output
```
🕰️  Cycle Check: 14:23:45
Sensor Test-Tomato | Moisture: 45.2% | Temp: 28.5°C | Hum: 65.3%
```

### Web Dashboard
- 🟢 Real-Time Connected
- Live sensor readings updating
- Chart auto-scrolling
- Pump buttons functional

## Troubleshooting

### Issue: WebSocket not connecting
**Check:** Daemon running? Port 8000 open?

### Issue: No sensor data
**Check:** Plants added? Daemon loop running?

### Issue: Chart not updating
**Check:** Browser console for errors

---
For detailed testing, see REALTIME_SETUP.md
