"""
Real-time WebSocket manager for broadcasting sensor data and system events
"""
import asyncio
import json
from typing import Set
from fastapi import WebSocket
from datetime import datetime

class ConnectionManager:
    def __init__(self):
        self.active_connections: Set[WebSocket] = set()
        
    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.add(websocket)
        
    def disconnect(self, websocket: WebSocket):
        self.active_connections.discard(websocket)
        
    async def broadcast(self, message: dict):
        """Broadcast message to all connected clients"""
        disconnected = set()
        # Create a copy of the set to avoid "set changed size during iteration" error
        connections_copy = self.active_connections.copy()
        
        for connection in connections_copy:
            try:
                await connection.send_json(message)
            except Exception:
                disconnected.add(connection)
        
        # Clean up disconnected clients
        for conn in disconnected:
            self.disconnect(conn)
    
    async def send_sensor_update(self, plant_name: str, moisture: float, temp: float, humidity: float):
        """Send sensor reading update"""
        await self.broadcast({
            "type": "sensor_update",
            "timestamp": datetime.now().strftime("%H:%M:%S"),
            "plant_name": plant_name,
            "moisture": moisture,
            "temperature": temp,
            "humidity": humidity
        })
    
    async def send_pump_event(self, plant_name: str, status: str, duration: int = 0):
        """Send pump activation/deactivation event"""
        await self.broadcast({
            "type": "pump_event",
            "timestamp": datetime.now().strftime("%H:%M:%S"),
            "plant_name": plant_name,
            "status": status,  # "on" or "off"
            "duration": duration
        })
    
    async def send_system_alert(self, message: str, level: str = "info"):
        """Send system alert (info, warning, error)"""
        await self.broadcast({
            "type": "system_alert",
            "timestamp": datetime.now().strftime("%H:%M:%S"),
            "message": message,
            "level": level
        })

# Global manager instance
manager = ConnectionManager()
