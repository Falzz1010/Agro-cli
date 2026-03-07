# Panduan Akses Dashboard Web

## 🌐 Mengakses Dashboard Web

### Jawaban Cepat

**Jika Anda melihat `ERR_ADDRESS_INVALID` saat mengakses dashboard web:**

❌ **JANGAN GUNAKAN:** `http://0.0.0.0:8001`  
✅ **GUNAKAN SEBAGAI GANTINYA:** `http://127.0.0.1:8001` atau `http://localhost:8001`

---

## 📖 Memahami Masalah

### Apa itu 0.0.0.0?

`0.0.0.0` adalah alamat IP khusus yang berarti "semua antarmuka jaringan" pada sisi server. Ini digunakan untuk:
- Melakukan binding pada server agar mendengarkan di semua antarmuka jaringan yang tersedia.
- Memungkinkan koneksi dari alamat IP mana pun (localhost, LAN, dll.).

Namun, **browser tidak dapat terhubung langsung ke 0.0.0.0** karena itu bukan alamat tujuan yang valid.

### Alamat yang Harus Digunakan

| Alamat | Deskripsi | Kapan Digunakan |
|---------|-------------|-------------|
| `127.0.0.1` | Alamat loopback IPv4 | ✅ Selalu berfungsi secara lokal |
| `localhost` | Hostname untuk loopback | ✅ Selalu berfungsi secara lokal |
| `0.0.0.0` | Semua antarmuka (binding server) | ❌ Jangan pernah digunakan di browser |
| `192.168.x.x` | IP jaringan lokal Anda | ✅ Untuk akses LAN |

---

## 🚀 Panduan Akses Langkah demi Langkah

### 1. Mulai Server Web

Buka terminal dan jalankan:
```bash
cargo run -- serve
```

### 2. Buka Browser Anda

**Pilih SALAH SATU dari URL berikut:**

#### Opsi A: Menggunakan 127.0.0.1 (Direkomendasikan)
```
http://127.0.0.1:8001
```

#### Opsi B: Menggunakan localhost
```
http://localhost:8001
```

#### Opsi C: Menggunakan IP LAN Anda (untuk akses dari perangkat lain)
Pertama, cari alamat IP Anda (Windows: `ipconfig`, Linux/Mac: `ip addr show`). Gunakan alamat tersebut, misalnya:
```
http://192.168.1.100:8001
```

---

## 🔧 Pemecahan Masalah

### Masalah: "Can't reach this page" atau "ERR_ADDRESS_INVALID"
**Solusi:** Ubah `0.0.0.0` menjadi `127.0.0.1` pada URL.

### Masalah: "Connection refused"
**Solusi:** Pastikan server web sedang berjalan (`cargo run -- serve`) dan periksa apakah nomor port sudah benar di file `.env`.

---

**Versi:** 1.3.0  
**Pembaruan Terakhir:** 5 Maret 2026
