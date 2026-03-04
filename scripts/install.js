#!/usr/bin/env node

/**
 * AgroCLI - Postinstall Script
 * 
 * Builds the Rust binary using cargo during `npm install`.
 * Requires Rust toolchain to be installed (https://rustup.rs).
 */

const { execSync } = require("child_process");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");

// Check if cargo is available
function checkCargo() {
    try {
        execSync("cargo --version", { stdio: "pipe" });
        return true;
    } catch {
        return false;
    }
}

function main() {
    console.log("");
    console.log("🌿 ╔══════════════════════════════════════════════╗");
    console.log("🌿 ║       AgroCLI - Postinstall Setup            ║");
    console.log("🌿 ╚══════════════════════════════════════════════╝");
    console.log("");

    // 1. Check Rust toolchain
    if (!checkCargo()) {
        console.error("❌ Rust toolchain not found!");
        console.error("");
        console.error("   AgroCLI is written in Rust and needs to be compiled.");
        console.error("   Please install Rust first:");
        console.error("");
        console.error("   👉 https://rustup.rs");
        console.error("");
        console.error("   After installing Rust, run:");
        console.error("   $ npm run postinstall");
        console.error("");
        process.exit(1);
    }

    console.log("✅ Rust toolchain found");
    console.log("🔨 Building AgroCLI (this may take a few minutes)...");
    console.log("");

    // 2. Build with cargo
    try {
        execSync("cargo build --release", {
            cwd: ROOT,
            stdio: "inherit",
            env: { ...process.env },
        });
    } catch (err) {
        console.error("");
        console.error("❌ Build failed! Please check the error above.");
        console.error("   Make sure all system dependencies are available.");
        process.exit(1);
    }

    // 3. Copy .env.example to .env if .env doesn't exist
    const fs = require("fs");
    const envPath = path.join(ROOT, ".env");
    const envExample = path.join(ROOT, ".env.example");
    if (!fs.existsSync(envPath) && fs.existsSync(envExample)) {
        fs.copyFileSync(envExample, envPath);
        console.log("📄 Created .env from .env.example");
    }

    // 4. Create data directory
    const dataDir = path.join(ROOT, "data");
    if (!fs.existsSync(dataDir)) {
        fs.mkdirSync(dataDir, { recursive: true });
        console.log("📁 Created data/ directory");
    }

    console.log("");
    console.log("🎉 ╔══════════════════════════════════════════════╗");
    console.log("🎉 ║       AgroCLI installed successfully!        ║");
    console.log("🎉 ╚══════════════════════════════════════════════╝");
    console.log("");
    console.log("   Get started:");
    console.log("   $ agrocli              # Interactive TUI");
    console.log("   $ agrocli daemon       # Run automation daemon");
    console.log("   $ agrocli serve        # Start web dashboard");
    console.log("   $ agrocli --help       # See all commands");
    console.log("");
    console.log("   ⚙️  Edit .env to configure your API keys & settings");
    console.log("");
}

main();
