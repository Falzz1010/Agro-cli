# Panduan Pemecahan Masalah Dashboard Web

## 🔍 Masalah: Grafik Tidak Muncul

Jika Anda melihat dashboard web tetapi grafik kosong atau tidak muncul, ikuti panduan ini.

---

## ✅ Daftar Periksa Cepat

Sebelum melakukan pemecahan masalah, pastikan:

- [ ] Server web sedang berjalan (`cargo run -- serve`).
- [ ] Anda telah menambahkan setidaknya satu tanaman.
- [ ] Mode Daemon sedang berjalan (untuk data waktu nyata).
- [ ] Konsol browser tidak menunjukkan kesalahan (F12 → tab Console).

---

## 📊 Mengapa Grafik Mungkin Kosong

### 1. Belum Ada Tanaman yang Ditambahkan

**Gejala:** Dashboard menunjukkan "Waiting for sensor data from Daemon..."

**Solusi:** Tambahkan tanaman terlebih dahulu via TUI atau CLI:
```bash
cargo run -- add --plant-type tomato --name "Tomat-Saya"
```

### 2. Daemon Tidak Berjalan

**Gejala:** Tanaman muncul tetapi data tidak pernah diperbarui, grafik tetap kosong.

**Penjelasan:** Dashboard web menampilkan data waktu nyata yang dikirim oleh daemon. Tanpa daemon, tidak ada data sensor yang disiarkan ke dashboard.

**Solusi:** Jalankan daemon di terminal terpisah:
```bash
cargo run -- daemon
```

### 3. WebSocket Tidak Terhubung

**Gejala:** Status menunjukkan "Live: Disconnected" berwarna merah.

**Solusi:** Pastikan server berjalan pada port yang benar (cek file `.env`). Akses dashboard di `http://127.0.0.1:8001`.

### 4. Chart.js Tidak Memuat

**Gejala:** Konsol menunjukkan kesalahan "Chart is not defined".

**Solusi:** Periksa koneksi internet (Chart.js dimuat dari CDN) atau unduh Chart.js secara lokal.

---

## 📈 Memahami Grafik

### Grafik Tren Kelembaban (Moisture Trends)
- **Sumbu X:** Waktu (JJ:MM:DD).
- **Sumbu Y:** Persentase kelembaban (0-100%).
- **Pembaruan:** Setiap 5 detik (saat daemon berjalan).

### Grafik Riwayat Sensor
- **Tujuan:** Melihat data historis (24 jam atau 7 hari).
- **Sumber Data:** Database (tabel `sensor_logs`).

---

**Versi:** 1.2.0  
**Pembaruan Terakhir:** 4 Maret 2026
