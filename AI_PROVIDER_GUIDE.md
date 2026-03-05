# AI Provider Configuration Guide (v1.3.0)

## 🤖 Overview

AgroCLI Edge uses **Google Gemini** as its primary intelligent brain. The system is built with a tool-calling architecture that allows the AI to interact directly with your garden's hardware and database.

---

## 🎯 Primary Provider: Google Gemini

### Models Supporting Tool-Calling
- **Gemini 1.5 Flash** - Highly recommended. Ultra-fast and efficient for daily garden management.
- **Gemini 1.5 Pro** - Best for complex multi-plant reasoning and disease diagnosis.

### Setup Instructions
1. **Get API Key**: Visit [Google AI Studio](https://aistudio.google.com/app/apikey).
2. **Configure Env**: Add the following to your `.env` file:
   ```bash
   GEMINI_API_KEY=your_actual_api_key_here
   ```
3. **Rust Engine**: The Rust core will automatically detect this key on startup.

---

## 🛠️ AI Capabilities (Tool-Calling)

The AI Agent isn't just a chatbot; it has "hands" to work in your garden:

- **`get_garden_status`**: Queries the SQLite database for real-time moisture, temp, and plant health.
- **`water_plant_action`**: Triggers the physical pump (or mock pump) for a specific plant.
- **`search_plant_database`**: Looks up care rules in `plants.yaml`.

### Example Commands
- "Bagaimana kondisi kebun saya hari ini?" (Inquiry)
- "Siram tanaman tomat sekarang." (Action)
- "Apakah cabai saya butuh pupuk?" (Reasoning)

---

## 🧪 Simulation Mode

If you don't have an API key, AgroCLI Edge defaults to **Simulation Mode**.

- **Logic**: Rule-based regex parsing.
- **Features**: Supports basic "status" and "water" commands.
- **Cost**: 100% Free and Offline.

---

## 🔧 Troubleshooting

### API 401 Unauthorized
- **Cause**: Invalid API Key.
- **Fix**: Check your `.env` formatting. Ensure there are no spaces around the `=`.

### API 429 Rate Limit
- **Cause**: Free tier limits reached.
- **Fix**: Gemini Flash allows ~15 requests per minute on the free tier. Wait a moment or switch to a different key.

### Tool Failure
- **Cause**: AI tried to water a plant that doesn't exist.
- **Fix**: Ask the AI "Tampilkan semua tanaman saya" first to see the correct names.

---
**Version:** 1.3.0  
**Last Updated:** March 5, 2026
