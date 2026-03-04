#!/usr/bin/env node

/**
 * AgroCLI - The Intelligent Garden Brain
 * 
 * This is the npm wrapper that launches the compiled Rust binary.
 * The binary is built during `npm install` via the postinstall script.
 */

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

// Determine the binary name based on platform
const isWindows = process.platform === "win32";
const binaryName = isWindows ? "AgroCLI.exe" : "AgroCLI";

// Look for the binary in the release or debug build directory
const packageRoot = path.resolve(__dirname, "..");
const releaseBinary = path.join(packageRoot, "target", "release", binaryName);
const debugBinary = path.join(packageRoot, "target", "debug", binaryName);

let binaryPath;
if (fs.existsSync(releaseBinary)) {
    binaryPath = releaseBinary;
} else if (fs.existsSync(debugBinary)) {
    binaryPath = debugBinary;
} else {
    console.error("❌ AgroCLI binary not found!");
    console.error("");
    console.error("The Rust binary has not been compiled yet.");
    console.error("Please ensure you have Rust installed (https://rustup.rs)");
    console.error("Then run: npm run postinstall");
    process.exit(1);
}

// Forward all arguments to the Rust binary
const args = process.argv.slice(2);
const child = spawn(binaryPath, args, {
    stdio: "inherit",
    cwd: packageRoot,
    env: { ...process.env },
});

child.on("error", (err) => {
    console.error("❌ Failed to start AgroCLI:", err.message);
    process.exit(1);
});

child.on("exit", (code) => {
    process.exit(code ?? 0);
});
