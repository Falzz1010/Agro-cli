"""
CLI Real-time client - Subscribe to daemon WebSocket broadcasts
"""
import asyncio
import json
from typing import Optional, Callable
import websockets
from rich.console import Console

console = Console()

class CLIRealtimeClient:
    def __init__(self, ws_url: str = "ws://localhost:8000/ws"):
        self.ws_url = ws_url
        self.ws: Optional[websockets.WebSocketClientProtocol] = None
        self.connected = False
        self.callbacks = {
            "sensor_update": [],
            "pump_event": [],
            "system_alert": []
        }
        
    def on_sensor_update(self, callback: Callable):
        """Register callback for sensor updates"""
        self.callbacks["sensor_update"].append(callback)
        
    def on_pump_event(self, callback: Callable):
        """Register callback for pump events"""
        self.callbacks["pump_event"].append(callback)
        
    def on_system_alert(self, callback: Callable):
        """Register callback for system alerts"""
        self.callbacks["system_alert"].append(callback)
    
    async def connect(self):
        """Connect to WebSocket server"""
        try:
            self.ws = await websockets.connect(self.ws_url)
            self.connected = True
            console.print("[green]✓ Connected to real-time server[/green]")
            return True
        except Exception as e:
            console.print(f"[yellow]⚠ Could not connect to real-time server: {e}[/yellow]")
            console.print("[dim]Run 'python main.py serve' in another terminal for real-time features[/dim]")
            self.connected = False
            return False
    
    async def disconnect(self):
        """Disconnect from WebSocket server"""
        if self.ws:
            await self.ws.close()
            self.connected = False
            console.print("[dim]Disconnected from real-time server[/dim]")
    
    async def listen(self):
        """Listen for messages from server"""
        if not self.ws:
            return
            
        try:
            async for message in self.ws:
                data = json.loads(message)
                msg_type = data.get("type")
                
                # Call registered callbacks
                if msg_type in self.callbacks:
                    for callback in self.callbacks[msg_type]:
                        try:
                            if asyncio.iscoroutinefunction(callback):
                                await callback(data)
                            else:
                                callback(data)
                        except Exception as e:
                            console.print(f"[red]Callback error: {e}[/red]")
                            
        except websockets.exceptions.ConnectionClosed:
            self.connected = False
            console.print("[yellow]Connection lost[/yellow]")
        except Exception as e:
            console.print(f"[red]Listen error: {e}[/red]")
    
    async def run_with_reconnect(self, main_task):
        """Run main task with auto-reconnect"""
        while True:
            if await self.connect():
                try:
                    # Run both listen and main task concurrently
                    await asyncio.gather(
                        self.listen(),
                        main_task()
                    )
                except asyncio.CancelledError:
                    break
                except Exception as e:
                    console.print(f"[red]Error: {e}[/red]")
            
            # Wait before reconnecting
            await asyncio.sleep(3)
            console.print("[dim]Attempting to reconnect...[/dim]")

# Global client instance
cli_client = CLIRealtimeClient()
