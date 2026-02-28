import random

def read_soil_moisture(plant_name: str) -> float:
    # 20% to 80% range mock 
    return round(random.uniform(20.0, 80.0), 1)

def read_temperature() -> float:
    return round(random.uniform(22.0, 35.0), 1)

def read_humidity() -> float:
    return round(random.uniform(50.0, 90.0), 1)
