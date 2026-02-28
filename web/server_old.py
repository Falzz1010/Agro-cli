from fastapi import FastAPI, BackgroundTasks, WebSocket, WebSocketDisconnect, Depends, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse
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

# Initialize FastAPI with security headers
app = FastAPI(
    title="AgroCLI Remote Brain",
    description="Secure Smart Farming IoT Platform",
    version="2.0.0"
)

# Rate limiter
limiter = Limiter(key_func=get_remote_address)
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# CORS configuration
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000", "http://localhost:8000"],  # Adjust as needed
    allow_credentials=True,
    allow_methods=["GET", "POST"],
    allow_headers=["*"],
)

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket endpoint for real-time updates"""
    await manager.connect(websocket)
    console.print("[green]WebSocket client connected[/green]")
    try:
        while True:
            # Keep connection alive and listen for client messages
            data = await websocket.receive_text()
            # Echo back or handle client commands if needed
    except WebSocketDisconnect:
        manager.disconnect(websocket)
        console.print("[dim]WebSocket client disconnected[/dim]")

@app.post("/api/broadcast/sensor")
async def broadcast_sensor(data: dict):
    """API endpoint for daemon to broadcast sensor data"""
    await manager.send_sensor_update(
        data["plant_name"],
        data["moisture"],
        data["temperature"],
        data["humidity"]
    )
    return {"status": "broadcasted"}

@app.post("/api/broadcast/pump")
async def broadcast_pump(data: dict):
    """API endpoint for daemon to broadcast pump events"""
    await manager.send_pump_event(
        data["plant_name"],
        data["status"],
        data.get("duration", 0)
    )
    return {"status": "broadcasted"}

@app.post("/api/broadcast/alert")
async def broadcast_alert(data: dict):
    """API endpoint for daemon to broadcast system alerts"""
    await manager.send_system_alert(
        data["message"],
        data.get("level", "info")
    )
    return {"status": "broadcasted"}

@app.get("/", response_class=HTMLResponse)
def read_root():
    """Serves the minimalistic Smart Farming Web Dashboard."""
    # Read stats dynamically
    stats_data = get_garden_stats()
    plants = get_all_active_plants()
    
    # Generate HTML cards for plants
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
        <title>AgroCLI Dashboard</title>
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
        <style>
            body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background-color: #121212; color: #e0e0e0; padding: 20px; }}
            h1 {{ color: #4caf50; text-align: center; margin-bottom: 5px;}}
            .subtitle {{ text-align: center; color: #888; margin-bottom: 30px; font-size: 14px;}}
            .connection-status {{ text-align: center; padding: 10px; margin-bottom: 20px; border-radius: 8px; font-weight: bold; }}
            .connection-status.connected {{ background: #1b5e20; color: #4caf50; }}
            .connection-status.disconnected {{ background: #b71c1c; color: #ef5350; }}
            .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 20px; margin-bottom: 30px; }}
            .card {{ background: #1e1e1e; padding: 20px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.3); border: 1px solid #333; position: relative; }}
            .card h3 {{ margin-top: 0; color: #81c784; border-bottom: 1px solid #333; padding-bottom: 10px; }}
            .card .live-indicator {{ position: absolute; top: 15px; right: 15px; width: 10px; height: 10px; background: #4caf50; border-radius: 50%; animation: pulse 2s infinite; }}
            @keyframes pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.3; }} }}
            button {{ background-color: #2196f3; color: white; border: none; padding: 10px 15px; border-radius: 6px; cursor: pointer; font-weight: bold; width: 100%; transition: background 0.3s; }}
            button:hover {{ background-color: #1976d2; }}
            button:disabled {{ background-color: #555; cursor: not-allowed; }}
            .stats-card {{ background: #2c3e50; border-color: #34495e; }}
            .chart-container {{ background: #1e1e1e; padding: 20px; border-radius: 12px; border: 1px solid #333; height: 400px; }}
            .sensor-reading {{ font-size: 12px; color: #aaa; margin: 10px 0; padding: 10px; background: #2a2a2a; border-radius: 6px; }}
            .alert {{ padding: 10px; margin: 10px 0; border-radius: 6px; animation: slideIn 0.3s; }}
            .alert.info {{ background: #1565c0; }}
            .alert.warning {{ background: #f57c00; }}
            .alert.error {{ background: #c62828; }}
            @keyframes slideIn {{ from {{ transform: translateX(-100%); opacity: 0; }} to {{ transform: translateX(0); opacity: 1; }} }}
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
                    setTimeout(connectWebSocket, 3000); // Reconnect after 3s
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
                    statusEl.textContent = '🟢 Real-Time Connected';
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
                
                // Add new label
                chartData.labels.push(data.timestamp);
                if (chartData.labels.length > MAX_POINTS) {{
                    chartData.labels.shift();
                }}
                
                // Update or create dataset for this plant
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
                
                // Add data point
                chartData.datasets[data.plant_name].data.push(data.moisture);
                if (chartData.datasets[data.plant_name].data.length > MAX_POINTS) {{
                    chartData.datasets[data.plant_name].data.shift();
                }}
                
                // Update chart
                chart.data.labels = chartData.labels;
                chart.data.datasets = Object.values(chartData.datasets);
                chart.update('none'); // No animation for smooth real-time
            }}
            
            function updatePumpStatus(data) {{
                const plantCard = document.getElementById('plant-' + data.plant_name);
                if (plantCard) {{
                    const button = plantCard.querySelector('button');
                    if (data.status === 'on') {{
                        button.disabled = true;
                        button.textContent = '💧 Pumping...';
                        showAlert(`Pump activated for ${{data.plant_name}} (${{data.duration}}s)`, 'info');
                    }} else {{
                        button.disabled = false;
                        button.textContent = '💧 Trigger Pump';
                        showAlert(`Pump finished for ${{data.plant_name}}`, 'info');
                    }}
                }}
            }}
            
            function showAlert(message, level) {{
                const alertContainer = document.getElementById('alertContainer');
                const alert = document.createElement('div');
                alert.className = 'alert ' + level;
                alert.textContent = message;
                alertContainer.appendChild(alert);
                
                setTimeout(() => {{
                    alert.style.opacity = '0';
                    setTimeout(() => alert.remove(), 300);
                }}, 5000);
            }}
            
            async function waterPlant(name) {{
                if(confirm('Turn on pump for ' + name + '?')) {{
                    let res = await fetch('/api/water/' + name, {{method: 'POST'}});
                    let data = await res.json();
                }}
            }}
            
            async function fetchTelemetry() {{
                const res = await fetch('/api/telemetry');
                return await res.json();
            }}

            window.onload = async function() {{
                // Connect WebSocket
                connectWebSocket();
                
                // Load historical data
                const logs = await fetchTelemetry();
                
                // Initialize chart with historical data
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
                            title: {{ display: true, text: 'Real-Time Soil Moisture', color: '#fff' }},
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
        <h1>🌾 AgroCLI Remote Brain</h1>
        <div class="subtitle">Self-Hosted Local Network Control</div>
        
        <div id="connectionStatus" class="connection-status disconnected">🔴 Connecting...</div>
        <div id="alertContainer"></div>
        
        <div class="grid">
            <div class="card stats-card">
                <h3>📊 System Status</h3>
                <p><strong>Active Plants:</strong> {stats_data['active_plants']}</p>
                <p><strong>Harvested:</strong> {stats_data['harvested_plants']}</p>
                <p><strong>Real-Time:</strong> <span style="color: #4caf50;">WebSocket Active</span></p>
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
def api_get_telemetry():
    """Returns the historical sensor logs as JSON for Chart.js"""
    return get_recent_sensor_logs(limit=30)

@app.post("/api/water/{plant_name}")
async def api_water_plant(plant_name: str, background_tasks: BackgroundTasks):
    """API Endpoint to manually trigger a pump from the web interface or an ESP32."""
    plant = get_plant(plant_name)
    if not plant:
        return {"status": "error", "message": f"Plant {plant_name} not found"}
    
    # Broadcast pump ON event
    await manager.send_pump_event(plant_name, "on", 3)
    
    # Run pump in background
    background_tasks.add_task(water_plant, plant_name, 3)
    background_tasks.add_task(update_care, plant_name, "water")
    
    # Schedule pump OFF event after duration
    async def send_pump_off():
        await asyncio.sleep(3)
        await manager.send_pump_event(plant_name, "off")
    
    background_tasks.add_task(send_pump_off)
    
    return {"status": "success", "message": f"Pump activated for {plant_name}"}

def serve():
    """Starts the FastAPI Web Dashboard."""
    console.print(Panel("[bold green]🌐 Starting AgroCLI Web Server...[/bold green]\nOpen [cyan]http://localhost:8000[/cyan] in your browser from any device on the network.", expand=False))
    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="warning")
