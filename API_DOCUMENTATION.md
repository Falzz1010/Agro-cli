# 📡 AgroCLI API Documentation

## Base URL
```
http://localhost:8000
```

## WebSocket Endpoint

### Connect to Real-Time Updates
```
ws://localhost:8000/ws
```

**Message Types:**
```json
{
  "type": "sensor_update",
  "timestamp": "11:30:15",
  "plant_name": "Tomat-Saya",
  "moisture": 51.6,
  "temperature": 32.5,
  "humidity": 77.2
}

{
  "type": "pump_event",
  "timestamp": "11:30:20",
  "plant_name": "Tomat-Saya",
  "status": "on",
  "duration": 3
}

{
  "type": "system_alert",
  "timestamp": "11:30:25",
  "message": "Tomat-Saya moisture threshold breached",
  "level": "warning"
}
```

---

## REST API Endpoints

### 1. Get Garden Statistics
```http
GET /api/stats
```

**Response:**
```json
{
  "active_plants": 4,
  "harvested_plants": 2,
  "type_breakdown": {
    "tomato": 2,
    "chili": 2
  }
}
```

---

### 2. Get All Active Plants
```http
GET /api/plants
```

**Response:**
```json
[
  {
    "name": "Tomat-Saya",
    "plant_type": "tomato",
    "planted_date": "2026-02-20",
    "last_watered": "2026-02-28",
    "last_fertilized": "2026-02-25"
  }
]
```

---

### 3. Get Plant Details
```http
GET /api/plants/{plant_name}
```

**Response:**
```json
{
  "name": "Tomat-Saya",
  "plant_type": "tomato",
  "planted_date": "2026-02-20",
  "last_watered": "2026-02-28",
  "last_fertilized": "2026-02-25",
  "status": "active"
}
```

---

### 4. Trigger Pump (Water Plant)
```http
POST /api/water/{plant_name}
```

**Response:**
```json
{
  "status": "success",
  "message": "Pump activated for Tomat-Saya"
}
```

---

### 5. Get Telemetry Data
```http
GET /api/telemetry?limit=30
```

**Query Parameters:**
- `limit` (optional): Number of records to return (default: 30)

**Response:**
```json
[
  {
    "plant_name": "Tomat-Saya",
    "soil_moisture": 51.6,
    "temperature": 32.5,
    "humidity": 77.2,
    "timestamp": "2026-02-28 11:30:15"
  }
]
```

---

### 6. Broadcast Sensor Data (Internal)
```http
POST /api/broadcast/sensor
```

**Request Body:**
```json
{
  "plant_name": "Tomat-Saya",
  "moisture": 51.6,
  "temperature": 32.5,
  "humidity": 77.2
}
```

**Response:**
```json
{
  "status": "broadcasted"
}
```

---

### 7. Broadcast Pump Event (Internal)
```http
POST /api/broadcast/pump
```

**Request Body:**
```json
{
  "plant_name": "Tomat-Saya",
  "status": "on",
  "duration": 3
}
```

**Response:**
```json
{
  "status": "broadcasted"
}
```

---

### 8. Broadcast System Alert (Internal)
```http
POST /api/broadcast/alert
```

**Request Body:**
```json
{
  "message": "System alert message",
  "level": "info"
}
```

**Levels:** `info`, `warning`, `error`

**Response:**
```json
{
  "status": "broadcasted"
}
```

---

## Interactive API Documentation

FastAPI provides automatic interactive API documentation:

### Swagger UI
```
http://localhost:8000/docs
```

### ReDoc
```
http://localhost:8000/redoc
```

---

## Error Responses

All endpoints return standard error responses:

```json
{
  "status": "error",
  "message": "Error description"
}
```

**HTTP Status Codes:**
- `200` - Success
- `400` - Bad Request
- `404` - Not Found
- `500` - Internal Server Error

---

## Rate Limiting

Currently no rate limiting is implemented. For production use, consider adding rate limiting middleware.

---

## Authentication

Currently no authentication is required. For production use, consider adding:
- API Key authentication
- JWT tokens
- OAuth2

---

## CORS

CORS is enabled for all origins by default. For production, configure specific allowed origins in `web/server.py`.

---

## Example Usage

### Python
```python
import requests

# Get garden stats
response = requests.get("http://localhost:8000/api/stats")
print(response.json())

# Water a plant
response = requests.post("http://localhost:8000/api/water/Tomat-Saya")
print(response.json())
```

### JavaScript
```javascript
// Get garden stats
fetch('http://localhost:8000/api/stats')
  .then(response => response.json())
  .then(data => console.log(data));

// Water a plant
fetch('http://localhost:8000/api/water/Tomat-Saya', {
  method: 'POST'
})
  .then(response => response.json())
  .then(data => console.log(data));
```

### cURL
```bash
# Get garden stats
curl http://localhost:8000/api/stats

# Water a plant
curl -X POST http://localhost:8000/api/water/Tomat-Saya
```

---

## WebSocket Client Example

### JavaScript
```javascript
const ws = new WebSocket('ws://localhost:8000/ws');

ws.onopen = () => {
  console.log('Connected to AgroCLI');
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
  
  if (data.type === 'sensor_update') {
    console.log(`${data.plant_name}: ${data.moisture}% moisture`);
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected from AgroCLI');
};
```

### Python
```python
import asyncio
import websockets
import json

async def listen():
    uri = "ws://localhost:8000/ws"
    async with websockets.connect(uri) as websocket:
        while True:
            message = await websocket.recv()
            data = json.loads(message)
            print(f"Received: {data}")

asyncio.run(listen())
```

---

## Integration Examples

### Home Assistant
```yaml
sensor:
  - platform: rest
    resource: http://localhost:8000/api/telemetry?limit=1
    name: Garden Moisture
    value_template: '{{ value_json[0].soil_moisture }}'
    unit_of_measurement: '%'
```

### Node-RED
Use HTTP Request node to call API endpoints and WebSocket node for real-time updates.

### ESP32/Arduino
```cpp
#include <HTTPClient.h>

void waterPlant(String plantName) {
  HTTPClient http;
  String url = "http://192.168.1.100:8000/api/water/" + plantName;
  http.begin(url);
  int httpCode = http.POST("");
  http.end();
}
```
