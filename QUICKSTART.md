# ⚡ Quick Start Guide - AgroCLI Real-Time System

## 🎯 Goal
Dalam 5 menit, kamu akan punya sistem smart farming yang fully real-time dengan:
- Live sensor monitoring
- Auto-updating web dashboard
- Automated watering system

## 📋 Prerequisites
- Python 3.8+ installed
- Terminal/Command Prompt
- Web browser

## 🚀 Step-by-Step Setup

### 1️⃣ Install Dependencies (1 minute)
```bash
pip install -r requirements.txt
```

Expected output:
```
Successfully installed fastapi-0.109.0 uvicorn-0.27.0 websockets-12.0 ...
```

### 2️⃣ Initialize System (30 seconds)
```bash
python main.py init
```

Expected output:
```
┌────────────────────────────────────────┐
│ 🌱 AgroCLI Initialized!                │
│ Your local tracking database is ready. │
└────────────────────────────────────────┘
```

### 3️⃣ Add Test Plants (30 seconds)
```bash
python main.py add tomato "My-Tomato"
python main.py add chili "My-Chili"
```

Expected output:
```
Success! Added My-Tomato (tomato) to your garden.
Success! Added My-Chili (chili) to your garden.
```

### 4️⃣ Start Web Dashboard (Terminal 1)
```bash
python main.py serve
```

Expected output:
```
┌─────────────────────────────────────────┐
│ 🌐 Starting AgroCLI Web Server...      │
│ Open http://localhost:8000 in browser  │
└─────────────────────────────────────────┘
INFO:     Uvicorn running on http://0.0.0.0:8000
```

**✅ Checkpoint:** Open browser → `http://localhost:8000`
You should see the dashboard with your 2 plants.

### 5️⃣ Start Daemon Mode (Terminal 2)
Open a NEW terminal window:
```bash
python main.py daemon
```

Expected output:
```
┌────────────────────────────────────────┐
│ 🤖 AgroCLI Daemon Mode Activated      │
│ Monitoring sensors 24/7                │
└────────────────────────────────────────┘
✓ WebSocket broadcasting enabled

🕰️  Cycle Check: 14:23:45
Sensor My-Tomato | Moisture: 45.2% | Temp: 28.5°C | Hum: 65.3%
Sensor My-Chili | Moisture: 52.1% | Temp: 28.5°C | Hum: 65.3%
```

### 6️⃣ Watch Real-Time Magic! ✨

Go back to your browser. You should now see:

1. **Connection Status:** 🟢 Real-Time Connected
2. **Live Sensor Readings:** Updating every 5 seconds
3. **Chart:** Auto-scrolling with new data points
4. **Pulsing Green Dots:** On each plant card

## 🧪 Test the System

### Test 1: Manual Pump Trigger
1. Click "💧 Trigger Pump" on any plant
2. Watch the button change to "💧 Pumping..."
3. See notification appear
4. Check Terminal 2 for pump activation log

### Test 2: Multiple Browser Windows
1. Open `http://localhost:8000` in another browser/tab
2. Trigger pump from Browser 1
3. Browser 2 should also see the pump event
4. Both receive same sensor updates

### Test 3: Auto-Reconnect
1. Stop daemon (Ctrl+C in Terminal 2)
2. Dashboard shows: 🔴 Disconnected - Reconnecting...
3. Restart daemon: `python main.py daemon`
4. Dashboard auto-reconnects: 🟢 Real-Time Connected

## 📱 Access from Phone/Tablet

1. Find your computer's IP address:
   ```bash
   # Windows
   ipconfig
   
   # Mac/Linux
   ifconfig | grep inet
   ```

2. On your phone, open browser:
   ```
   http://192.168.1.XXX:8000
   ```
   (Replace XXX with your IP)

3. You should see the same dashboard, updating in real-time!

## 🎮 Interactive Mode (Bonus)

Want a menu-driven interface? Just run:
```bash
python main.py
```

You'll get a beautiful interactive menu:
```
┌────────────────────────────────────────┐
│      🌱 AgroCLI - Smart Farming       │
│   === THE INTELLIGENT GARDEN BRAIN === │
└────────────────────────────────────────┘

What would you like to do?
❯ 🌱 Check Today's Tasks
  ➕ Add New Plant
  📊 View Garden Stats
  ✂️  Harvest a Plant
  ☁️  Configure Weather API
  🔌 Run Daemon Automation
  🌐 Start Web Dashboard
  ❌ Exit
```

## 🔧 Common Issues & Fixes

### Issue: "Port 8000 already in use"
**Fix:** Kill existing process or use different port:
```bash
# Windows
netstat -ano | findstr :8000
taskkill /PID <PID> /F

# Mac/Linux
lsof -ti:8000 | xargs kill -9
```

### Issue: "Module not found: fastapi"
**Fix:** Install dependencies again:
```bash
pip install --upgrade -r requirements.txt
```

### Issue: WebSocket not connecting
**Fix:** 
1. Make sure daemon is running
2. Check firewall settings
3. Try `http://127.0.0.1:8000` instead of localhost

### Issue: No sensor data showing
**Fix:**
1. Verify daemon is running (check Terminal 2)
2. Refresh browser page
3. Check browser console (F12) for errors

## 📊 What's Happening Behind the Scenes?

```
Terminal 1 (Web Server)          Terminal 2 (Daemon)
      │                                │
      │                                │
      │◄───────WebSocket───────────────┤
      │                                │
      │         Sensor Data            │
      │         Pump Events            │
      │         System Alerts          │
      │                                │
      ▼                                ▼
   Browser                         Hardware
   (You)                          (Sensors)
```

Every 5 seconds:
1. Daemon reads sensors (currently mock data)
2. Daemon broadcasts data via WebSocket
3. Web server pushes to all connected browsers
4. Your browser updates chart & displays
5. Every 60 seconds, data is saved to database

## 🎯 Next Steps

Now that you have real-time system running:

1. **Customize Plant Rules** - Edit `plants.yaml`:
   ```yaml
   tomato:
     water_interval_days: 1
     min_moisture_level: 40.0  # Trigger pump below 40%
     water_ml: 250
   ```

2. **Configure Weather API** - Get free key from openweathermap.org:
   ```bash
   python main.py
   # Select: ☁️  Configure Weather API
   ```

3. **View Statistics**:
   ```bash
   python main.py stats
   python main.py stats --export garden_data.csv
   ```

4. **Read Full Documentation**:
   - `REALTIME_SETUP.md` - Detailed real-time features
   - `ARCHITECTURE.md` - System architecture & design
   - `README.md` - Complete feature list

## 🎉 Congratulations!

You now have a fully functional real-time smart farming system! 

The system is currently using mock sensors. To connect real hardware:
- Edit `hardware/sensors.py` for real sensor readings
- Edit `hardware/pump.py` for real pump control
- See `ARCHITECTURE.md` for hardware integration guide

## 💡 Pro Tips

1. **Keep both terminals visible** - Use split screen to monitor both
2. **Check daemon logs** - All important events are logged there
3. **Use mobile access** - Monitor your garden from anywhere in the house
4. **Export data regularly** - `python main.py stats --export backup.csv`
5. **Test failsafe** - System locks pump after 5 consecutive triggers

## 🆘 Need Help?

- Check `REALTIME_SETUP.md` for troubleshooting
- Review `ARCHITECTURE.md` for system design
- Open GitHub issue for bugs
- Read code comments for implementation details

---

**Happy Farming! 🌱**
