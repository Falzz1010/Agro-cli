# 🚀 Panduan Deployment AgroCLI Edge (v1.3.0)

## 📦 Ikhtisar
AgroCLI Edge dioptimalkan untuk lingkungan dengan keterbatasan sumber daya. Deployment biasanya melibatkan pembuatan (building) biner Rust dan menjalankannya sebagai layanan latar belakang.

---

## 💻 Linux / Raspberry Pi (Direkomendasikan)

### 1. Build Biner
```bash
cargo build --release
```

### 2. Pengaturan Systemd (Berjalan Terus 24/7)
Buat file layanan untuk memastikan AgroCLI Edge dimulai ulang secara otomatis.

`sudo nano /etc/systemd/system/agro-edge.service`

```ini
[Unit]
Description=AgroCLI Edge Smart Farming Hub
After=network.target

[Service]
Type=simple
User=penggunaanda
WorkingDirectory=/jalur/ke/AgroCLI
EnvironmentFile=/jalur/ke/AgroCLI/.env
ExecStart=/jalur/ke/AgroCLI/target/release/AgroCLI serve
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### 3. Aktifkan & Mulai
```bash
sudo systemctl daemon-reload
sudo systemctl enable agro-edge
sudo systemctl start agro-edge
```

---

## 🐳 Deployment Docker

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

### Jalankan Perintah
```bash
docker build -t agro-edge .
docker run -d --name garden-hub -p 8001:8001 -v $(pwd)/data:/app/data agro-edge
```

---

## 🔐 Daftar Periksa Keamanan
- **Reverse Proxy**: Gunakan Nginx atau Caddy untuk menangani HTTPS/SSL.
- **Port Forwarding**: Hanya buka port 8001 jika Basic Auth diaktifkan di `.env`.
- **Database Backup**: Lakukan pencadangan (backup) rutin untuk `data/garden.db`.

---
**AgroCLI Edge - Dibuat untuk Keandalan.**
