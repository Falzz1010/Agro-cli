# Web Dashboard Troubleshooting Guide

## 🔍 Problem: Charts Not Showing

If you see the web dashboard but the charts are empty or not displaying, follow this guide.

---

## ✅ Quick Checklist

Before troubleshooting, make sure:

- [ ] Web server is running (`cargo run -- serve`)
- [ ] You have added at least one plant
- [ ] Daemon mode is running (for real-time data)
- [ ] Browser console shows no errors (F12 → Console tab)

---

## 📊 Why Charts Might Be Empty

### 1. No Plants Added Yet

**Symptom:** Dashboard shows "Waiting for sensor data from Daemon..."

**Solution:** Add a plant first

```bash
# In a new terminal (while web server is running)
cargo run

# From the TUI menu, select:
# ➕ Add New Plant

# Or use CLI directly:
cargo run -- add --plant-type tomato --name "My-Tomato"
```

### 2. Daemon Not Running

**Symptom:** Plants show but data never updates, charts remain empty

**Explanation:** The web dashboard displays real-time data sent by the daemon. Without the daemon running, no sensor data is broadcast to the dashboard.

**Solution:** Start the daemon in a separate terminal

```bash
# Terminal 1: Web Server
cargo run -- serve

# Terminal 2: Daemon (sends sensor data)
cargo run -- daemon

# Terminal 3: TUI (optional, for management)
cargo run
```

**Expected Output from Daemon:**
```
🔌 AgroCLI Daemon Activated. (Press 'q' to return to menu/exit)

Cycle Check: 14:23:45
✅ My-Tomato: Moisture 45.2% (OK)
✓ Real-time broadcast: 1/1 plants
```

### 3. WebSocket Not Connected

**Symptom:** Status shows "Live: Disconnected" in red

**Check:**
1. Open browser console (F12)
2. Look for WebSocket errors
3. Check if URL is correct

**Common Issues:**
- Wrong port number
- Server not running
- Firewall blocking WebSocket

**Solution:**
```bash
# Make sure server is running on correct port
# Check .env file:
PORT=8001

# Access dashboard at:
http://127.0.0.1:8001
```

### 4. Chart.js Not Loading

**Symptom:** Console shows "Chart is not defined" error

**Check:** Open browser console (F12) and look for:
```
Failed to load resource: https://cdn.jsdelivr.net/npm/chart.js
```

**Solution:**
- Check internet connection (Chart.js loads from CDN)
- Or download Chart.js locally and update `index.html`

---

## 🚀 Complete Setup Guide

### Step-by-Step: Getting Charts to Display

#### 1. Initialize Database (First Time Only)
```bash
cargo run -- init
```

#### 2. Add Plants
```bash
# Add a tomato plant
cargo run -- add --plant-type tomato --name "Tomato-1"

# Add a chili plant
cargo run -- add --plant-type chili --name "Chili-1"
```

#### 3. Start Web Server
```bash
# Terminal 1
cargo run -- serve
```

You should see:
```
🌐 [WEB] Real-Time Dashboard running at http://0.0.0.0:8001
```

#### 4. Start Daemon
```bash
# Terminal 2
cargo run -- daemon
```

You should see sensor readings every 5 seconds:
```
Cycle Check: 14:23:45
✅ Tomato-1: Moisture 45.2% (OK)
✅ Chili-1: Moisture 38.7% (OK)
✓ Real-time broadcast: 2/2 plants
```

#### 5. Open Dashboard
```
http://127.0.0.1:8001
```

**What You Should See:**
- Connection status: "Live: Connected" (green)
- Plant cards with current sensor readings
- Two charts:
  - "Moisture Trends (%)" - showing moisture levels
  - "Temp & Humidity Trends" - showing temperature and humidity
- Charts update every 5 seconds as daemon sends data

---

## 🔧 Advanced Troubleshooting

### Check WebSocket Connection

Open browser console (F12) and run:
```javascript
// Check if WebSocket is connected
console.log('WebSocket URL:', `ws://${window.location.host}/ws`);
```

### Check if Data is Being Received

In browser console:
```javascript
// Monitor WebSocket messages
const ws = new WebSocket(`ws://${window.location.host}/ws`);
ws.onmessage = (event) => {
    console.log('Received:', JSON.parse(event.data));
};
```

### Check Database for Historical Data

```bash
# Check if sensor logs exist
sqlite3 data/garden.db "SELECT COUNT(*) FROM sensor_logs;"

# View recent logs
sqlite3 data/garden.db "SELECT * FROM sensor_logs ORDER BY timestamp DESC LIMIT 10;"
```

### Verify Plant Configuration

```bash
# Check active plants
sqlite3 data/garden.db "SELECT * FROM plants WHERE status='active';"
```

---

## 📈 Understanding the Charts

### Moisture Trends Chart
- **X-axis:** Time (HH:MM:SS)
- **Y-axis:** Moisture percentage (0-100%)
- **Lines:** One line per plant
- **Updates:** Every 5 seconds (when daemon is running)
- **Data Points:** Last 30 readings (rolling window)

### Temp & Humidity Chart
- **Left Y-axis:** Temperature (°C)
- **Right Y-axis:** Humidity (%)
- **Solid lines:** Temperature
- **Dashed lines:** Humidity
- **Updates:** Every 5 seconds
- **Data Points:** Last 30 readings

### Sensor History Chart
- **Purpose:** View historical data (24h or 7d)
- **How to use:**
  1. Select a plant from dropdown
  2. Choose time period (24h or 7d)
  3. Chart displays historical trends
- **Data source:** Database (sensor_logs table)

---

## 🐛 Common Errors and Solutions

### Error: "Can't reach this page"
**Solution:** Use `http://127.0.0.1:8001` instead of `http://0.0.0.0:8001`

### Error: "WebSocket connection failed"
**Causes:**
- Server not running
- Wrong port
- Firewall blocking

**Solution:**
```bash
# Check if server is running
netstat -an | findstr "8001"  # Windows
netstat -an | grep "8001"     # Linux/Mac

# Try different port in .env
PORT=8080
```

### Error: "No data found for plant"
**Causes:**
- Plant was just added (no history yet)
- Daemon never ran
- Database is empty

**Solution:**
```bash
# Run daemon for at least 1 minute to collect data
cargo run -- daemon

# Wait for several sensor readings
# Then check dashboard history
```

### Charts Show Flat Lines
**Cause:** Mock sensors generate random data, but values might be similar

**This is normal!** Mock sensors simulate stable conditions. Real sensors will show more variation.

---

## 💡 Tips for Best Experience

1. **Keep Daemon Running**
   - Charts only update when daemon is active
   - Daemon sends sensor data every 5 seconds

2. **Multiple Terminals**
   ```
   Terminal 1: cargo run -- serve    (Web Server)
   Terminal 2: cargo run -- daemon   (Data Collection)
   Terminal 3: cargo run             (TUI Management)
   ```

3. **Browser Refresh**
   - If charts don't appear, try hard refresh: `Ctrl+F5` (Windows) or `Cmd+Shift+R` (Mac)

4. **Check Console**
   - Always check browser console (F12) for errors
   - Look for red error messages

5. **Historical Data**
   - History chart requires at least 1 hour of data
   - Run daemon for longer periods to see trends

---

## 📞 Still Having Issues?

### Collect Debug Information

1. **Check server logs:**
   ```bash
   # Run with debug logging
   RUST_LOG=debug cargo run -- serve
   ```

2. **Check daemon logs:**
   ```bash
   RUST_LOG=debug cargo run -- daemon
   ```

3. **Browser console:**
   - Open DevTools (F12)
   - Go to Console tab
   - Copy any error messages

4. **Network tab:**
   - Open DevTools (F12)
   - Go to Network tab
   - Check if WebSocket connection is established
   - Look for failed requests

### Report Issue

If charts still don't work, create an issue with:
- Operating system
- Browser and version
- Server logs
- Browser console errors
- Steps you've tried

---

## ✅ Success Checklist

When everything is working correctly, you should see:

- ✅ Web server running on http://127.0.0.1:8001
- ✅ Daemon running and showing sensor readings
- ✅ Dashboard shows "Live: Connected" (green)
- ✅ Plant cards display current sensor values
- ✅ Moisture chart shows lines for each plant
- ✅ Temp/Humidity chart shows temperature and humidity trends
- ✅ Charts update every 5 seconds
- ✅ History chart shows data when plant is selected

---

**Version:** 1.2.0  
**Last Updated:** March 4, 2026
