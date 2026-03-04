# AI Provider Configuration Guide

## 🤖 Overview

AgroCLI AI Agent Mode now supports multiple AI providers, allowing you to choose the best model for your needs. You can configure API keys either through environment variables or input them manually when starting AI Agent Mode.

---

## 🎯 Supported AI Providers

### 1. Google Gemini (Recommended)

**Models:**
- **Gemini 1.5 Flash** - Fast, efficient, great for quick responses
- **Gemini 1.5 Pro** - Advanced reasoning, better for complex queries

**Pros:**
- ✅ Free tier available (60 requests/minute)
- ✅ Fast response times
- ✅ Good at function calling
- ✅ Multilingual support

**Get API Key:**
1. Visit: https://makersuite.google.com/app/apikey
2. Sign in with Google account
3. Click "Create API Key"
4. Copy the key

**Environment Variable:**
```bash
GEMINI_API_KEY=your_gemini_api_key_here
```

### 2. Anthropic Claude

**Models:**
- **Claude 3.5 Sonnet** - Balanced performance and capability
- **Claude 3 Opus** - Most capable, best reasoning

**Pros:**
- ✅ Excellent reasoning abilities
- ✅ Long context window (200K tokens)
- ✅ Strong at following instructions
- ✅ Good safety features

**Get API Key:**
1. Visit: https://console.anthropic.com/
2. Sign up for an account
3. Go to API Keys section
4. Generate new key

**Environment Variable:**
```bash
ANTHROPIC_API_KEY=your_anthropic_api_key_here
```

**Note:** Claude integration is coming soon. Currently shows placeholder message.

### 3. OpenAI ChatGPT

**Models:**
- **ChatGPT-4 Turbo** - Latest GPT-4 with improved performance
- **ChatGPT-5** - Next generation (when available)

**Pros:**
- ✅ Industry-leading performance
- ✅ Excellent at complex tasks
- ✅ Strong function calling
- ✅ Wide knowledge base

**Get API Key:**
1. Visit: https://platform.openai.com/api-keys
2. Sign in or create account
3. Click "Create new secret key"
4. Copy and save the key

**Environment Variable:**
```bash
OPENAI_API_KEY=your_openai_api_key_here
```

**Note:** OpenAI integration is coming soon. Currently shows placeholder message.

### 4. Simulation Mode

**No API Key Required!**

**Features:**
- ✅ Works offline
- ✅ No cost
- ✅ Basic commands: 'status', 'siram'
- ✅ Good for testing

**Limitations:**
- ❌ No natural language understanding
- ❌ Limited commands
- ❌ No context awareness

---

## 🚀 Quick Start

### Method 1: Using Environment Variables (Recommended)

1. **Copy .env.example to .env:**
   ```bash
   cp .env.example .env
   ```

2. **Edit .env and add your API key:**
   ```bash
   # Choose one:
   GEMINI_API_KEY=AIzaSyC...your_key_here
   # or
   ANTHROPIC_API_KEY=sk-ant-...your_key_here
   # or
   OPENAI_API_KEY=sk-...your_key_here
   ```

3. **Start AI Agent Mode:**
   ```bash
   cargo run
   # Select: 🤖 AI Agent Mode
   ```

4. **Select your provider:**
   - The system will detect your API key automatically
   - Choose your preferred model
   - Start chatting!

### Method 2: Manual Input (No .env file)

1. **Start AI Agent Mode:**
   ```bash
   cargo run
   # Select: 🤖 AI Agent Mode
   ```

2. **Select AI Provider:**
   ```
   ? Select AI Provider: 
   > Google Gemini 1.5 Flash (Fast & Efficient)
     Google Gemini 1.5 Pro (Advanced)
     Anthropic Claude 3.5 Sonnet (Balanced)
     Anthropic Claude 3 Opus (Most Capable)
     OpenAI ChatGPT-4 Turbo
     OpenAI ChatGPT-5 (Latest)
     Simulation Mode (No API Key Required)
   ```

3. **Enter API Key:**
   ```
   ⚠️  API Key not found in environment
   
   ? Enter API Key: [paste your key here]
   ```

4. **Start chatting!**

---

## 📋 Configuration Display

After configuration, you'll see:

```
  ┌─────────────────────────────────────────────────────────────┐
  │ 📋 ACTIVE CONFIGURATION                                     │
  ├─────────────────────────────────────────────────────────────┤
  │ Provider: Gemini 1.5 Flash                                  │
  │ Status: ✅ Connected                                        │
  └─────────────────────────────────────────────────────────────┘
```

---

## 💡 Usage Examples

### Example 1: Check Garden Status

**Input:**
```
👤 You: How are my plants doing?
```

**Response:**
```
  ┌─────────────────────────────────────────────────────────────┐
  │ 🤖 AgroAI Response:
  ├─────────────────────────────────────────────────────────────┤
  │ I'll check your garden status for you.                     │
  │                                                             │
  │ You have 2 active plants:                                  │
  │ • Tomato-1: Moisture 45%, Temperature 28°C - Healthy      │
  │ • Chili-1: Moisture 32%, Temperature 27°C - Needs water   │
  │                                                             │
  │ Recommendation: Water Chili-1 soon.                        │
  └─────────────────────────────────────────────────────────────┘
```

### Example 2: Water a Plant

**Input:**
```
👤 You: Water the tomato plant
```

**Response:**
```
  🛠️  AI executing tool: water_plant_action...

  ┌─────────────────────────────────────────────────────────────┐
  │ 🤖 AgroAI Response:
  ├─────────────────────────────────────────────────────────────┤
  │ I've activated the water pump for Tomato-1 for 3 seconds.  │
  │ The plant should receive approximately 250ml of water.      │
  │                                                             │
  │ Current moisture level: 45% → Expected: ~65%               │
  └─────────────────────────────────────────────────────────────┘
```

---

## 🔧 Troubleshooting

### Issue: "API Key not found"

**Solution:**
1. Check if .env file exists in project root
2. Verify API key is correctly formatted
3. Restart the application after adding key

### Issue: "API Error 401: Unauthorized"

**Causes:**
- Invalid API key
- Expired API key
- Wrong provider selected

**Solution:**
1. Verify your API key is correct
2. Check if key has proper permissions
3. Generate a new key if needed

### Issue: "API Error 429: Rate Limit"

**Causes:**
- Too many requests
- Free tier limit reached

**Solution:**
1. Wait a few minutes
2. Upgrade to paid tier
3. Use different provider

### Issue: "Connection timeout"

**Causes:**
- No internet connection
- Firewall blocking requests
- API service down

**Solution:**
1. Check internet connection
2. Try different network
3. Use Simulation Mode as fallback

---

## 💰 Cost Comparison

### Free Tiers

| Provider | Free Tier | Rate Limit |
|----------|-----------|------------|
| Gemini Flash | 60 req/min | 1,500 req/day |
| Gemini Pro | 2 req/min | 50 req/day |
| Claude | $5 credit | Limited |
| ChatGPT | $5 credit | Limited |
| Simulation | Unlimited | None |

### Paid Pricing (Approximate)

| Provider | Model | Price per 1M tokens |
|----------|-------|---------------------|
| Gemini | Flash | $0.075 (input) / $0.30 (output) |
| Gemini | Pro | $1.25 (input) / $5.00 (output) |
| Claude | Sonnet | $3.00 (input) / $15.00 (output) |
| Claude | Opus | $15.00 (input) / $75.00 (output) |
| ChatGPT | GPT-4 | $10.00 (input) / $30.00 (output) |

**Note:** Prices subject to change. Check provider websites for current pricing.

---

## 🎯 Which Provider to Choose?

### For Beginners
**Recommendation:** Gemini 1.5 Flash
- Free tier is generous
- Fast responses
- Easy to get started

### For Advanced Users
**Recommendation:** Claude 3.5 Sonnet or Gemini 1.5 Pro
- Better reasoning
- More accurate
- Handles complex queries

### For Testing
**Recommendation:** Simulation Mode
- No API key needed
- Works offline
- Good for development

### For Production
**Recommendation:** Gemini 1.5 Pro or Claude Opus
- Most reliable
- Best performance
- Worth the cost

---

## 🔐 Security Best Practices

1. **Never commit API keys to git**
   ```bash
   # .gitignore should include:
   .env
   ```

2. **Use environment variables**
   - Don't hardcode keys in code
   - Use .env file

3. **Rotate keys regularly**
   - Generate new keys every 3-6 months
   - Revoke old keys

4. **Limit key permissions**
   - Only grant necessary permissions
   - Use separate keys for dev/prod

5. **Monitor usage**
   - Check API usage regularly
   - Set up billing alerts

---

## 📚 Additional Resources

### Gemini
- Documentation: https://ai.google.dev/docs
- Pricing: https://ai.google.dev/pricing
- API Keys: https://makersuite.google.com/app/apikey

### Claude
- Documentation: https://docs.anthropic.com/
- Pricing: https://www.anthropic.com/pricing
- Console: https://console.anthropic.com/

### OpenAI
- Documentation: https://platform.openai.com/docs
- Pricing: https://openai.com/pricing
- API Keys: https://platform.openai.com/api-keys

---

## 🆘 Support

If you encounter issues:
1. Check this guide first
2. Review error messages carefully
3. Try Simulation Mode as fallback
4. Create GitHub issue with details

---

**Version:** 1.2.0  
**Last Updated:** March 4, 2026
