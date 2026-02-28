import argparse
import sys
import os
import time
import asyncio
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.align import Align
from rich.text import Text
from rich.live import Live
from rich.layout import Layout
import questionary
from types import SimpleNamespace

# Import refactored modules
from core.database import (init_db, add_plant, get_all_active_plants, 
                           update_care, harvest_plant, get_garden_stats, 
                           export_stats_to_csv, log_sensor_data,
                           load_config, save_config)
from core.weather import get_weather
from core.engine import load_rules, calculate_today_tasks
from hardware.sensors import read_soil_moisture, read_temperature, read_humidity
from hardware.pump import water_plant

if sys.stdout.encoding != 'utf-8':
    sys.stdout.reconfigure(encoding='utf-8')

VERSION = "1.0.0"
console = Console()

# ==========================================
# CLI APPLICATION & MENUS
# ==========================================

def display_banner():
    banner_text = r"""
     █████╗  ██████╗ ██████╗  ██████╗  ██████╗██╗     ██╗
    ██╔══██╗██╔════╝ ██╔══██╗██╔═══██╗██╔════╝██║     ██║
    ███████║██║  ███╗██████╔╝██║   ██║██║     ██║     ██║
    ██╔══██║██║   ██║██╔══██╗██║   ██║██║     ██║     ██║
    ██║  ██║╚██████╔╝██║  ██║╚██████╔╝╚██████╗███████╗██║
    ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚══════╝╚═╝
"""
    # Build text dynamically
    text = Text(banner_text, style="bold bright_green", justify="center")
    
    text.append("=== THE INTELLIGENT GARDEN BRAIN ===\n", style="bold white")
    text.append(f"v{VERSION} | Local Network Node Active\n", style="dim green")
    text.append("Made with 💚 by Naufal Rizky\n", style="italic cyan")
    
    # Wrap perfectly in a modern borderless Panel
    panel = Panel(
        text,
        border_style="green",
        expand=False,
        padding=(1, 5)
    )
    console.print(Align.center(panel))
    console.print()

def init():
    init_db()
    console.print(Panel("[bold green]🌱 AgroCLI Initialized![/bold green]\nYour local tracking database is ready.\nRun [cyan]python main.py add <plant_type> <name>[/cyan] to start planting.", expand=False))

def add(args):
    rules = load_rules()
    plant_type = args.type.lower()
    if plant_type not in rules:
        console.print(f"[bold red]Error:[/bold red] Plant type '{plant_type}' is not recognized in plants.yaml.")
        console.print(f"Available types: {', '.join(rules.keys())}")
        sys.exit(1)
        
    success = add_plant(plant_type, args.name)
    if success:
        console.print(f"[bold green]Success![/bold green] Added {args.name} ({plant_type}) to your garden.")
    else:
        console.print(f"[bold red]Error:[/bold red] A plant named '{args.name}' already exists.")

def today(args):
    config = load_config()
    city = args.city or config.get("city")
    api_key = args.api_key or config.get("api_key")
    
    weather_cond = None
    if city and api_key:
        weather_cond = get_weather(city, api_key)
        if weather_cond:
            console.print(f"🌦️  Current Weather in {city}: [bold blue]{weather_cond}[/bold blue]")
        else:
            console.print(f"⚠️  Could not fetch weather for {city}.")
    
    active_plants = get_all_active_plants()
    if not active_plants:
        console.print("Your garden is empty! Add a plant first.")
        return []
        
    tasks = calculate_today_tasks(active_plants, weather_condition=weather_cond)
    
    if not tasks:
        console.print(Panel("[bold green]All caught up![/bold green] No tasks needed today for your garden.", expand=False))
        return []

    table = Table(title="🌱 Today's Garden Tasks")
    table.add_column("Plant", style="cyan", no_wrap=True)
    table.add_column("Type", style="magenta")
    table.add_column("Water", justify="center")
    table.add_column("Fertilize", justify="center")
    
    needs_action = []

    for task in tasks:
        water_str = ""
        if task["needs_water"]:
            if task["skip_watering_due_to_weather"]:
                water_str = "[yellow]Skip (Rain)[/yellow]"
            else:
                water_str = f"[blue]💧 {task['water_ml']}ml[/blue]"
                needs_action.append((task["name"], "water"))
        else:
            water_str = "[dim]OK[/dim]"
            
        fert_str = "[green]🌾 Yes[/green]" if task["needs_fertilizer"] else "[dim]OK[/dim]"
        if task["needs_fertilizer"]:
             needs_action.append((task["name"], "fertilizer"))

        if task["needs_water"] or task["needs_fertilizer"]:
            table.add_row(task["name"], task["plant_type"], water_str, fert_str)

    if not args.mark_done:
        console.print(table)
    
    if args.mark_done and needs_action:
        for name, care_type in needs_action:
            update_care(name, care_type)
        console.print("[bold green]✅ All listed tasks have been marked as completed in the database![/bold green]")
    elif needs_action and not hasattr(args, "is_interactive"):
        console.print("Run [cyan]python main.py today --mark-done[/cyan] after you finish these tasks.")
        
    return needs_action

def harvest(args):
    success = harvest_plant(args.name)
    if success:
         console.print(f"[bold green]🎉 Harvested![/bold green] {args.name} has been archived.")
    else:
         console.print(f"[bold red]Error:[/bold red] Could not find an active plant named '{args.name}'.")

def stats(args):
    stats_data = get_garden_stats()
    console.print(f"[bold green]Total Active Plants:[/bold green] {stats_data['active_plants']}")
    console.print(f"[bold yellow]Total Harvested/Archived:[/bold yellow] {stats_data['harvested_plants']}")
    
    if stats_data['type_breakdown']:
        table = Table(title="Plant Types Distribution")
        table.add_column("Type", style="cyan")
        table.add_column("Count", style="magenta")
        for p_type, count in stats_data['type_breakdown'].items():
            table.add_row(p_type, str(count))
        console.print(table)
        
    if args.export:
        export_stats_to_csv(args.export)
        console.print(f"[bold green]✅ Data exported to [cyan]{args.export}[/cyan][/bold green]")

def interactive_mode():
    display_banner()
    
    if not os.path.exists("data/garden.db"):
        console.print("[yellow]Welcome to AgroCLI! Let's initialize your garden first.[/yellow]")
        init()
        
    while True:
        choice = questionary.select(
            "What would you like to do?",
            choices=[
                "🌱 Check Today's Tasks (Real-Time)",
                "➕ Add New Plant",
                "📊 View Garden Stats (Real-Time)",
                "📡 Live Sensor Monitor",
                "✂️  Harvest a Plant",
                "☁️  Configure Weather API",
                "🔌 Run Daemon Automation",
                "🌐 Start Web Dashboard",
                "❌ Exit"
            ],
            style=questionary.Style([('selected', 'fg:green bold')])
        ).ask()
        
        if choice is None or choice == "❌ Exit":
            console.print("[bold green]Happy Farming! Goodbye. 👋[/bold green]")
            break
            
        elif choice == "🌱 Check Today's Tasks (Real-Time)":
            live_task_monitor()
                
        elif choice == "➕ Add New Plant":
            rules = load_rules()
            if not rules:
                console.print("[red]No plants loaded in plants.yaml[/red]")
                continue
            plant_type = questionary.select("Select plant type:", choices=list(rules.keys())).ask()
            if not plant_type:
                continue
            name = questionary.text("Give your plant a unique nickname:").ask()
            if not name:
                continue
            add(SimpleNamespace(type=plant_type, name=name))
            
        elif choice == "📊 View Garden Stats (Real-Time)":
            live_stats_monitor()
                    
        elif choice == "📡 Live Sensor Monitor":
            live_sensor_monitor()
                    
        elif choice == "✂️  Harvest a Plant":
            active_plants = get_all_active_plants()
            if not active_plants:
                console.print("[yellow]No active plants to harvest.[/yellow]")
                continue
            choices = [p["name"] for p in active_plants]
            choices.append("Cancel")
            to_harvest = questionary.select("Which plant do you want to harvest?", choices=choices).ask()
            if to_harvest and to_harvest != "Cancel":
                harvest(SimpleNamespace(name=to_harvest))
                
        elif choice == "☁️  Configure Weather API":
            console.print("[dim]Get a free API key at openweathermap.org[/dim]")
            city = questionary.text("Enter your City:", default="Surabaya").ask()
            if city:
                save_config("city", city)
            api_key = questionary.text("Enter your OpenWeatherMap API Key (Leave blank to use default):").ask()
            if api_key:
                save_config("api_key", api_key)
            console.print("[bold green]Weather configured successfully![/bold green]")
            
        elif choice == "🔌 Run Daemon Automation":
            daemon_mode()
            
        elif choice == "🌐 Start Web Dashboard":
            # Direct import of server here to avoid early FastAPI loading for CLI-only usage
            from web.server import serve
            serve()
                
        console.print("\n" + "-"*50 + "\n")

def live_task_monitor():
    """Real-time task monitoring with auto-refresh"""
    from rich.live import Live
    from rich.table import Table
    from rich.panel import Panel
    
    console.print(Panel("[bold green]📋 Live Task Monitor[/bold green]\nPress Ctrl+C to exit", expand=False))
    
    config = load_config()
    city = config.get("city")
    api_key = config.get("api_key")
    
    try:
        with Live(console=console, refresh_per_second=0.5) as live:
            while True:
                # Get weather
                weather_cond = None
                if city and api_key:
                    weather_cond = get_weather(city, api_key)
                
                # Create header
                header = f"[bold cyan]Live Task Monitor[/bold cyan] | {time.strftime('%H:%M:%S')}"
                if weather_cond:
                    header += f" | 🌦️  {city}: [blue]{weather_cond}[/blue]"
                
                # Get tasks
                active_plants = get_all_active_plants()
                if not active_plants:
                    live.update(Panel("[yellow]No active plants. Add plants first![/yellow]", title=header))
                    time.sleep(2)
                    continue
                
                tasks = calculate_today_tasks(active_plants, weather_condition=weather_cond)
                
                if not tasks:
                    live.update(Panel("[bold green]✓ All caught up![/bold green] No tasks needed today.", title=header))
                    time.sleep(2)
                    continue
                
                # Create tasks table
                table = Table(show_header=True, header_style="bold magenta", title=header)
                table.add_column("Plant", style="cyan", no_wrap=True)
                table.add_column("Type", style="green")
                table.add_column("Water", justify="center")
                table.add_column("Fertilize", justify="center")
                table.add_column("Status", justify="center")
                
                for task in tasks:
                    water_str = ""
                    if task["needs_water"]:
                        if task["skip_watering_due_to_weather"]:
                            water_str = "[yellow]Skip (Rain)[/yellow]"
                        else:
                            water_str = f"[blue]💧 {task['water_ml']}ml[/blue]"
                    else:
                        water_str = "[dim]OK[/dim]"
                    
                    fert_str = "[green]🌾 Yes[/green]" if task["needs_fertilizer"] else "[dim]OK[/dim]"
                    
                    # Status indicator
                    if task["needs_water"] or task["needs_fertilizer"]:
                        status = "[red]⚠️  Action[/red]"
                    else:
                        status = "[green]✓ OK[/green]"
                    
                    table.add_row(
                        task["name"],
                        task["plant_type"],
                        water_str,
                        fert_str,
                        status
                    )
                
                live.update(table)
                time.sleep(2)
                
    except KeyboardInterrupt:
        console.print("\n[bold green]✓ Task monitor stopped[/bold green]")
        
        # Ask if want to mark tasks as done
        if tasks:
            mark = questionary.confirm("Mark all tasks as completed?").ask()
            if mark:
                needs_action = []
                for task in tasks:
                    if task["needs_water"]:
                        needs_action.append((task["name"], "water"))
                    if task["needs_fertilizer"]:
                        needs_action.append((task["name"], "fertilizer"))
                
                for name, care_type in needs_action:
                    update_care(name, care_type)
                console.print("[bold green]✅ All tasks marked as completed![/bold green]")

def live_stats_monitor():
    """Real-time stats monitoring in terminal"""
    from rich.live import Live
    from rich.table import Table
    from rich.panel import Panel
    from rich.layout import Layout
    
    console.print(Panel("[bold green]📊 Real-Time Stats Monitor[/bold green]\nPress Ctrl+C to exit", expand=False))
    
    try:
        with Live(console=console, refresh_per_second=1) as live:
            while True:
                # Create layout
                layout = Layout()
                layout.split_column(
                    Layout(name="header", size=3),
                    Layout(name="stats", size=10),
                    Layout(name="plants")
                )
                
                # Header
                header_text = f"[bold green]AgroCLI Live Stats[/bold green] | {time.strftime('%H:%M:%S')}"
                layout["header"].update(Panel(header_text, style="green"))
                
                # Stats
                stats_data = get_garden_stats()
                stats_table = Table(show_header=False, box=None)
                stats_table.add_row("🌱 Active Plants:", f"[green]{stats_data['active_plants']}[/green]")
                stats_table.add_row("🎉 Harvested:", f"[yellow]{stats_data['harvested_plants']}[/yellow]")
                stats_table.add_row("📊 Total:", f"[cyan]{stats_data['active_plants'] + stats_data['harvested_plants']}[/cyan]")
                layout["stats"].update(Panel(stats_table, title="Statistics", border_style="blue"))
                
                # Plants table
                plants = get_all_active_plants()
                plants_table = Table(title="Active Plants", show_header=True, header_style="bold magenta")
                plants_table.add_column("Name", style="cyan")
                plants_table.add_column("Type", style="green")
                plants_table.add_column("Planted", style="yellow")
                plants_table.add_column("Last Watered", style="blue")
                
                for plant in plants:
                    plants_table.add_row(
                        plant["name"],
                        plant["plant_type"],
                        plant["planted_date"],
                        plant["last_watered"]
                    )
                
                layout["plants"].update(plants_table)
                
                live.update(layout)
                time.sleep(1)
                
    except KeyboardInterrupt:
        console.print("\n[bold green]✓ Stats monitor stopped[/bold green]")

def live_sensor_monitor():
    """Real-time sensor monitoring in terminal with WebSocket"""
    from rich.live import Live
    from rich.table import Table
    from rich.panel import Panel
    from core.cli_realtime import cli_client
    
    console.print(Panel("[bold green]📡 Live Sensor Monitor[/bold green]\nConnecting to real-time server...", expand=False))
    
    active_plants = get_all_active_plants()
    if not active_plants:
        console.print("[yellow]No active plants to monitor.[/yellow]")
        return
    
    # Store latest sensor data
    sensor_data = {}
    for plant in active_plants:
        sensor_data[plant["name"]] = {
            "moisture": 0.0,
            "temperature": 0.0,
            "humidity": 0.0,
            "timestamp": "Waiting..."
        }
    
    # Callback for sensor updates
    def on_sensor_update(data):
        plant_name = data.get("plant_name")
        if plant_name in sensor_data:
            sensor_data[plant_name] = {
                "moisture": data.get("moisture", 0),
                "temperature": data.get("temperature", 0),
                "humidity": data.get("humidity", 0),
                "timestamp": data.get("timestamp", "")
            }
    
    cli_client.on_sensor_update(on_sensor_update)
    
    async def monitor_loop():
        try:
            with Live(console=console, refresh_per_second=2) as live:
                while True:
                    # Build table
                    status_text = "🟢 Live" if cli_client.connected else "🔴 Offline"
                    table = Table(
                        title=f"🌡️  Live Sensor Readings | {time.strftime('%H:%M:%S')} | {status_text}", 
                        show_header=True, 
                        header_style="bold cyan"
                    )
                    table.add_column("Plant", style="green", no_wrap=True)
                    table.add_column("💧 Moisture", justify="center", style="blue")
                    table.add_column("🌡️  Temp", justify="center", style="red")
                    table.add_column("💨 Humidity", justify="center", style="cyan")
                    table.add_column("Status", justify="center")
                    table.add_column("Updated", justify="center", style="dim")
                    
                    rules = load_rules()
                    for plant in active_plants:
                        name = plant["name"]
                        data = sensor_data[name]
                        
                        # Determine status
                        plant_type = plant["plant_type"].lower()
                        if plant_type in rules and data["moisture"] > 0:
                            min_moisture = rules[plant_type].get("min_moisture_level", 30)
                            if data["moisture"] < min_moisture:
                                status = "[red]⚠️  LOW[/red]"
                            else:
                                status = "[green]✓ OK[/green]"
                        else:
                            status = "[dim]Waiting...[/dim]"
                        
                        table.add_row(
                            name,
                            f"{data['moisture']:.1f}%" if data['moisture'] > 0 else "-",
                            f"{data['temperature']:.1f}°C" if data['temperature'] > 0 else "-",
                            f"{data['humidity']:.1f}%" if data['humidity'] > 0 else "-",
                            status,
                            data['timestamp']
                        )
                    
                    if not cli_client.connected:
                        table.caption = "[yellow]⚠ Not connected to daemon. Run 'python main.py serve' in another terminal.[/yellow]"
                    
                    live.update(table)
                    await asyncio.sleep(0.5)
                    
        except asyncio.CancelledError:
            pass
    
    try:
        asyncio.run(cli_client.run_with_reconnect(monitor_loop))
    except KeyboardInterrupt:
        console.print("\n[bold green]✓ Sensor monitor stopped[/bold green]")
    except Exception as e:
        console.print(f"\n[bold red]Error: {e}[/bold red]")

def daemon_mode():
    console.print(Panel("[bold green]🤖 AgroCLI Daemon Mode Activated[/bold green]\nMonitoring sensors and automating irrigation 24/7. Press Ctrl+C to stop.", expand=False))
    config = load_config()
    city = config.get("city")
    api_key = config.get("api_key")
    
    # Import WebSocket client for broadcasting to server
    try:
        import websockets
        has_websocket = True
        console.print("[green]✓ WebSocket client enabled[/green]")
    except ImportError:
        has_websocket = False
        console.print("[yellow]⚠ WebSocket not available (install websockets)[/yellow]")
    
    # --- PHASE 3: FAILSAFE & OPTIMIZATION STATE ---
    pump_consecutive_triggers = {} 
    last_db_write_timestamp = {}
    DB_WRITE_INTERVAL_SECONDS = 60
    ws_connection = None
    # ----------------------------------------------
    
    async def connect_to_server():
        """Connect to WebSocket server"""
        nonlocal ws_connection
        try:
            ws_connection = await websockets.connect("ws://localhost:8000/ws")
            console.print("[green]✓ Connected to WebSocket server[/green]")
            return True
        except Exception as e:
            console.print(f"[yellow]⚠ Could not connect to WebSocket server: {e}[/yellow]")
            console.print("[dim]Run 'python main.py serve' in another terminal for real-time features[/dim]")
            return False
    
    async def broadcast_sensor_data(name, moisture, temp, humidity):
        """Helper to broadcast sensor data via HTTP POST to web server"""
        if has_websocket:
            try:
                import aiohttp
                timeout = aiohttp.ClientTimeout(total=5)  # Increase to 5 seconds
                async with aiohttp.ClientSession(timeout=timeout) as session:
                    async with session.post("http://localhost:8000/api/broadcast/sensor", json={
                        "plant_name": name,
                        "moisture": moisture,
                        "temperature": temp,
                        "humidity": humidity
                    }) as response:
                        if response.status == 200:
                            console.print(f"[dim green]✓ {name}[/dim green]")
                        else:
                            console.print(f"[dim yellow]⚠ {name}: HTTP {response.status}[/dim yellow]")
            except asyncio.TimeoutError:
                console.print(f"[dim yellow]⏱ {name}: Timeout (server slow)[/dim yellow]")
            except Exception as e:
                console.print(f"[dim red]✗ {name}: {type(e).__name__}[/dim red]")
    
    async def broadcast_pump_event(name, status, duration=0):
        """Helper to broadcast pump events via HTTP POST to web server"""
        if has_websocket:  # Remove ws_connection check
            try:
                import aiohttp
                timeout = aiohttp.ClientTimeout(total=1)
                async with aiohttp.ClientSession(timeout=timeout) as session:
                    async with session.post("http://localhost:8000/api/broadcast/pump", json={
                        "plant_name": name,
                        "status": status,
                        "duration": duration
                    }) as response:
                        await response.text()  # Consume response
            except Exception:
                pass  # Silently fail if server not running
    
    async def broadcast_alert(message, level="info"):
        """Helper to broadcast system alerts via HTTP POST to web server"""
        if has_websocket:  # Remove ws_connection check
            try:
                import aiohttp
                timeout = aiohttp.ClientTimeout(total=1)
                async with aiohttp.ClientSession(timeout=timeout) as session:
                    async with session.post("http://localhost:8000/api/broadcast/alert", json={
                        "message": message,
                        "level": level
                    }) as response:
                        await response.text()  # Consume response
            except Exception:
                pass  # Silently fail if server not running
    
    async def daemon_loop():
        # Try to connect to WebSocket server
        if has_websocket:
            await connect_to_server()
        
        try:
            while True:
                console.print(f"\n[bold magenta]🕰️  Cycle Check: {time.strftime('%H:%M:%S')}[/bold magenta]")
                weather_cond = None
                if city and api_key:
                    weather_cond = get_weather(city, api_key)
                    
                active_plants = get_all_active_plants()
                if not active_plants:
                    console.print("[yellow]No active plants to monitor. Sleeping...[/yellow]")
                    await asyncio.sleep(10)
                    continue
                    
                for plant in active_plants:
                    name = plant["name"]
                    
                    if name not in pump_consecutive_triggers:
                        pump_consecutive_triggers[name] = 0
                    if name not in last_db_write_timestamp:
                        last_db_write_timestamp[name] = 0
                    
                    moisture = read_soil_moisture(name)
                    temp = read_temperature()
                    humidity = read_humidity()
                    
                    # Broadcast sensor data to WebSocket clients
                    await broadcast_sensor_data(name, moisture, temp, humidity)
                    
                    current_time = time.time()
                    if current_time - last_db_write_timestamp[name] >= DB_WRITE_INTERVAL_SECONDS:
                        log_sensor_data(name, moisture, temp, humidity)
                        last_db_write_timestamp[name] = current_time
                        console.print(f"[dim]Sensor {name} | Moisture: {moisture}% | Temp: {temp}°C | Hum: {humidity}% (Logged to DB)[/dim]")
                    else:
                        console.print(f"[dim]Sensor {name} | Moisture: {moisture}% | Temp: {temp}°C | Hum: {humidity}%[/dim]")
                    
                    tasks = calculate_today_tasks([plant], weather_condition=weather_cond, real_time_moisture=moisture)
                    if tasks:
                        task = tasks[0]
                        if task["needs_water"]:
                            if pump_consecutive_triggers[name] >= 5:
                                msg = f"🚨 [EMERGENCY] {name}: Pump triggered 5 consecutive times! Pump LOCKED."
                                console.print(f"[bold red on yellow]{msg}[/bold red on yellow]")
                                await broadcast_alert(msg, "error")
                                continue
                                
                            if task["skip_watering_due_to_weather"]:
                                msg = f"⚠️ {name}: Needs water but raining. Skipping pump."
                                console.print(f"[yellow]{msg}[/yellow]")
                                await broadcast_alert(msg, "warning")
                                pump_consecutive_triggers[name] = 0
                            else:
                                msg = f"🚨 {name}: Minimum moisture threshold breached!"
                                console.print(f"[bold red]{msg}[/bold red]")
                                await broadcast_alert(msg, "warning")
                                await broadcast_pump_event(name, "on", 3)
                                water_plant(name, duration_seconds=3)
                                update_care(name, "water")
                                await broadcast_pump_event(name, "off")
                                pump_consecutive_triggers[name] += 1
                        else:
                            if pump_consecutive_triggers[name] > 0:
                                msg = f"✅ {name} moisture recovered. Failsafe reset."
                                console.print(f"[green]{msg}[/green]")
                                await broadcast_alert(msg, "info")
                                pump_consecutive_triggers[name] = 0
                
                await asyncio.sleep(5)
                
        except KeyboardInterrupt:
            console.print("\n[bold red]🛑 Daemon Mode gracefully stopped.[/bold red]")
        except asyncio.CancelledError:
            console.print("\n[bold red]🛑 Daemon Mode stopped.[/bold red]")
        except Exception as e:
            console.print(f"[bold red]Daemon Error:[/bold red] {e}")
        finally:
            # Cleanup WebSocket connection
            if ws_connection:
                try:
                    await ws_connection.close()
                except:
                    pass
    
    # Run the async daemon loop
    if has_websocket:
        try:
            asyncio.run(daemon_loop())
        except KeyboardInterrupt:
            console.print("\n[bold green]✓ Daemon stopped cleanly[/bold green]")
    else:
        # Fallback to sync mode if WebSocket not available
        console.print("[yellow]Running in sync mode (no real-time updates)[/yellow]")
        while True:
            try:
                console.print(f"\n[bold magenta]🕰️  Cycle Check: {time.strftime('%H:%M:%S')}[/bold magenta]")
                weather_cond = None
                if city and api_key:
                    weather_cond = get_weather(city, api_key)
                    
                active_plants = get_all_active_plants()
                if not active_plants:
                    console.print("[yellow]No active plants to monitor. Sleeping...[/yellow]")
                    time.sleep(10)
                    continue
                    
                for plant in active_plants:
                    name = plant["name"]
                    
                    if name not in pump_consecutive_triggers:
                        pump_consecutive_triggers[name] = 0
                    if name not in last_db_write_timestamp:
                        last_db_write_timestamp[name] = 0
                    
                    moisture = read_soil_moisture(name)
                    temp = read_temperature()
                    humidity = read_humidity()
                    
                    current_time = time.time()
                    if current_time - last_db_write_timestamp[name] >= DB_WRITE_INTERVAL_SECONDS:
                        log_sensor_data(name, moisture, temp, humidity)
                        last_db_write_timestamp[name] = current_time
                        console.print(f"[dim]Sensor {name} | Moisture: {moisture}% | Temp: {temp}°C | Hum: {humidity}% (Logged to DB)[/dim]")
                    else:
                        console.print(f"[dim]Sensor {name} | Moisture: {moisture}% | Temp: {temp}°C | Hum: {humidity}%[/dim]")
                    
                    tasks = calculate_today_tasks([plant], weather_condition=weather_cond, real_time_moisture=moisture)
                    if tasks:
                        task = tasks[0]
                        if task["needs_water"]:
                            if pump_consecutive_triggers[name] >= 5:
                                console.print(f"[bold red on yellow]🚨 [EMERGENCY] {name}: Pump triggered 5 consecutive times! Pump LOCKED.[/bold red on yellow]")
                                continue
                                
                            if task["skip_watering_due_to_weather"]:
                                console.print(f"[yellow]⚠️ {name}: Needs water but raining. Skipping pump.[/yellow]")
                                pump_consecutive_triggers[name] = 0
                            else:
                                console.print(f"[bold red]🚨 {name}: Minimum moisture threshold breached![/bold red]")
                                water_plant(name, duration_seconds=3)
                                update_care(name, "water")
                                pump_consecutive_triggers[name] += 1
                        else:
                            if pump_consecutive_triggers[name] > 0:
                                console.print(f"[green]✅ {name} moisture recovered. Failsafe reset.[/green]")
                                pump_consecutive_triggers[name] = 0
                
                time.sleep(5)
                
            except KeyboardInterrupt:
                console.print("\n[bold red]🛑 Daemon Mode gracefully stopped.[/bold red]")
                break
            except Exception as e:
                console.print(f"[bold red]Daemon Error:[/bold red] {e}")
                time.sleep(10)

def main():
    parser = argparse.ArgumentParser(description="AgroCLI - Self-Hosted Farming Assistant")
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    parser_init = subparsers.add_parser("init", help="Initialize the garden tracking database")
    
    parser_add = subparsers.add_parser("add", help="Add a new plant")
    parser_add.add_argument("type", help="Plant type (e.g., tomato, chili) matching plants.yaml")
    parser_add.add_argument("name", help="A unique nickname for your plant")
    
    parser_today = subparsers.add_parser("today", help="Get today's tasks")
    parser_today.add_argument("--city", help="City name for weather-aware watering")
    parser_today.add_argument("--api-key", help="OpenWeatherMap API Key")
    parser_today.add_argument("--mark-done", action="store_true", help="Automatically mark all shown tasks as completed today")

    parser_harvest = subparsers.add_parser("harvest", help="Harvest/archive a plant")
    parser_harvest.add_argument("name", help="Nickname of the plant to harvest")
    
    parser_stats = subparsers.add_parser("stats", help="View garden statistics")
    parser_stats.add_argument("--export", metavar="FILE", help="Export garden data to a CSV file (e.g., stats.csv)")

    parser_daemon = subparsers.add_parser("daemon", help="Run in background IoT monitoring loop")
    
    parser_serve = subparsers.add_parser("serve", help="Start the Web Dashboard and API")

    args = parser.parse_args()

    if args.command is None:
        interactive_mode()
        sys.exit(0)

    if args.command != "init" and not os.path.exists("data/garden.db"):
        console.print("[bold red]Database not found![/bold red] Please run [cyan]python main.py init[/cyan] first.")
        sys.exit(1)

    if args.command == "init":
        init()
    elif args.command == "add":
        add(args)
    elif args.command == "today":
        today(args)
    elif args.command == "harvest":
        harvest(args)
    elif args.command == "stats":
        stats(args)
    elif args.command == "daemon":
        daemon_mode()
    elif args.command == "serve":
        from web.server import serve
        serve()
    else:
        parser.print_help()

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        console.print(f"[bold red]Unexpected Error:[/bold red] {e}")
