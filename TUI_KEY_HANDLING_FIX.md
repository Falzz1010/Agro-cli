# Perbaikan Penanganan Tombol TUI

## 🐛 Masalah

Pengguna melaporkan bahwa menekan `q` atau `Esc` pada Layar Langsung (Live Tasks, Garden Stats, Live Sensor) tidak membuahkan hasil (tidak kembali ke menu utama).

## 🔍 Akar Masalah

Masalah ini disebabkan oleh pemfilteran aktivitas tombol yang terlalu ketat:

```rust
// KODE LAMA - Terlalu ketat
if key.kind != KeyEventKind::Press { return; }
```

Filter ini menolak aktivitas `KeyEventKind::Repeat`, yang umum terjadi di Windows ketika sebuah tombol ditekan lama atau ditekan dengan cepat.

## ✅ Solusi

### 1. Filter Aktivitas Tombol yang Lebih Longgar

Mengubah dari hanya menerima `Press` menjadi hanya menolak `Release`:

```rust
// KODE BARU - Lebih permisif
if key.kind == KeyEventKind::Release { return; }
```

Ini memungkinkan aktivitas `Press` dan `Repeat` untuk diproses.

### 2. Reset Status Secara Langsung

Alih-alih memanggil `self.back()`, langsung reset status untuk menghindari potensi masalah:

```rust
// KODE BARU
KeyCode::Char('q') => {
    self.screen = Screen::MainMenu;
    self.pending = Pending::None;
}
KeyCode::Esc => {
    self.screen = Screen::MainMenu;
    self.pending = Pending::None;
}
```

### 3. Pengurangan Interval Polling

Meningkatkan responsivitas dengan mengurangi interval polling aktivitas:

```rust
// LAMA: 200ms
event::poll(Duration::from_millis(200))

// BARU: 100ms (2x lebih responsif)
event::poll(Duration::from_millis(100))
```

## 🧪 Pengujian

1. **Tekan Cepat**: Masuk ke layar Live Tasks, tekan `q` dengan cepat. **Hasil:** ✅ Kembali ke menu utama segera.
2. **Tekan Lama**: Tahan tombol `q` selama 1 detik. **Hasil:** ✅ Kembali ke menu utama (tidak berulang kali).
3. **Tombol Esc**: Tekan `Esc`. **Hasil:** ✅ Kembali ke menu utama.

## 📊 Jenis Aktivitas Tombol (Windows)

| Jenis Aktivitas | Kapan Terpicu | Harus Diproses? |
|-----------------|---------------|-----------------|
| `Press`         | Tekanan awal tombol | ✅ Ya |
| `Repeat`        | Tombol ditahan | ✅ Ya |
| `Release`       | Tombol dilepas | ❌ Tidak |

---

**Versi:** 1.2.0  
**Tanggal:** 4 Maret 2026  
**Status:** ✅ Diperbaiki
