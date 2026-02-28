"""
Secure Web Server with Authentication, Rate Limiting, and Input Validation
"""
from fastapi import FastAPI, BackgroundTasks, WebSocket, WebSocketDisconnect, Depends, Request, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.security import HTTPBasic, HTTPBasicCredentials
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address
from slowapi.errors import RateLimitExceeded
import uvicorn
import os
from dotenv import load_dotenv

from core.database import get_garden_stats, get_all_active_plants, get_plant, update_care, get_recent_sensor_logs
from hardware.pump import water_plant
from core.realtime import manager
from core.security import verify_credentials, log_security_event, validate_plant_name
from rich.console import Console
from rich.panel import Panel
import asyncio

# Load environment variables
load_dotenv()

console = Console()

# Initialize FastAPI
app = FastAPI(
    title="AgroCLI Secure Platform",
    description="Secure Smart Farming IoT Platform with Authentication",
    version="2.0.0-secure"
)

# Rate limiter
limiter = Limiter(key_func=get_remote_address)
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# CORS configuration
allowed_origins = os.getenv("ALLOWED_ORIGINS", "http://localhost:3000,http://localhost:8000").split(",")
app.add_middleware(
    CORSMiddleware,
    allow_origins=allowed_origins,
    allow_credentials=True,
    allow_methods=["GET", "POST"],
    allow_headers=["*"],
)

# Security headers middleware
@app.middleware("http")
async def add_security_headers(request: Request, call_next):
    response = await call_next(request)
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-Frame-Options"] = "DENY"
    response.headers["X-XSS-Protection"] = "1; mode=block"
    response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
    return response

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket endpoint for real-time updates"""
    await manager.connect(websocket)
    client_ip = websocket.client.host if websocket.client else "unknown"
    log_security_event("websocket_connect", "Client connected", ip=client_ip)
    console.print(f"[green]WebSocket client connected from {client_ip}[/green]")
    
    try:
        while True:
            data = await websocket.receive_text()
    except WebSocketDisconnect:
        manager.disconnect(websocket)
        log_security_event("websocket_disconnect", "Client disconnected", ip=client_ip)
        console.print(f"[dim]WebSocket client disconnected from {client_ip}[/dim]")

@app.post("/api/broadcast/sensor")
@limiter.limit(os.getenv("RATE_LIMIT_API", "100/minute"))
async def broadcast_sensor(request: Request, data: dict):
    """Internal API for daemon to broadcast sensor data"""
    try:
        await manager.send_sensor_update(
            validate_plant_name(data["plant_name"]),
            float(data["moisture"]),
            float(data["temperature"]),
            float(data["humidity"])
        )
        return {"status": "broadcasted"}
    except Exception as e:
        return {"status": "error", "message": str(e)}

@app.post("/api/broadcast/pump")
@limiter.limit(os.getenv("RATE_LIMIT_API", "100/minute"))
async def broadcast_pump(request: Request, data: dict):
    """Internal API for daemon to broadcast pump events"""
    await manager.send_pump_event(
        data["plant_name"],
        data["status"],
        data.get("duration", 0)
    )
    return {"status": "broadcasted"}

@app.post("/api/broadcast/alert")
@limiter.limit(os.getenv("RATE_LIMIT_API", "100/minute"))
async def broadcast_alert(request: Request, data: dict):
    """Internal API for daemon to broadcast alerts"""
    await manager.send_system_alert(
        data["message"],
        data.get("level", "info")
    )
    return {"status": "broadcasted"}

@app.get("/", response_class=HTMLResponse)
async def read_root(request: Request, username: str = Depends(verify_credentials)):
    """Secure dashboard - requires authentication"""
    log_security_event("dashboard_access", f"User accessed dashboard", 
                      ip=request.client.host, user=username)
    
    stats_data = get_garden_stats()
    plants = get_all_active_plants()
    
    plant_cards = ""
    for p in plants:
        name = p["name"]
        plant_cards += f"""
        <div class="card" id="plant-{name}">
            <div class="live-indicator"></div>
            <h3>🌱 {name} ({p["plant_type"]})</h3>
            <p>Last Watered: {p["last_watered"]}</p>
            <div class="sensor-reading">Waiting for sensor data...</div>
            <button onclick="waterPlant('{name}')">💧 Trigger Pump</button>
        </div>
        """

    html_content = f"""
    <!DOCTYPE html>
    <html>
    <head>
        <title>AgroCLI Secure Dashboard</title>
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline' cdn.jsdelivr.net; style-src 'self' 'unsafe-inline';">
        <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
        <style>
            body {{ font-family: 'Segoe UI', sans-serif; background: #121212; color: #e0e0e0; padding: 20px; margin: 0; }}
            .header {{ background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%); padding: 20px; border-radius: 12px; margin-bottom: 20px; }}
            .header h1 {{ margin: 0; color: #4caf50; }}
            .header .user-info {{ color: #aaa; font-size: 14px; margin-top: 5px; }}
            .security-badge {{ display: inline-block; padding: 5px 10px; background: #2e7d32; border-radius: 4px; font-size: 12px; margin-left: 10px; }}
            .connection-status {{ text-align: center; padding: 10px; margin-bottom: 20px; border-radius: 8px; font-weight: bold; }}
            .connection-status.connected {{ background: #1b5e20; color: #4caf50; }}
            .connection-status.disconnected {{ background: #b71c1c; color: #ef5350; }}
            .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 20px; margin-bottom: 30px; }}
            .card {{ background: #1e1e1e; padding: 20px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.3); border: 1px solid #333; position: relative; }}
            .card h3 {{ margin-top: 0; color: #81c784; border-bottom: 1px solid #333; padding-bottom: 10px; }}
            .card .live-indicator {{ position: absolute; top: 15px; right: 15px; width: 10px; height: 10px; background: #4caf50; border-radius: 50%; animation: pulse 2s infinite; }}
            @keyframes pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.3; }} }}
            button {{ background: #2196f3; color: white; border: none; padding: 10px 15px; border-radius: 6px; cursor: pointer; font-weight: bold; width: 100%; transition: 0.3s; }}
            button:hover {{ background: #1976d2; }}
            button:disabled {{ background: #555; cursor: not-allowed; }}
            .stats-card {{ background: #2c3e50; border-color: #34495e; }}
            .chart-container {{ background: #1e1e1e; padding: 20px; border-radius: 12px; border: 1px solid #333; height: 400px; }}
            .sensor-reading {{ font-size: 12px; color: #aaa; margin: 10px 0; padding: 10px; background: #2a2a2a; border-radius: 6px; }}
            .logout-btn {{ position: fixed; top: 20px; right: 20px; background: #f44336; padding: 10px 20px; border-radius: 6px; cursor: pointer; }}
        </style>
        <script>
            let ws;
            let chart;
            let chartData = {{ labels: [], datasets: {{}} }};
            const MAX_POINTS = 30;
            
            function connectWebSocket() {{
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                ws = new WebSocket(protocol + '//' + window.location.host + '/ws');
                
                ws.onopen = () => {{
                    console.log('WebSocket connected');
                    updateConnectionStatus(true);
                }};
                
                ws.onclose = () => {{
                    console.log('WebSocket disconnected');
                    updateConnectionStatus(false);
                    setTimeout(connectWebSocket, 3000);
                }};
                
                ws.onerror = (error) => {{
                    console.error('WebSocket error:', error);
                }};
                
                ws.onmessage = (event) => {{
                    const message = JSON.parse(event.data);
                    handleWebSocketMessage(message);
                }};
            }}
            
            function updateConnectionStatus(connected) {{
                const statusEl = document.getElementById('connectionStatus');
                if (connected) {{
                    statusEl.className = 'connection-status connected';
                    statusEl.textContent = '🟢 Real-Time Connected (Secure)';
                }} else {{
                    statusEl.className = 'connection-status disconnected';
                    statusEl.textContent = '🔴 Disconnected - Reconnecting...';
                }}
            }}
            
            function handleWebSocketMessage(message) {{
                console.log('Received:', message);
                
                switch(message.type) {{
                    case 'sensor_update':
                        updateSensorDisplay(message);
                        updateChart(message);
                        break;
                    case 'pump_event':
                        updatePumpStatus(message);
                        break;
                    case 'system_alert':
                        showAlert(message.message, message.level);
                        break;
                }}
            }}
            
            function updateSensorDisplay(data) {{
                const plantCard = document.getElementById('plant-' + data.plant_name);
                if (plantCard) {{
                    const sensorDiv = plantCard.querySelector('.sensor-reading');
                    if (sensorDiv) {{
                        sensorDiv.innerHTML = `
                            <small style="color: #888;">Live: ${{data.timestamp}}</small><br>
                            💧 Moisture: <strong>${{data.moisture.toFixed(1)}}%</strong><br>
                            🌡️ Temp: ${{data.temperature.toFixed(1)}}°C | 💨 Humidity: ${{data.humidity.toFixed(1)}}%
                        `;
                    }}
                }}
            }}
            
            function updateChart(data) {{
                if (!chart) return;
                
                chartData.labels.push(data.timestamp);
                if (chartData.labels.length > MAX_POINTS) {{
                    chartData.labels.shift();
                }}
                
                if (!chartData.datasets[data.plant_name]) {{
                    const colors = ['#4caf50', '#2196f3', '#ff9800', '#f44336', '#9c27b0'];
                    const colorIdx = Object.keys(chartData.datasets).length % colors.length;
                    
                    chartData.datasets[data.plant_name] = {{
                        label: data.plant_name + ' Moisture %',
                        data: [],
                        borderColor: colors[colorIdx],
                        backgroundColor: colors[colorIdx],
                        tension: 0.4
                    }};
                }}
                
                chartData.datasets[data.plant_name].data.push(data.moisture);
                if (chartData.datasets[data.plant_name].data.length > MAX_POINTS) {{
                    chartData.datasets[data.plant_name].data.shift();
                }}
                
                chart.data.labels = chartData.labels;
                chart.data.datasets = Object.values(chartData.datasets);
                chart.update('none');
            }}
            
            function updatePumpStatus(data) {{
                const plantCard = document.getElementById('plant-' + data.plant_name);
                if (plantCard) {{
                    const button = plantCard.querySelector('button');
                    if (data.status === 'on') {{
                        button.disabled = true;
                        button.textContent = '💧 Pumping...';
                    }} else {{
                        button.disabled = false;
                        button.textContent = '💧 Trigger Pump';
                    }}
                }}
            }}
            
            function showAlert(message, level) {{
                alert(message);
            }}
            
            async function waterPlant(name) {{
                if(confirm('Turn on pump for ' + name + '?')) {{
                    try {{
                        let res = await fetch('/api/water/' + name, {{method: 'POST'}});
                        let data = await res.json();
                        alert(data.message || 'Pump activated');
                    }} catch(e) {{
                        alert('Error: ' + e.message);
                    }}
                }}
            }}
            
            async function fetchTelemetry() {{
                const res = await fetch('/api/telemetry');
                return await res.json();
            }}

            window.onload = async function() {{
                connectWebSocket();
                
                const logs = await fetchTelemetry();
                const plantDataSets = {{}};
                const labels = [];
                
                logs.forEach(log => {{
                    if(!labels.includes(log.timestamp)) labels.push(log.timestamp);
                    if(!plantDataSets[log.plant_name]) {{
                        plantDataSets[log.plant_name] = [];
                    }}
                }});
                
                const colors = ['#4caf50', '#2196f3', '#ff9800', '#f44336'];
                let colorIdx = 0;
                
                const datasets = Object.keys(plantDataSets).map(p_name => {{
                    const dataPoints = labels.map(lbl => {{
                        const match = logs.find(l => l.timestamp === lbl && l.plant_name === p_name);
                        return match ? match.soil_moisture : null;
                    }});
                    
                    let bgCol = colors[colorIdx % colors.length];
                    colorIdx++;
                    
                    chartData.datasets[p_name] = {{
                        label: p_name + ' Moisture %',
                        data: dataPoints,
                        borderColor: bgCol,
                        backgroundColor: bgCol,
                        tension: 0.4,
                        spanGaps: true
                    }};
                    
                    return chartData.datasets[p_name];
                }});
                
                chartData.labels = labels;

                const ctx = document.getElementById('moistureChart').getContext('2d');
                chart = new Chart(ctx, {{
                    type: 'line',
                    data: {{
                        labels: chartData.labels,
                        datasets: datasets
                    }},
                    options: {{
                        responsive: true,
                        maintainAspectRatio: false,
                        animation: false,
                        plugins: {{
                            title: {{ display: true, text: 'Real-Time Soil Moisture (Secure)', color: '#fff' }},
                            legend: {{ labels: {{ color: '#fff' }} }}
                        }},
                        scales: {{
                            x: {{ ticks: {{ color: '#888' }} }},
                            y: {{ min: 0, max: 100, ticks: {{ color: '#888' }} }}
                        }}
                    }}
                }});
            }};
        </script>
    </head>
    <body>
        <div class="header">
            <h1>🔒 AgroCLI Secure Dashboard <span class="security-badge">🛡️ Protected</span></h1>
            <div class="user-info">Logged in as: {username} | Session: Active</div>
        </div>
        
        <div id="connectionStatus" class="connection-status disconnected">🔴 Connecting...</div>
        
        <div class="grid">
            <div class="card stats-card">
                <h3>📊 System Status</h3>
                <p><strong>Active Plants:</strong> {stats_data['active_plants']}</p>
                <p><strong>Harvested:</strong> {stats_data['harvested_plants']}</p>
                <p><strong>Security:</strong> <span style="color: #4caf50;">✓ Authenticated</span></p>
                <p><strong>Real-Time:</strong> <span style="color: #4caf50;">✓ WebSocket Active</span></p>
            </div>
            {plant_cards}
        </div>
        
        <div class="chart-container">
            <canvas id="moistureChart"></canvas>
        </div>
    </body>
    </html>
    """
    return HTMLResponse(content=html_content)

@app.get("/api/telemetry")
async def api_get_telemetry(username: str = Depends(verify_credentials)):
    """Get historical sensor data - requires authentication"""
    return get_recent_sensor_logs(limit=30)

@app.post("/api/water/{plant_name}")
@limiter.limit(os.getenv("RATE_LIMIT_PUMP", "5/minute"))
async def api_water_plant(
    request: Request,
    plant_name: str,
    background_tasks: BackgroundTasks,
    username: str = Depends(verify_credentials)
):
    """Trigger pump - requires authentication and rate limited"""
    plant_name = validate_plant_name(plant_name)
    plant = get_plant(plant_name)
    
    if not plant:
        log_security_event("pump_trigger_failed", f"Plant not found: {plant_name}", 
                          ip=request.client.host, user=username)
        raise HTTPException(status_code=404, detail=f"Plant {plant_name} not found")
    
    log_security_event("pump_trigger", f"Pump activated for {plant_name}", 
                      ip=request.client.host, user=username)
    
    await manager.send_pump_event(plant_name, "on", 3)
    background_tasks.add_task(water_plant, plant_name, 3)
    background_tasks.add_task(update_care, plant_name, "water")
    
    async def send_pump_off():
        await asyncio.sleep(3)
        await manager.send_pump_event(plant_name, "off")
    
    background_tasks.add_task(send_pump_off)
    
    return {"status": "success", "message": f"Pump activated for {plant_name}"}

def serve():
    """Start secure web server"""
    console.print(Panel(
        "[bold green]🔒 Starting AgroCLI Secure Web Server...[/bold green]\n"
        "Open [cyan]http://localhost:8000[/cyan] in your browser\n"
        "[yellow]⚠ Authentication required![/yellow]\n"
        f"Username: {os.getenv('ADMIN_USERNAME', 'admin')}\n"
        "Password: (from .env file)",
        expand=False
    ))
    
    enable_https = os.getenv("ENABLE_HTTPS", "false").lower() == "true"
    
    if enable_https:
        uvicorn.run(
            app,
            host=os.getenv("HOST", "0.0.0.0"),
            port=int(os.getenv("PORT", 8000)),
            ssl_keyfile=os.getenv("SSL_KEYFILE", "key.pem"),
            ssl_certfile=os.getenv("SSL_CERTFILE", "cert.pem"),
            log_level="warning"
        )
    else:
        uvicorn.run(
            app,
            host=os.getenv("HOST", "0.0.0.0"),
            port=int(os.getenv("PORT", 8000)),
            log_level="warning"
        )
