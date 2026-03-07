# ⚡ Panduan Memulai Cepat - AgroCLI Edge

## 🎯 Tujuan
Dalam 5 menit, Anda akan memiliki sistem pertanian pintar berperforma tinggi yang berjalan:
- **Inti Rust**: Pemrosesan sensor sangat cepat.
- **Web Axum**: Dashboard waktu nyata premium.
- **AI Agent**: Otak kebun yang cerdas.

## 📋 Prasyarat
- [Rust & Cargo](https://rustup.rs/) (v1.75+)
- Git terinstal.

## 🚀 Pengaturan 5 Menit

### 1️⃣ Clone & Build
```bash
git clone https://github.com/yourusername/AgroCLI.git
cd AgroCLI
cargo build --release
```

### 2️⃣ Inisialisasi Kebun
```bash
cargo run -- init
```
*Ini akan membuat `data/garden.db` lokal Anda dan menyiapkan skema.*

### 3️⃣ Tambahkan Tanaman Pertama Anda
```bash
cargo run -- add tomato "Tomat-Saya"
```

### 4️⃣ Jalankan Mesin (Terminal 1)
```bash
cargo run -- serve
```
*Ini akan memulai **Server Web**, **Hub WebSocket**, dan **Daemon Otomatisasi** sekaligus.*

**✅ Titik Pemeriksaan:** Buka `http://localhost:8001` di browser Anda.

### 5️⃣ Jalankan TUI Interaktif (Terminal 2)
```bash
cargo run -- interactive
```
*Antarmuka terminal yang indah untuk pemantauan lokal.*

---

## 🧪 Uji Cepat

1. **Pemeriksaan Dashboard**: Buka dashboard web. Anda seharusnya melihat "Tomat-Saya" muncul.
2. **Pertanyaan AI**: Ketik perintah `ai-agent` atau gunakan feed AI di dashboard untuk bertanya: "Bagaimana kondisi tomat saya?"
3. **Siram Manual**: Klik tombol "💧 Trigger Pump" di dashboard. Perhatikan log di Terminal 1 untuk aktivasi.

## 🔧 Konfigurasi Pro
- **Aturan Perawatan**: Edit `plants.yaml` untuk mengubah ambang batas kelembaban.
- **Otak AI**: Tambahkan `GEMINI_API_KEY` Anda ke `.env` untuk kecerdasan penuh.
- **Akses**: Akses dari ponsel Anda via `http://[ip-anda]:8001`.

---
**Selamat Bertani! 🌱**
