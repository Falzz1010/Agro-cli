import requests
import logging

def get_weather(city: str, api_key: str = None):
    if not city:
        return None
    
    # Use user-provided key, otherwise fallback to default provided by user
    final_api_key = api_key if api_key else "0020b2ba2c29724e22ed6bba1dcbac94"
        
    url = f"http://api.openweathermap.org/data/2.5/weather?q={city}&appid={final_api_key}"
    try:
        res = requests.get(url, timeout=5)
        if res.status_code == 200:
            data = res.json()
            if "weather" in data and len(data["weather"]) > 0:
                return data["weather"][0]["main"]
    except requests.RequestException as e:
        logging.warning(f"Could not fetch weather: {e}")
    return None
