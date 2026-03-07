# Panduan Konfigurasi Provider AI (v1.3.0)

## 🤖 Ikhtisar

AgroCLI Edge menggunakan **Google Gemini** sebagai otak cerdas utamanya. Sistem ini dibangun dengan arsitektur tool-calling yang memungkinkan AI berinteraksi langsung dengan perangkat keras dan database kebun Anda.

---

## 🎯 Provider Utama: Google Gemini

### Model yang Mendukung Tool-Calling
- **Gemini 1.5 Flash** - Sangat direkomendasikan. Sangat cepat dan efisien untuk manajemen kebun harian.
- **Gemini 1.5 Pro** - Terbaik untuk penalaran multi-tanaman yang kompleks dan diagnosis penyakit.

### Instruksi Pengaturan
1. **Dapatkan Kunci API**: Kunjungi [Google AI Studio](https://aistudio.google.com/app/apikey).
2. **Konfigurasi Env**: Tambahkan baris berikut ke file `.env` Anda:
   ```bash
   GEMINI_API_KEY=kunci_api_asli_anda_di_sini
   ```
3. **Mesin Rust**: Inti Rust akan secara otomatis mendeteksi kunci ini saat startup.

---

## 🛠️ Kemampuan AI (Tool-Calling)

AI Agent bukan sekadar chatbot; ia memiliki "tangan" untuk bekerja di kebun Anda:

- **`get_garden_status`**: Menanyakan database SQLite untuk kelembaban, suhu, dan kesehatan tanaman secara real-time.
- **`water_plant_action`**: Memicu pompa fisik (atau pompa tiruan) untuk tanaman tertentu.
- **`search_plant_database`**: Mencari aturan perawatan di `plants.yaml`.

### Contoh Perintah
- "Bagaimana kondisi kebun saya hari ini?" (Pertanyaan)
- "Siram tanaman tomat sekarang." (Tindakan)
- "Apakah cabai saya butuh pupuk?" (Penalaran)

---

## 🧪 Mode Simulasi

Jika Anda tidak memiliki kunci API, AgroCLI Edge akan beralih ke **Mode Simulasi** secara default.

- **Logika**: Analisis berbasis aturan regex sederhana.
- **Fitur**: Mendukung perintah dasar "status" dan "siram".
- **Biaya**: 100% Gratis dan Offline.

---

## 🔧 Pemecahan Masalah

### API 401 Unauthorized
- **Penyebab**: Kunci API tidak valid.
- **Solusi**: Periksa format file `.env` Anda. Pastikan tidak ada spasi di sekitar tanda `=`.

### API 429 Rate Limit
- **Penyebab**: Batas kuota gratis tercapai.
- **Solusi**: Gemini Flash mengizinkan sekitar 15 permintaan per menit pada paket gratis. Tunggu sejenak atau gunakan kunci API lain.

### Kegagalan Alat (Tool Failure)
- **Penyebab**: AI mencoba menyiram tanaman yang tidak ada.
- **Solusi**: Tanya AI "Tampilkan semua tanaman saya" terlebih dahulu untuk melihat nama-nama yang benar.

---
**Versi:** 1.3.0  
**Pembaruan Terakhir:** 5 Maret 2026
