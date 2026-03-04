# 🚀 AgroCLI Deployment Guide

Complete guide for deploying AgroCLI in various environments.

## 📦 Deployment Options

1. **Local Development** - Run on your computer
2. **Raspberry Pi** - IoT device with real sensors
3. **Docker** - Containerized deployment
4. **Cloud Server** - VPS/Cloud hosting
5. **Home Server** - 24/7 home automation

---

## 💻 Local Development

### Windows
```cmd
# Clone repository
git clone https://github.com/yourusername/agrocli.git
cd agrocli

# Create virtual environment
python -m venv venv
venv\Scripts\activate.bat

# Install dependencies
pip install -r requirements.txt

# Initialize database
python main.py init

# Run
python main.py serve  # Terminal 1
python main.py daemon  # Terminal 2
```

### Linux/Mac
```bash
# Clone repository
git clone https://github.com/yourusername/agrocli.git
cd agrocli

# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Initialize database
python main.py init

# Run
python main.py serve &  # Background
python main.py daemon &  # Background
```

---

## 🥧 Raspberry Pi Deployment

### Prerequisites
- Raspberry Pi 3/4/5
- Raspbian OS (Bullseye or newer)
- Python 3.8+
- Internet connection

### Installation
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Python and dependencies
sudo apt install python3 python3-pip python3-venv git -y

# Clone repository
cd ~
git clone https://github.com/yourusername/agrocli.git
cd agrocli

# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Initialize database
python main.py init
```

### Systemd Service (Auto-start on boot)

**Create service file for Web Server:**
```bash
sudo nano /etc/systemd/system/agrocli-web.service
```

```ini
[Unit]
Description=AgroCLI Web Server
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi/agrocli
Environment="PATH=/home/pi/agrocli/venv/bin"
ExecStart=/home/pi/agrocli/venv/bin/python main.py serve
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**Create service file for Daemon:**
```bash
sudo nano /etc/systemd/system/agrocli-daemon.service
```

```ini
[Unit]
Description=AgroCLI Daemon
After=network.target agrocli-web.service

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi/agrocli
Environment="PATH=/home/pi/agrocli/venv/bin"
ExecStart=/home/pi/agrocli/venv/bin/python main.py daemon
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**Enable and start services:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable agrocli-web agrocli-daemon
sudo systemctl start agrocli-web agrocli-daemon

# Check status
sudo systemctl status agrocli-web
sudo systemctl status agrocli-daemon

# View logs
sudo journalctl -u agrocli-web -f
sudo journalctl -u agrocli-daemon -f
```

### Hardware Setup (Real Sensors)

**Soil Moisture Sensor (Analog):**
```python
# hardware/sensors.py
import board
import busio
import adafruit_ads1x15.ads1115 as ADS
from adafruit_ads1x15.analog_in import AnalogIn

i2c = busio.I2C(board.SCL, board.SDA)
ads = ADS.ADS1115(i2c)
chan = AnalogIn(ads, ADS.P0)

def read_soil_moisture(plant_name):
    voltage = chan.voltage
    moisture = (voltage / 3.3) * 100
    return moisture
```

**DHT22 Temperature/Humidity Sensor:**
```python
import adafruit_dht
import board

dht_device = adafruit_dht.DHT22(board.D4)

def read_temperature():
    return dht_device.temperature

def read_humidity():
    return dht_device.humidity
```

**Relay for Pump:**
```python
import RPi.GPIO as GPIO

PUMP_PIN = 17
GPIO.setmode(GPIO.BCM)
GPIO.setup(PUMP_PIN, GPIO.OUT)

def water_plant(plant_name, duration_seconds=3):
    GPIO.output(PUMP_PIN, GPIO.HIGH)
    time.sleep(duration_seconds)
    GPIO.output(PUMP_PIN, GPIO.LOW)
```

---

## 🐳 Docker Deployment

### Quick Start
```bash
# Clone repository
git clone https://github.com/yourusername/agrocli.git
cd agrocli

# Build and run with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

### Manual Docker Build
```bash
# Build image
docker build -t agrocli:latest .

# Run web server
docker run -d \
  --name agrocli-web \
  -p 8000:8000 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/app/logs \
  agrocli:latest

# Run daemon
docker run -d \
  --name agrocli-daemon \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/app/logs \
  agrocli:latest python main.py daemon
```

### Docker with Hardware Access (Raspberry Pi)
```bash
docker run -d \
  --name agrocli-daemon \
  --device /dev/i2c-1 \
  --device /dev/gpiomem \
  --privileged \
  -v $(pwd)/data:/app/data \
  agrocli:latest python main.py daemon
```

---

## ☁️ Cloud Server Deployment

### DigitalOcean / AWS / Azure

**1. Create VPS:**
- Ubuntu 22.04 LTS
- 1GB RAM minimum
- 10GB storage

**2. SSH into server:**
```bash
ssh root@your-server-ip
```

**3. Install dependencies:**
```bash
apt update && apt upgrade -y
apt install python3 python3-pip python3-venv git nginx -y
```

**4. Clone and setup:**
```bash
cd /opt
git clone https://github.com/yourusername/agrocli.git
cd agrocli
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
python main.py init
```

**5. Create systemd services** (same as Raspberry Pi section)

**6. Setup Nginx reverse proxy:**
```bash
sudo nano /etc/nginx/sites-available/agrocli
```

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

```bash
sudo ln -s /etc/nginx/sites-available/agrocli /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

**7. Setup SSL with Let's Encrypt:**
```bash
sudo apt install certbot python3-certbot-nginx -y
sudo certbot --nginx -d your-domain.com
```

---

## 🏠 Home Server (24/7)

### Using Old PC/Laptop

**1. Install Ubuntu Server:**
- Download Ubuntu Server 22.04 LTS
- Create bootable USB
- Install on old PC

**2. Setup AgroCLI:**
```bash
# Follow Linux installation steps above
# Setup systemd services for auto-start
```

**3. Access from local network:**
```
http://192.168.1.XXX:8000
```

**4. Port forwarding (optional):**
- Login to router admin panel
- Forward port 8000 to server IP
- Access from anywhere: http://your-public-ip:8000

---

## 🔒 Security Considerations

### Production Checklist

- [ ] Change default credentials
- [ ] Enable HTTPS/SSL
- [ ] Setup firewall (ufw)
- [ ] Regular backups
- [ ] Update dependencies
- [ ] Monitor logs
- [ ] Rate limiting
- [ ] Authentication

### Firewall Setup
```bash
# Ubuntu/Debian
sudo ufw allow 22/tcp  # SSH
sudo ufw allow 80/tcp  # HTTP
sudo ufw allow 443/tcp  # HTTPS
sudo ufw enable
```

### Backup Script
```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d_%H%M%S)
tar -czf /backups/agrocli_$DATE.tar.gz \
  /opt/agrocli/data \
  /opt/agrocli/logs \
  /opt/agrocli/plants.yaml

# Keep only last 7 backups
find /backups -name "agrocli_*.tar.gz" -mtime +7 -delete
```

Add to crontab:
```bash
crontab -e
# Add: 0 2 * * * /opt/agrocli/backup.sh
```

---

## 📊 Monitoring

### System Monitoring
```bash
# CPU/Memory usage
htop

# Disk usage
df -h

# Service status
systemctl status agrocli-*

# Logs
tail -f logs/security.log
journalctl -u agrocli-web -f
```

### Application Monitoring

**Add to web/server.py:**
```python
from prometheus_client import Counter, Histogram
import time

REQUEST_COUNT = Counter('requests_total', 'Total requests')
REQUEST_LATENCY = Histogram('request_latency_seconds', 'Request latency')

@app.middleware("http")
async def monitor_requests(request, call_next):
    REQUEST_COUNT.inc()
    start_time = time.time()
    response = await call_next(request)
    REQUEST_LATENCY.observe(time.time() - start_time)
    return response
```

---

## 🔄 Updates

### Manual Update
```bash
cd /opt/agrocli
git pull
source venv/bin/activate
pip install -r requirements.txt --upgrade
sudo systemctl restart agrocli-web agrocli-daemon
```

### Auto-update Script
```bash
#!/bin/bash
# update.sh
cd /opt/agrocli
git pull
source venv/bin/activate
pip install -r requirements.txt --upgrade
sudo systemctl restart agrocli-web agrocli-daemon
echo "Updated at $(date)" >> /var/log/agrocli-updates.log
```

---

## 🆘 Troubleshooting

### Service won't start
```bash
# Check logs
sudo journalctl -u agrocli-web -n 50
sudo journalctl -u agrocli-daemon -n 50

# Check permissions
ls -la /opt/agrocli/data
sudo chown -R pi:pi /opt/agrocli

# Check Python path
which python3
/opt/agrocli/venv/bin/python --version
```

### Port already in use
```bash
# Find process using port 8000
sudo lsof -i :8000
sudo kill -9 <PID>
```

### Database locked
```bash
# Stop services
sudo systemctl stop agrocli-web agrocli-daemon

# Check database
sqlite3 data/garden.db "PRAGMA integrity_check;"

# Restart services
sudo systemctl start agrocli-web agrocli-daemon
```

---

## 📞 Support

For deployment issues:
- Check logs first
- Search GitHub Issues
- Create new issue with deployment details
- Join community discussions

---

**Happy Deploying! 🚀**
