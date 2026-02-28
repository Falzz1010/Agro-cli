import os
import yaml
from datetime import datetime

RULES_PATH = "plants.yaml"

def load_rules():
    if not os.path.exists(RULES_PATH):
        return {}
    with open(RULES_PATH, "r") as f:
        try:
            return yaml.safe_load(f) or {}
        except yaml.YAMLError:
            return {}

def calculate_today_tasks(active_plants: list, weather_condition: str = None, real_time_moisture: float = None) -> list:
    rules = load_rules()
    tasks = []
    today = datetime.now()
    rain_keywords = ["rain", "drizzle", "thunderstorm", "shower"]
    skip_watering_due_to_weather = False
    
    if weather_condition:
        weather_lower = weather_condition.lower()
        if any(kw in weather_lower for kw in rain_keywords):
            skip_watering_due_to_weather = True
    
    for plant in active_plants:
        plant_type = plant["plant_type"].lower()
        if plant_type not in rules:
            continue
            
        rule = rules[plant_type]
        needs_water = False
        
        # Override with Sensor level if available AND rules support it
        if real_time_moisture is not None:
            min_moisture = rule.get("min_moisture_level")
            if min_moisture is not None and real_time_moisture < min_moisture:
                needs_water = True
        else:
             # Fallback to date-based rules
            last_watered_date = datetime.strptime(plant["last_watered"], "%Y-%m-%d")
            days_since_watered = (today - last_watered_date).days
            water_interval = rule.get("water_interval_days", 1)
            needs_water = days_since_watered >= water_interval
        
        # Weather Override
        if needs_water and skip_watering_due_to_weather:
            needs_water = False # Override watering if it's raining
                
        # Calculate fertilizer needs
        last_fertilized_date = datetime.strptime(plant["last_fertilized"], "%Y-%m-%d")
        days_since_fertilized = (today - last_fertilized_date).days
        fertilizer_interval = rule.get("fertilizer_interval_days", 14)
        needs_fertilizer = days_since_fertilized >= fertilizer_interval
        
        if needs_water or needs_fertilizer:
            task = {
                "name": plant["name"],
                "plant_type": plant["plant_type"],
                "needs_water": needs_water,
                "skip_watering_due_to_weather": skip_watering_due_to_weather,
                "water_ml": rule.get("water_ml", 0),
                "needs_fertilizer": needs_fertilizer,
                "sun_hours": rule.get("sun_hours", 0)
            }
            tasks.append(task)
            
    return tasks
