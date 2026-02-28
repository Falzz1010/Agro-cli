import os
import json
import sqlite3
import csv
from datetime import datetime

CONFIG_PATH = "data/config.json"
DB_PATH = "data/garden.db"

def load_config() -> dict:
    if not os.path.exists(CONFIG_PATH):
        return {}
    try:
        with open(CONFIG_PATH, "r", encoding="utf-8") as f:
            return json.load(f)
    except (json.JSONDecodeError, IOError):
        return {}

def save_config(key: str, value: str) -> dict:
    os.makedirs(os.path.dirname(CONFIG_PATH), exist_ok=True)
    config = load_config()
    config[key] = value
    with open(CONFIG_PATH, "w", encoding="utf-8") as f:
        json.dump(config, f, indent=4)
    return config

def get_connection():
    os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)
    return sqlite3.connect(DB_PATH)

def init_db():
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS plants (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            plant_type TEXT NOT NULL,
            planted_date TEXT NOT NULL,
            last_watered TEXT NOT NULL,
            last_fertilized TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active'
        )
    ''')
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS sensor_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            ambient_temp REAL NOT NULL,
            ambient_humidity REAL NOT NULL,
            plant_name TEXT NOT NULL,
            soil_moisture REAL NOT NULL
        )
    ''')
    conn.commit()
    conn.close()

def add_plant(plant_type: str, name: str):
    conn = get_connection()
    cursor = conn.cursor()
    today = datetime.now().strftime("%Y-%m-%d")
    try:
        cursor.execute('''
            INSERT INTO plants (name, plant_type, planted_date, last_watered, last_fertilized)
            VALUES (?, ?, ?, ?, ?)
        ''', (name, plant_type, today, today, today))
        conn.commit()
        return True
    except sqlite3.IntegrityError:
        return False
    finally:
        conn.close()

def get_all_active_plants():
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("SELECT name, plant_type, planted_date, last_watered, last_fertilized FROM plants WHERE status = 'active'")
    rows = cursor.fetchall()
    conn.close()
    
    plants = []
    for row in rows:
        plants.append({
            "name": row[0],
            "plant_type": row[1],
            "planted_date": row[2],
            "last_watered": row[3],
            "last_fertilized": row[4]
        })
    return plants

def get_plant(name: str):
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("SELECT name, plant_type, planted_date, last_watered, last_fertilized, status FROM plants WHERE name = ?", (name,))
    row = cursor.fetchone()
    conn.close()
    
    if row:
        return {
            "name": row[0],
            "plant_type": row[1],
            "planted_date": row[2],
            "last_watered": row[3],
            "last_fertilized": row[4],
            "status": row[5]
        }
    return None

def update_care(name: str, care_type: str, date_str: str = None):
    if date_str is None:
        date_str = datetime.now().strftime("%Y-%m-%d")
        
    conn = get_connection()
    cursor = conn.cursor()
    
    if care_type == "water":
        cursor.execute("UPDATE plants SET last_watered = ? WHERE name = ? AND status = 'active'", (date_str, name))
    elif care_type == "fertilizer":
        cursor.execute("UPDATE plants SET last_fertilized = ? WHERE name = ? AND status = 'active'", (date_str, name))
        
    rows_affected = cursor.rowcount
    conn.commit()
    conn.close()
    return rows_affected > 0

def harvest_plant(name: str):
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("UPDATE plants SET status = 'harvested' WHERE name = ? AND status = 'active'", (name,))
    rows_affected = cursor.rowcount
    conn.commit()
    conn.close()
    return rows_affected > 0

def get_garden_stats():
    conn = get_connection()
    cursor = conn.cursor()
    
    cursor.execute("SELECT COUNT(*) FROM plants WHERE status = 'active'")
    active_count = cursor.fetchone()[0]
    
    cursor.execute("SELECT COUNT(*) FROM plants WHERE status = 'harvested'")
    harvested_count = cursor.fetchone()[0]
    
    cursor.execute("SELECT plant_type, COUNT(*) FROM plants GROUP BY plant_type")
    type_counts = dict(cursor.fetchall())
    
    conn.close()
    
    return {
        "active_plants": active_count,
        "harvested_plants": harvested_count,
        "type_breakdown": type_counts
    }

def get_recent_sensor_logs(limit: int = 50):
    conn = get_connection()
    cursor = conn.cursor()
    # Get the latest logs ordered by time
    cursor.execute('''
        SELECT timestamp, plant_name, soil_moisture, ambient_temp, ambient_humidity
        FROM sensor_logs
        ORDER BY id DESC LIMIT ?
    ''', (limit,))
    rows = cursor.fetchall()
    conn.close()
    
    # Reverse to get chronological order for the chart
    rows.reverse()
    
    logs = []
    for r in rows:
        logs.append({
            "timestamp": r[0].split(" ")[1], # Just grab the HH:MM:SS part
            "plant_name": r[1],
            "soil_moisture": r[2],
            "ambient_temp": r[3],
            "ambient_humidity": r[4]
        })
    return logs

def export_stats_to_csv(filepath: str):
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM plants")
    rows = cursor.fetchall()
    headers = [description[0] for description in cursor.description]
    conn.close()
    
    with open(filepath, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(headers)
        writer.writerows(rows)
    return True

def log_sensor_data(plant_name: str, moisture: float, temp: float, humidity: float):
    conn = get_connection()
    cursor = conn.cursor()
    now_str = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    cursor.execute('''
        INSERT INTO sensor_logs (timestamp, ambient_temp, ambient_humidity, plant_name, soil_moisture)
        VALUES (?, ?, ?, ?, ?)
    ''', (now_str, temp, humidity, plant_name, moisture))
    conn.commit()
    conn.close()
    return True
