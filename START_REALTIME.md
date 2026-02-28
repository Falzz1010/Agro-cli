# 🚀 Quick Start - Real-Time System

## Masalah Saat Ini:
CLI Monitor tidak terima data karena daemon dan web server jalan di proses terpisah.

## ✅ Solusi Simple:

### Opsi 1: Lihat Data di Web Dashboard (PALING MUDAH)

**Terminal 1 - Web Server:**
```powershell
.\venv\Scripts\Activate.ps1
python main.py serve
```

**Terminal 2 - Daemon:**
```powershell
.\venv\Scripts\Activate.ps1
python main.py daemon
```

**Browser:**
```
http://localhost:8000
```

Web dashboard akan auto-update setiap kali daemon log data ke database!

### Opsi 2: CLI Monitor Baca Database (FALLBACK)

Kalau WebSocket tidak jalan, CLI Monitor bisa baca langsung dari database:

**Terminal 3:**
```powershell
.\venv\Scripts\Activate.ps1
python main.py
# Pilih: 📊 View Garden Stats (Real-Time)
```

Ini akan auto-refresh setiap 1 detik dari database.

### Opsi 3: Lihat Log Daemon Langsung

Daemon sudah menampilkan sensor data real-time di terminal:
```
Sensor Tomat-Saya | Moisture: 58.0% | Temp: 31.4°C | Hum: 51.8%
```

Ini adalah monitoring real-time paling simple!

## 🎯 Rekomendasi:

Untuk sekarang, gunakan **Web Dashboard** karena paling reliable:

1. Start web server
2. Start daemon  
3. Buka browser: http://localhost:8000
4. Lihat chart auto-update!

CLI Monitor via WebSocket masih experimental dan butuh setup lebih kompleks.
