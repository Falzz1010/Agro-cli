# 🌱 AgroCLI Edge - Platform IoT Pertanian Pintar

> Sistem manajemen kebun cerdas yang dihosting sendiri dengan pemantauan dan otomatisasi waktu nyata. Sekarang sepenuhnya ditulis ulang dalam **Rust** untuk performa dan keandalan maksimal.

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7-blue.svg)](https://github.com/tokio-rs/axum)
[![Websocket](https://img.shields.io/badge/WebSocket-Real--Time-orange.svg)](https://github.com/tokio-rs/axum)
[![Ratatui](https://img.shields.io/badge/Ratatui-0.26-green.svg)](https://ratatui.rs/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## ✨ Fitur Utama

### 🌐 Dashboard Web Premium (v1.3.0)
- **Responsivitas Seluler Penuh** - Desain glassmorphic premium yang menyesuaikan dengan ponsel, tablet, dan desktop.
- **Feed Interaksi AI Agent** - Aliran waktu nyata tentang apa yang dipikirkan dan dilakukan oleh "otak" kebun Anda.
- **Grafik Interaktif** - Tren Kelembaban, Suhu, dan Kelembaban udara dengan rendering performa tinggi.
- **Riwayat Sensor & Ekspor** - Lihat data historis hingga 7 hari dan ekspor langsung ke CSV.
- **Kontrol Manual** - Memicu penyiraman dan memperbarui ambang batas tanaman secara jarak jauh.
- **Akses Aman** - Endpoint yang dilindungi oleh autentikasi dasar (basic authentication).

### 🤖 Mode AI Agent
- **Dukungan AI Multimodal** - Didukung oleh Google Gemini (Flash/Pro) untuk pengambilan keputusan cerdas.
- **Kontrol Bahasa Alami** - Kontrol kebun Anda dengan bahasa alami (Bahasa Indonesia/Inggris).
- **Kemampuan Tool-Calling** - AI dapat langsung menanyakan database dan memicu perangkat keras melalui `water_plant_action`.
- **Mode Simulasi** - Uji logika AI dengan aman tanpa kunci API.

### 🔌 Lapisan IoT & Performa
- **Inti Rust Performa Tinggi** - Mesin asinkron yang dioptimalkan menggunakan `tokio` dan `axum`.
- **Penyiaran Asinkron Langsung** - Komunikasi internal nol latensi antar modul melalui saluran khusus.
- **Mode Daemon** - Pemantauan otomatis 24/7, pengecekan cuaca, dan perlindungan kegagalan (failsafe).
- **Persistensi SQLite** - Penyimpanan data lokal yang andal dengan `sqlx`.
- **Ratatui TUI** - Antarmuka terminal yang apik untuk manajemen dan pemantauan cepat.

## 🚀 Memulai Cepat

### Prasyarat
- [Rust & Cargo](https://rustup.rs/) (v1.75+)
- [SQLite](https://www.sqlite.org/)

### Instalasi
```bash
# Clone repositori
git clone https://github.com/yourusername/AgroCLI.git
cd AgroCLI

# Build proyek
cargo build --release

# Inisialisasi kebun (hanya pertama kali)
cargo run -- init
```

### Penggunaan

**Terminal 1 - Dashboard Web & Mesin Logika:**
```bash
# Memulai server dan logika daemon
cargo run -- serve
```

**Terminal 2 - Antarmuka Interaktif (TUI):**
```bash
# Dashboard terminal yang apik
cargo run -- interactive
```

**Terminal 3 - AI Agent (Opsional):**
```bash
# Chat bahasa alami langsung
cargo run -- ai-agent
```

## 📚 Dokumentasi

- **[QUICKSTART.md](QUICKSTART.md)** - Panduan pengaturan 5 menit.
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Aliran data internal dan pola asinkron.
- **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)** - Detail API REST & WebSocket.
- **[AI_PROVIDER_GUIDE.md](AI_PROVIDER_GUIDE.md)** - Mengonfigurasi Gemini dan model AI lainnya.

## 🔧 Konfigurasi

Semua rahasia dan pengaturan server dikelola melalui `.env`:
```bash
PORT=8001
GEMINI_API_KEY=kunci_anda_di_sini
ADMIN_USERNAME=admin
ADMIN_PASSWORD=kata_sandi_anda
```

## 👨‍💻 Penulis

**Tim AgroCLI**
Dibuat dengan 💚 dan **Rust** untuk para penggemar pertanian pintar.

---
**Selamat Bertani! 🌱**
