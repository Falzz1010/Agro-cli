"""
WebSocket Event Manager for Real-Time Communication
Broadcasts sensor readings, pump actions, and system events to all connected clients.
"""
import json
import asyncio
from typing import Set, Dict, Any
from fastapi import WebSocket

class ConnectionManager:
    def __init__(self):
        self.active_connections: Set[WebSocket] = set()
        
    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.add(websocket)
        
    def disconnect(self, websocket: WebSocket):
        self.active_connections.discard(websocket)
        
    async def broadcast(self, message: Dict[str, Any]):
        """Broadcast message to all connected clients"""
        if not self.active_connections:
            return
            
        message_json = json.dumps(message)
        disconnected = set()
        
        for connection in self.active_connections:
            try:
                await connection.send_text(message_json)
            except Exception:
                disconnected.add(connection)
        
        # Clean up disconnected clients
        for conn in disconnected:
            self.disconnect(conn)
    
    def broadcast_sync(self, message: Dict[str, Any]):
        """Synchronous wrapper for broadcasting from non-async code"""
        try:
            loop = asyncio.get_event_loop()
            if loop.is_running():
                asyncio.create_task(self.broadcast(message))
            else:
                loop.run_until_complete(self.broadcast(message))
        except RuntimeError:
            # No event loop in current thread (daemon mode)
            pass

# Global instance
manager = ConnectionManager()

def emit_sensor_reading(plant_name: str, moisture: float, temp: float, humidity: float):
    """Emit sensor reading event"""
    manager.broadcast_sync({
        "type": "sensor_reading",
        "data": {
            "plant_name": plant_name,
            "moisture": moisture,
            "temperature": temp,
            "humidity": humidity
        }
    })

def emit_pump_action(plant_name: str, duration: int, status: str):
    """Emit pump action event (on/off)"""
    manager.broadcast_sync({
        "type": "pump_action",
        "data": {
            "plant_name": plant_name,
            "duration": duration,
            "status": status
        }
    })

def emit_system_alert(message: str, level: str = "info"):
    """Emit system alert/notification"""
    manager.broadcast_sync({
        "type": "system_alert",
        "data": {
            "message": message,
            "level": level
        }
    })
