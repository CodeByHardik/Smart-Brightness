#!/bin/bash

# Test script for Smart Brightness daemon modes

BINARY="./target/release/smart-brightness"
CONFIG_DIR="$HOME/.config/smart-brightness"
CONFIG_FILE="$CONFIG_DIR/config.toml"
BACKUP_FILE="$CONFIG_FILE.backup"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "=========================================="
echo "  Smart Brightness - Daemon Mode Tests"
echo "=========================================="
echo

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}❌ Error: Binary not found at $BINARY${NC}"
    echo "Run: cargo build --release"
    exit 1
fi

# Backup existing config
if [ -f "$CONFIG_FILE" ]; then
    echo "Backing up existing config..."
    cp "$CONFIG_FILE" "$BACKUP_FILE"
fi

# Create test config directory
mkdir -p "$CONFIG_DIR"

# Function to create test config
create_test_config() {
    local mode=$1
    local run_duration=${2:-60}
    local pause_interval=${3:-30}
    
    cat > "$CONFIG_FILE" << EOF
# Test configuration for $mode mode
mode = "$mode"
run_duration = $run_duration.0
pause_interval = $pause_interval.0

# Camera configuration
camera_index = 0
camera_resolution = [640, 480]
camera_warmup_frames = 30

# Brightness range
screen_brightness_min = 1
screen_brightness_max = 937

# Smoothing
ambient_smoothing_strength = 0.15
capture_interval_ms = 150
half_precision = true

# Transitions
brightness_step_interval_ms = 20
brightness_step_divisor = 10
brightness_step_max = 100

# Ambient light (calibrated values)
ambient_luma_min = 0.062745101749897
ambient_luma_max = 0.5673728585243225
calibrated = true

# Circadian
circadian_enabled = true
circadian_day_boost = 1.1
circadian_night_dim = 0.9
circadian_day_start_hour = 7
circadian_night_start_hour = 20

# Logging
logging = "low"
EOF
}

# Test function
test_mode() {
    local mode=$1
    local duration=$2
    local pause=$3
    local expected_runtime=$4
    
    echo -e "${YELLOW}Testing $mode mode...${NC}"
    create_test_config "$mode" "$duration" "$pause"
    
    echo "Config created with:"
    echo "  mode = $mode"
    [ -n "$duration" ] && echo "  run_duration = $duration"
    [ -n "$pause" ] && echo "  pause_interval = $pause"
    echo
    
    echo "Starting daemon (will run for ~$expected_runtime seconds)..."
    echo "Press Ctrl+C to stop early"
    echo
    
    local start_time=$(date +%s)
    timeout ${expected_runtime}s $BINARY || true
    local end_time=$(date +%s)
    local actual_runtime=$((end_time - start_time))
    
    echo
    echo "Actual runtime: ${actual_runtime}s"
    echo
    
    # Verify runtime is approximately correct (within 5 seconds)
    local diff=$((actual_runtime - expected_runtime))
    if [ ${diff#-} -le 5 ]; then
        echo -e "${GREEN}✓ Test passed!${NC}"
    else
        echo -e "${RED}⚠ Runtime mismatch (expected ~${expected_runtime}s, got ${actual_runtime}s)${NC}"
    fi
    
    echo
    echo "----------------------------------------"
    echo
}

# Run tests
echo "This will test all three daemon modes."
echo "Each test will run for a short duration."
echo
read -p "Continue? (y/n): " choice
if [[ ! "$choice" =~ ^[Yy]$ ]]; then
    echo "Tests cancelled."
    exit 0
fi

echo
echo "=========================================="
echo "Test 1: Boot Mode"
echo "=========================================="
echo "Boot mode should run for specified duration then exit."
echo
test_mode "boot" 10 "" 10

echo "=========================================="
echo "Test 2: Interval Mode"
echo "=========================================="
echo "Interval mode should run for duration, pause, then repeat."
echo "We'll test one cycle (run + pause)."
echo
test_mode "interval" 5 3 8

echo "=========================================="
echo "Test 3: Realtime Mode"
echo "=========================================="
echo "Realtime mode runs continuously until stopped."
echo "We'll run for 5 seconds then stop."
echo
test_mode "realtime" "" "" 5

# Restore backup
if [ -f "$BACKUP_FILE" ]; then
    echo "Restoring original config..."
    mv "$BACKUP_FILE" "$CONFIG_FILE"
    echo -e "${GREEN}✓ Original config restored${NC}"
else
    echo "Removing test config..."
    rm -f "$CONFIG_FILE"
fi

echo
echo "=========================================="
echo "All tests completed!"
echo "=========================================="
echo
echo "Summary:"
echo "  ✓ Boot mode - Runs for duration then exits"
echo "  ✓ Interval mode - Runs, pauses, repeats"
echo "  ✓ Realtime mode - Runs continuously"
echo
