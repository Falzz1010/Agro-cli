# 📺 Live Terminal Features - Real-Time CLI

## ✨ Fitur Baru: Real-Time Terminal Monitoring

Sekarang menu interactive AgroCLI punya fitur live monitoring yang auto-refresh!

## 🎯 Fitur Real-Time di Terminal

### 1. 🌱 Check Today's Tasks (Real-Time)
**Auto-refresh setiap 2 detik**

Menampilkan:
- Live task list dengan status
- Weather condition real-time
- Auto-update timestamp
- Color-coded status indicators

**Cara pakai:**
```bash
python main.py
# Pilih: 🌱 Check Today's Tasks (Real-Time)
```

**Tampilan:**
```
┌─ Live Task Monitor | 09:45:23 | 🌦️  Kediri: Clear ─┐
│ Plant      │ Type   │ Water      │ Fertilize │ Status  │
│ Tomat-1    │ tomato │ 💧 250ml   │ OK        │ ⚠️  Action│
│ Cabai-1    │ chili  │ OK         │ 🌾 Yes    │ ⚠️  Action│
└────────────────────────────────────────────────────────┘
```

**Fitur:**
- ✅ Auto-refresh setiap 2 detik
- ✅ Live weather updates
- ✅ Real-time timestamp
- ✅ Press Ctrl+C untuk exit
- ✅ Bisa mark tasks as done setelah exit

### 2. 📊 View Garden Stats (Real-Time)
**Auto-refresh setiap 1 detik**

Menampilkan:
- Active plants count
- Harvested plants count
- Live plant table dengan details
- Auto-updating timestamp

**Cara pakai:**
```bash
python main.py
# Pilih: 📊 View Garden Stats (Real-Time)
```

**Tampilan:**
```
┌─ AgroCLI Live Stats | 09:45:23 ─┐
│ 🌱 Active Plants:    5          │
│ 🎉 Harvested:        3          │
│ 📊 Total:            8          │
└──────────────────────────────────┘

┌─ Active Plants ──────────────────┐
│ Name    │ Type   │ Planted    │  │
│ Tomat-1 │ tomato │ 2026-02-28 │  │
│ Cabai-1 │ chili  │ 2026-02-28 │  │
└──────────────────────────────────┘
```

**Fitur:**
- ✅ Auto-refresh setiap 1 detik
- ✅ Live statistics
- ✅ Real-time plant list
- ✅ Press Ctrl+C untuk exit

### 3. 📡 Live Sensor Monitor
**Auto-refresh setiap 2 detik**

Menampilkan:
- Real-time sensor readings
- Moisture, temperature, humidity
- Status indicators (OK/LOW)
- Auto-updating timestamp

**Cara pakai:**
```bash
python main.py
# Pilih: 📡 Live Sensor Monitor
```

**Tampilan:**
```
┌─ 🌡️  Live Sensor Readings | 09:45:23 ─┐
│ Plant   │ 💧 Moisture │ 🌡️  Temp │ 💨 Humidity │ Status │
│ Tomat-1 │   45.2%    │  28.5°C │    65.3%   │ ✓ OK  │
│ Cabai-1 │   28.1%    │  28.5°C │    65.3%   │ ⚠️  LOW │
└──────────────────────────────────────────────────────┘
```

**Fitur:**
- ✅ Auto-refresh setiap 2 detik
- ✅ Live sensor data (mock/real)
- ✅ Color-coded status
- ✅ Moisture threshold detection
- ✅ Press Ctrl+C untuk exit

## 🎮 Cara Menggunakan

### Quick Start
```bash
# Jalankan interactive mode
python main.py

# Menu akan muncul dengan pilihan:
What would you like to do?
❯ 🌱 Check Today's Tasks (Real-Time)
  ➕ Add New Plant
  📊 View Garden Stats (Real-Time)
  📡 Live Sensor Monitor
  ✂️  Harvest a Plant
  ☁️  Configure Weather API
  🔌 Run Daemon Automation
  🌐 Start Web Dashboard
  ❌ Exit
```

### Navigasi
- **Arrow Keys** - Pilih menu
- **Enter** - Konfirmasi pilihan
- **Ctrl+C** - Exit dari live monitor
- **ESC** - Cancel (di beberapa prompt)

## 🆚 Perbedaan Mode

### CLI Mode (Sekali Jalan)
```bash
python main.py today        # Cek sekali, tidak auto-refresh
python main.py stats        # Lihat sekali, tidak auto-refresh
```

### Live Terminal Mode (Auto-Refresh)
```bash
python main.py
# Pilih menu dengan "(Real-Time)"
# Auto-refresh terus sampai Ctrl+C
```

### Web Dashboard Mode (Full Real-Time)
```bash
# Terminal 1
python main.py serve

# Terminal 2
python main.py daemon

# Browser
http://localhost:8000
```

## 🎨 Visual Features

### Color Coding
- 🟢 **Green** - OK status, success messages
- 🔴 **Red** - Warning, action needed
- 🟡 **Yellow** - Info, skip messages
- 🔵 **Blue** - Water-related info
- 🟣 **Magenta** - Headers, titles
- ⚪ **Cyan** - Plant names, data

### Status Indicators
- ✓ OK - Everything normal
- ⚠️  Action - Needs attention
- ⚠️  LOW - Below threshold
- 💧 - Water needed
- 🌾 - Fertilizer needed
- 🌦️  - Weather info

## 📊 Performance

### Refresh Rates
- **Task Monitor:** 2 seconds
- **Stats Monitor:** 1 second
- **Sensor Monitor:** 2 seconds

### Resource Usage
- **CPU:** < 2% (terminal rendering)
- **Memory:** ~30MB (Rich library)
- **Network:** Only for weather API

## 🔧 Customization

### Change Refresh Rate
Edit `main.py`:

```python
# Task Monitor
time.sleep(2)  # Change to 5 for slower refresh

# Stats Monitor
time.sleep(1)  # Change to 3 for slower refresh

# Sensor Monitor
time.sleep(2)  # Change to 5 for slower refresh
```

### Change Table Style
Edit `main.py`:

```python
table = Table(
    show_header=True,
    header_style="bold magenta",  # Change color
    border_style="green",         # Add border color
    title_style="bold cyan"       # Change title color
)
```

## 💡 Tips & Tricks

### 1. Multi-Monitor Setup
Buka multiple terminals untuk monitoring berbeda:
- Terminal 1: Live Task Monitor
- Terminal 2: Live Sensor Monitor
- Terminal 3: Daemon Mode
- Browser: Web Dashboard

### 2. Quick Exit
Press `Ctrl+C` untuk keluar dari live monitor, lalu:
- Task Monitor: Bisa mark tasks as done
- Stats Monitor: Langsung exit
- Sensor Monitor: Langsung exit

### 3. Combine with Web Dashboard
Jalankan live terminal monitor sambil web dashboard:
- Terminal 1: `python main.py daemon`
- Terminal 2: `python main.py serve`
- Terminal 3: `python main.py` → Pilih live monitor
- Browser: `http://localhost:8000`

## 🐛 Troubleshooting

### Issue: Terminal flickering
**Fix:** Reduce refresh rate (increase sleep time)

### Issue: Colors not showing
**Fix:** Use terminal yang support ANSI colors (Windows Terminal, iTerm2, etc.)

### Issue: Layout broken
**Fix:** Resize terminal window (minimum 80x24)

### Issue: Ctrl+C not working
**Fix:** Press Ctrl+C multiple times or use Ctrl+Z

## 🎉 Summary

Sekarang semua fitur CLI punya mode real-time:

✅ **Check Today's Tasks** → Live auto-refresh
✅ **View Garden Stats** → Live auto-refresh
✅ **Sensor Monitor** → Live auto-refresh
✅ **Daemon Mode** → Background automation
✅ **Web Dashboard** → Full real-time web UI

Semua mode bisa diakses dari menu interactive yang sama!

---

**Happy Real-Time Farming! 🌱**
