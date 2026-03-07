# 📡 Dokumentasi API AgroCLI Edge

## URL Dasar
```
http://localhost:8001
```

> [!IMPORTANT]
> Beberapa endpoint memerlukan **Basic Authentication**. Gunakan kredensial yang ditentukan dalam file `.env` Anda (`ADMIN_USERNAME` dan `ADMIN_PASSWORD`).

## Endpoint WebSocket

### Hubungkan ke Pembaruan Waktu Nyata
```
ws://localhost:8001/ws
```

**Format Pesan (Server → Klien):**
Server menyiarkan pesan sebagai string JSON.

**1. Pembaruan Sensor**
```json
{
  "type": "SensorUpdate",
  "data": {
    "plant_name": "Tomat-1",
    "moisture": 45.2,
    "temperature": 28.5,
    "humidity": 65.3,
    "timestamp": "14:23:45",
    "min_moisture": 40.0,
    "water_ml": 200
  }
}
```

**2. Log AI Agent**
```json
{
  "type": "AiLog",
  "data": {
    "timestamp": "14:24:00",
    "query": "Bagaimana kondisi kebun saya?",
    "response": "Kebun Anda terlihat sehat! Tomat-1 berada pada kelembaban 45.2%."
  }
}
```

---

## Endpoint API REST

### 1. Penyiraman Manual
```http
POST /api/command/water
Content-Type: application/json
```
**Isi Permintaan (Request Body):**
```json
{ "plant_name": "Tomat-1" }
```

### 2. Perbarui Pengaturan Tanaman
```http
POST /api/command/settings
Content-Type: application/json
```
**Isi Permintaan (Request Body):**
```json
{
  "plant_name": "Tomat-1",
  "min_moisture": 45.0,
  "water_ml": 250
}
```

### 3. Hapus Tanaman
```http
POST /api/command/delete
Content-Type: application/json
```
**Isi Permintaan (Request Body):**
```json
{ "plant_name": "Tomat-1" }
```

### 4. Dapatkan Riwayat Sensor
```http
GET /api/history/{plant_name}?hours=24
```
**Parameter:**
- `hours` (Query): Jumlah jam yang ingin diambil (default: 24).

### 5. Ekspor Data ke CSV
```http
GET /api/export/{plant_name}
```
> [!NOTE]
> Endpoint ini memicu unduhan langsung di browser. Memerlukan Basic Authentication.

---

## Endpoint Broadcast Internal (Daemon/AI Agent)

Endpoint ini digunakan untuk penyiaran internal frekuensi tinggi. Perhatikan bahwa mesin Rust lebih mengutamakan penggunaan saluran asinkron internal.

- `POST /api/broadcast/sensor`
- `POST /api/broadcast/ai`

---

## Keamanan
- **Auth**: Basic Authentication diterapkan pada `/api/export` dan rute manajemen sensitif lainnya.
- **CORS**: CORS permisif diaktifkan untuk kemudahan pengembangan.

---
**AgroCLI Edge - Performa Tinggi. Waktu Nyata. Aman.**
