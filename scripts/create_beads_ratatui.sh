#!/bin/bash

echo "Creating Epic: Modernize Ratatui UI..."

EPIC_OUTPUT=$(br create --type=epic \
  --title="Modernize Ratatui UI" \
  --description="$(cat <<'EOF'
Upgrade the AgroCLI terminal UI (Ratatui) to have a modern, premium aesthetic. This includes RGB color palettes, structured layouts, gauge widgets for sensors, and polished popups.
EOF
)" \
  --external-ref="prd:modern-tui")

# Extract the ID assuming it prints the ID or we can just use a placeholder in standard practice, 
# but usually `br` commands in shell scripts might be sequential. 
# Since we don't know the exact IDs it generates, we can guide the user to run them step by step, 
# or use a script that captures it. For simplicity, we'll use placeholder EPIC_ID that the user replaces.
# Wait, let's just use variables in bash.

EPIC_ID=$(echo "$EPIC_OUTPUT" | grep -o 'ralph-tui-[a-zA-Z0-9]*' | head -n 1)
if [ -z "$EPIC_ID" ]; then
    # Fallback if the output doesn't contain ID directly
    echo "Warning: Could not extract Epic ID. Please insert it manually."
    EPIC_ID="REPLACE_WITH_EPIC_ID"
fi

echo "Epic created with ID: $EPIC_ID"

echo "Creating US-001..."
B1_OUT=$(br create \
  --parent=$EPIC_ID \
  --title="US-001: Implement Core Theme Engine" \
  --description="$(cat <<'EOF'
As a developer, I need a centralized theme module so that UI styling is consistent across all screens.

## Acceptance Criteria
- [ ] Create `src/tui/theme.rs` (or similar) to hold color definitions and reusable UI components.
- [ ] Define a premium RGB color palette (e.g., for backgrounds, borders, text, success/warning states).
- [ ] Create a reusable function for drawing stylized Blocks (rounded borders, styled titles).
- [ ] `cargo check` passes
- [ ] `cargo clippy` passes
EOF
)" \
  --priority=1)
B1_ID=$(echo "$B1_OUT" | awk '{print $NF}') # Attempt to get ID, or user adjusts deps manually.

echo "Creating US-002..."
B2_OUT=$(br create \
  --parent=$EPIC_ID \
  --title="US-002: Upgrade Main Menu and Lists" \
  --description="$(cat <<'EOF'
As a user, I want the main menu and selection lists to look premium with clear highlights.

## Acceptance Criteria
- [ ] Refactor `Screen::MainMenu` to use the new Theme engine.
- [ ] Add distinct visual highlight styles (background color, bold text) for selected items.
- [ ] Refactor `Screen::SelectList` similarly.
- [ ] `cargo check` passes
- [ ] `cargo clippy` passes
EOF
)" \
  --priority=2)
B2_ID=$(echo "$B2_OUT" | awk '{print $NF}')

echo "Creating US-003..."
B3_OUT=$(br create \
  --parent=$EPIC_ID \
  --title="US-003: Revamp Live Sensor & Stats Screens" \
  --description="$(cat <<'EOF'
As a user, I want to see sensor data presented beautifully using gauges and structured layouts.

## Acceptance Criteria
- [ ] Update `Screen::LiveSensor` to use `Gauge` widgets for moisture and temperature instead of plain text.
- [ ] Update `Screen::GardenStats` to use a styled `Table` or clean Grid layout.
- [ ] Use thematic colors for OK (green) vs LOW (red/warning) states.
- [ ] `cargo check` passes
- [ ] `cargo clippy` passes
EOF
)" \
  --priority=3)
B3_ID=$(echo "$B3_OUT" | awk '{print $NF}')

echo "Creating US-004..."
B4_OUT=$(br create \
  --parent=$EPIC_ID \
  --title="US-004: Enhance Input and Popups" \
  --description="$(cat <<'EOF'
As a user, I want text inputs and confirmation dialogues to appear as floating popups.

## Acceptance Criteria
- [ ] Create a `centered_rect` helper to draw floating popup windows over the main screen.
- [ ] Update `Screen::TextInput`, `Screen::Confirm`, and `Screen::Message` to render as popups.
- [ ] `cargo check` passes
- [ ] `cargo clippy` passes
EOF
)" \
  --priority=4)
B4_ID=$(echo "$B4_OUT" | awk '{print $NF}')

echo "Syncing beads..."
br sync --flush-only

echo "Done! To add dependencies manually, run:"
echo "br dep add <US-002-ID> <US-001-ID>"
echo "br dep add <US-003-ID> <US-001-ID>"
echo "br dep add <US-004-ID> <US-001-ID>"
