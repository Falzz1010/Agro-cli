# 🚀 AgroCLI Edge Deployment Guide (v1.3.0)

## 📦 Overview
AgroCLI Edge is optimized for resource-constrained environments. Deployment typically involves building the Rust binary and running it as a background service.

---

## 💻 Linux / Raspberry Pi (Recommended)

### 1. Build Binary
```bash
cargo build --release
```

### 2. Systemd Setup (Continuous 24/7 Running)
Create a service file to ensure AgroCLI Edge restarts automatically.

`sudo nano /etc/systemd/system/agro-edge.service`

```ini
[Unit]
Description=AgroCLI Edge Smart Farming Hub
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/path/to/AgroCLI
EnvironmentFile=/path/to/AgroCLI/.env
ExecStart=/path/to/AgroCLI/target/release/AgroCLI serve
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### 3. Enable & Start
```bash
sudo systemctl daemon-reload
sudo systemctl enable agro-edge
sudo systemctl start agro-edge
```

---

## 🐳 Docker Deployment

### Dockerfile
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/AgroCLI .
COPY --from=builder /app/plants.yaml .
CMD ["./AgroCLI", "serve"]
```

### Run Command
```bash
docker build -t agro-edge .
docker run -d --name garden-hub -p 8001:8001 -v $(pwd)/data:/app/data agro-edge
```

---

## 🔐 Security Checklist
- **Reverse Proxy**: Use Nginx or Caddy to handle HTTPS/SSL.
- **Port Forwarding**: Only expose port 8001 if Basic Auth is enabled in `.env`.
- **Database Backup**: Regularly back up `data/garden.db`.

---
**AgroCLI Edge - Built for Reliability.**
