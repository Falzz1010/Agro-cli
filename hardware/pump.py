import time
from rich.console import Console

console = Console()

def water_plant(plant_name: str, duration_seconds: int = 2):
    # Import here to avoid circular dependency
    try:
        from core.events import emit_pump_action
        emit_pump_action(plant_name, duration_seconds, "on")
    except ImportError:
        pass
    
    console.print(f"[bold blue]💧 [IOT MOCK] {plant_name}: Pump ON[/bold blue]")
    time.sleep(duration_seconds)
    console.print(f"[bold cyan]💧 [IOT MOCK] {plant_name}: Pump OFF[/bold cyan]")
    
    try:
        from core.events import emit_pump_action
        emit_pump_action(plant_name, duration_seconds, "off")
    except ImportError:
        pass
