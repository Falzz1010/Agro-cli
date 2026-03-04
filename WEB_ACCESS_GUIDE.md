# Web Dashboard Access Guide

## 🌐 Accessing the Web Dashboard

### Quick Answer

**If you see `ERR_ADDRESS_INVALID` when accessing the web dashboard:**

❌ **DON'T USE:** `http://0.0.0.0:8001`  
✅ **USE INSTEAD:** `http://127.0.0.1:8001` or `http://localhost:8001`

---

## 📖 Understanding the Issue

### What is 0.0.0.0?

`0.0.0.0` is a special IP address that means "all network interfaces" on the server side. It's used for:
- Binding the server to listen on all available network interfaces
- Allowing connections from any IP address (localhost, LAN, etc.)

However, **browsers cannot connect directly to 0.0.0.0** because it's not a valid destination address.

### What to Use Instead

| Address | Description | When to Use |
|---------|-------------|-------------|
| `127.0.0.1` | IPv4 loopback address | ✅ Always works locally |
| `localhost` | Hostname for loopback | ✅ Always works locally |
| `0.0.0.0` | All interfaces (server binding) | ❌ Never use in browser |
| `192.168.x.x` | Your local network IP | ✅ For LAN access |

---

## 🚀 Step-by-Step Access Guide

### 1. Start the Web Server

Open a terminal and run:
```bash
cargo run -- serve
```

You should see output like:
```
🌐 Starting Web Dashboard... Link: http://0.0.0.0:8001
```

### 2. Open Your Browser

**Choose ONE of these URLs:**

#### Option A: Using 127.0.0.1 (Recommended)
```
http://127.0.0.1:8001
```

#### Option B: Using localhost
```
http://localhost:8001
```

#### Option C: Using your LAN IP (for access from other devices)
First, find your IP address:

**Windows:**
```cmd
ipconfig
```
Look for "IPv4 Address" (e.g., 192.168.1.100)

**Linux/Mac:**
```bash
ip addr show
# or
ifconfig
```

Then use:
```
http://192.168.1.100:8001
```

### 3. Verify Connection

You should see the AgroCLI Web Dashboard with:
- Real-time sensor data
- Live charts
- Pump control buttons
- WebSocket status indicator

---

## ⚙️ Configuration

### Checking Your Port

The port is configured in `.env` file:
```env
HOST=0.0.0.0
PORT=8001
```

- `HOST=0.0.0.0` means the server listens on all interfaces
- `PORT=8001` is the port number to use in your browser URL

### Changing the Port

If port 8001 is already in use, you can change it:

1. Edit `.env` file:
```env
PORT=8080
```

2. Restart the server:
```bash
cargo run -- serve
```

3. Access with new port:
```
http://127.0.0.1:8080
```

---

## 🔧 Troubleshooting

### Problem: "Can't reach this page" or "ERR_ADDRESS_INVALID"

**Solution:** Change `0.0.0.0` to `127.0.0.1` in the URL

❌ Wrong: `http://0.0.0.0:8001`  
✅ Correct: `http://127.0.0.1:8001`

### Problem: "Connection refused"

**Possible causes:**
1. Web server is not running
   - Solution: Run `cargo run -- serve` in a terminal

2. Wrong port number
   - Solution: Check `.env` file for correct PORT value

3. Firewall blocking the connection
   - Solution: Allow the port in your firewall settings

### Problem: "This site can't be reached" (timeout)

**Possible causes:**
1. Server crashed or stopped
   - Solution: Check the terminal running the server for errors

2. Port is blocked
   - Solution: Try a different port (e.g., 8080, 3000)

---

## 🌍 Network Access

### Accessing from Same Computer

Use loopback addresses:
- `http://127.0.0.1:8001`
- `http://localhost:8001`

### Accessing from Other Devices on Same Network

1. Find your computer's IP address (e.g., 192.168.1.100)
2. Make sure `HOST=0.0.0.0` in `.env` (allows external connections)
3. Access from other device: `http://192.168.1.100:8001`

**Note:** Make sure your firewall allows incoming connections on the port.

### Accessing from Internet (Advanced)

For internet access, you need:
1. Port forwarding on your router
2. Dynamic DNS or static IP
3. HTTPS with SSL certificate (recommended)
4. Proper security measures

**⚠️ Warning:** Exposing your server to the internet without proper security is dangerous!

---

## 📱 Mobile Access

### Same WiFi Network

1. Find your computer's IP: `192.168.x.x`
2. On your phone/tablet, open browser
3. Visit: `http://192.168.x.x:8001`

### Different Network

You'll need to set up:
- Port forwarding
- Dynamic DNS
- HTTPS (for security)

---

## 💡 Tips

1. **Bookmark the URL** in your browser for quick access
2. **Use 127.0.0.1** instead of localhost for slightly faster DNS resolution
3. **Keep the terminal open** while using the web dashboard
4. **Check the TUI** for the correct URL (it automatically converts 0.0.0.0 to 127.0.0.1)

---

## 🔗 Related Documentation

- [QUICKSTART.md](QUICKSTART.md) - Quick setup guide
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [API_DOCUMENTATION.md](API_DOCUMENTATION.md) - API endpoints

---

**Version:** 1.2.0  
**Last Updated:** March 4, 2026
