# 🤝 Berkontribusi di AgroCLI

Terima kasih atas minat Anda untuk berkontribusi di AgroCLI! Dokumen ini memberikan panduan untuk berkontribusi pada proyek ini.

## 🌟 Cara Berkontribusi

- 🐛 Melaporkan bug
- 💡 Menyarankan fitur baru
- 📝 Memperbaiki dokumentasi
- 🔧 Mengirimkan perbaikan bug
- ✨ Menambahkan fitur baru
- 🧪 Menulis pengujian (tests)
- 🌍 Menerjemahkan ke bahasa lain

## 🚀 Memulai

### 1. Fork Repositori
```bash
# Fork di GitHub, lalu clone fork Anda
git clone https://github.com/YOUR_USERNAME/agrocli.git
cd agrocli
```

### 2. Siapkan Lingkungan Pengembangan
```bash
# Buat lingkungan virtual
python -m venv venv
source venv/bin/activate  # Di Windows: venv\Scripts\activate

# Instal dependensi
pip install -r requirements.txt

# Instal dependensi pengembangan
pip install pytest black flake8 mypy
```

### 3. Buat Branch
```bash
git checkout -b feature/nama-fitur-anda
# atau
git checkout -b fix/deskripsi-bug
```

## 📋 Panduan Pengembangan

### Gaya Kode

Kami mengikuti PEP 8 dengan beberapa modifikasi:

```python
# Baik
def water_plant(plant_name: str, duration: int = 3) -> bool:
    """Menyiram tanaman tertentu untuk durasi yang diberikan"""
    pass

# Buruk
def waterPlant(plantName,duration=3):
    pass
```

**Jalankan linter:**
```bash
black .  # Memformat kode
flake8 .  # Memeriksa gaya
mypy .  # Pemeriksaan tipe data
```

### Pesan Commit

Ikuti konvensi pesan commit (Conventional Commits):

```
feat: tambah mode AI agent
fix: selesaikan masalah timeout WebSocket
docs: perbarui dokumentasi API
test: tambah unit test untuk modul database
refactor: optimalkan logika pembacaan sensor
```

### Pengujian (Testing)

Tulis pengujian untuk fitur baru:

```python
# tests/test_database.py
def test_add_plant():
    result = add_plant("tomato", "Tanaman-Uji")
    assert result == True
```

Jalankan pengujian:
```bash
pytest
pytest --cov  # Dengan cakupan kode (coverage)
```

### Dokumentasi

- Perbarui `README.md` untuk perubahan yang terlihat oleh pengguna.
- Perbarui `API_DOCUMENTATION.md` untuk perubahan API.
- Tambahkan docstrings ke semua fungsi.
- Perbarui `CHANGELOG.md`.

## 🔄 Proses Pull Request

### 1. Sebelum Mengirimkan

- [ ] Kode mengikuti panduan gaya.
- [ ] Semua pengujian lulus.
- [ ] Dokumentasi telah diperbarui.
- [ ] Pesan commit jelas.
- [ ] Tidak ada konflik merge.

### 2. Kirim PR

1. Push ke fork Anda.
2. Buat Pull Request di GitHub.
3. Isi template PR.
4. Tautkan issue terkait.

### 3. Template PR

```markdown
## Deskripsi
Deskripsi singkat tentang perubahan.

## Jenis Perubahan
- [ ] Perbaikan bug
- [ ] Fitur baru
- [ ] Perubahan besar (breaking change)
- [ ] Pembaruan dokumentasi

## Pengujian
Bagaimana ini diuji?

## Daftar Periksa (Checklist)
- [ ] Kode mengikuti panduan gaya
- [ ] Pengujian ditambahkan/diperbarui
- [ ] Dokumentasi diperbarui
- [ ] Tidak ada perubahan besar (atau telah didokumentasikan)
```

## 🐛 Laporan Bug

Gunakan GitHub Issues dengan template ini:

```markdown
**Deskripsi bug**
Deskripsi yang jelas tentang bug.

**Langkah Mereproduksi**
Langkah-langkah untuk mereproduksi:
1. Jalankan perintah '...'
2. Klik pada '...'
3. Lihat kesalahan (error)

**Perilaku yang Diharapkan**
Apa yang seharusnya terjadi.

**Tangkapan Layar (Screenshots)**
Jika ada.

**Lingkungan (Environment):**
- OS: [misal Windows 11]
- Versi Python: [misal 3.11]
- Versi AgroCLI: [misal 1.1.0]

**Konteks Tambahan**
Informasi lainnya.
```

## 💡 Permintaan Fitur

Gunakan GitHub Issues dengan template ini:

```markdown
**Apakah permintaan fitur ini terkait dengan suatu masalah?**
Deskripsi masalah tersebut.

**Deskripsi solusi yang Anda inginkan**
Deskripsi yang jelas tentang fitur yang diinginkan.

**Deskripsi alternatif yang telah Anda pertimbangkan**
Solusi lain yang telah Anda pikirkan.

**Konteks Tambahan**
Mockup, contoh, dll.
```

## 🏗️ Struktur Proyek

```
agrocli/
├── core/           # Logika bisnis inti
│   ├── database.py     # Operasi database
│   ├── engine.py       # Kalkulasi tugas
│   ├── realtime.py     # Manajer WebSocket
│   ├── ai_agent.py     # Parsing perintah AI
│   └── ai_executor.py  # Eksekusi tindakan AI
├── hardware/       # Abstraksi perangkat keras
│   ├── sensors.py      # Pembacaan sensor
│   └── pump.py         # Kontrol pompa
├── web/            # Server web
│   └── server.py       # Aplikasi FastAPI
├── tests/          # Unit tests (akan ditambahkan)
├── docs/           # Dokumentasi tambahan
└── main.py         # Titik masuk CLI
```

## 🎯 Area Prioritas

Kami sangat menerima kontribusi dalam:

1. **Pengujian (Testing)** - Unit tests, integration tests.
2. **Integrasi Perangkat Keras** - Dukungan sensor nyata.
3. **Fitur AI** - Integrasi LLM, NLP yang lebih baik.
4. **Aplikasi Seluler** - React Native atau Flutter.
5. **Internasionalisasi (i18n)** - Lebih banyak bahasa.
6. **Performa** - Optimalisasi, caching.
7. **Keamanan** - Autentikasi, otorisasi.

## 📞 Komunikasi

- **GitHub Issues** - Laporan bug, permintaan fitur.
- **GitHub Discussions** - Pertanyaan, ide.
- **Pull Requests** - Kontribusi kode.

## 📜 Kode Etik

### Janji Kami
Kami berjanji untuk menjadikan partisipasi dalam proyek kami sebagai pengalaman yang bebas gangguan bagi semua orang.

### Standar Kami
**Perilaku positif:**
- Menggunakan bahasa yang menyambut baik.
- Menghormati sudut pandang yang berbeda.
- Menerima kritik membangun dengan lapang dada.
- Fokus pada apa yang terbaik bagi komunitas.

**Perilaku yang tidak dapat diterima:**
- Trolling, komentar menghina/merendahkan.
- Gangguan publik atau pribadi (harassment).
- Mempublikasikan informasi pribadi orang lain.
- Perilaku tidak etis atau tidak profesional lainnya.

## 🙏 Pengakuan
Kontributor akan:
- Dicantumkan dalam `CONTRIBUTORS.md`.
- Disebutkan dalam catatan rilis (release notes).
- Diberikan kredit dalam dokumentasi.

## ❓ Pertanyaan?
Jangan ragu untuk:
- Membuka diskusi di GitHub Discussion.
- Berkomentar pada issue terkait.
- Menghubungi para pengelola (maintainers).

---

**Terima kasih telah berkontribusi di AgroCLI! 🌱**
