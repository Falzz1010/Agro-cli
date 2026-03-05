# 📡 AgroCLI Edge API Documentation

## Base URL
```
http://localhost:8001
```

> [!IMPORTANT]
> Some endpoints require **Basic Authentication**. Use the credentials defined in your `.env` file (`ADMIN_USERNAME` and `ADMIN_PASSWORD`).

## WebSocket Endpoint

### Connect to Real-Time Updates
```
ws://localhost:8001/ws
```

**Message Format (Server → Client):**
The server broadcasts messages as JSON strings.

**1. Sensor Update**
```json
{
  "type": "SensorUpdate",
  "data": {
    "plant_name": "Tomato-1",
    "moisture": 45.2,
    "temperature": 28.5,
    "humidity": 65.3,
    "timestamp": "14:23:45",
    "min_moisture": 40.0,
    "water_ml": 200
  }
}
```

**2. AI Agent Log**
```json
{
  "type": "AiLog",
  "data": {
    "timestamp": "14:24:00",
    "query": "How is my garden?",
    "response": "Your garden looks healthy! Tomato-1 is at 45.2% moisture."
  }
}
```

---

## REST API Endpoints

### 1. Manual Watering
```http
POST /api/command/water
Content-Type: application/json
```
**Request Body:**
```json
{ "plant_name": "Tomato-1" }
```

### 2. Update Plant Settings
```http
POST /api/command/settings
Content-Type: application/json
```
**Request Body:**
```json
{
  "plant_name": "Tomato-1",
  "min_moisture": 45.0,
  "water_ml": 250
}
```

### 3. Delete Plant
```http
POST /api/command/delete
Content-Type: application/json
```
**Request Body:**
```json
{ "plant_name": "Tomato-1" }
```

### 4. Get Sensor History
```http
GET /api/history/{plant_name}?hours=24
```
**Parameters:**
- `hours` (Query): Number of hours to retrieve (default: 24).

### 5. Export Data to CSV
```http
GET /api/export/{plant_name}
```
> [!NOTE]
> This endpoint triggers a direct browser download. It requires Basic Authentication.

---

## Internal Broadcast Endpoints (Daemon/AI Agent)

These endpoints are used for high-frequency internal broadcasting. Note that the Rust engine uses internal async channels preferentially.

- `POST /api/broadcast/sensor`
- `POST /api/broadcast/ai`

---

## Security
- **Auth**: Basic Authentication is enforced on `/api/export` and potentially other sensitive management routes.
- **CORS**: Permissive CORS is enabled for development ease.

---
**AgroCLI Edge - High Performance. Real-Time. Secure.**
