#!/usr/bin/env sh

# Define paths for sender and receiver
SENDER_DIR="~/fake_input_dir"
RECEIVER_DIR="~/fake_output_dir"
APP_BINARY="./target/release/amanita"

# Create directories if they don't exist
mkdir -p "$SENDER_DIR"
mkdir -p "$RECEIVER_DIR"

# Build the Rust application
echo "Building the Rust application..."
cargo build --release --bin amanita

# Start the receiver instance in the background
echo "Starting the receiver instance..."
$APP_BINARY --output-dir "$RECEIVER_DIR" --port '8080' & RECEIVER_PID=$!

# Give the receiver some time to start up
sleep 5

# Create a test file in the sender directory
echo "Creating a test file..."
echo 'Test content' > "$SENDER_DIR/test_file.txt"

# Start the sender instance
echo "Starting the sender instance..."
$APP_BINARY --from "$SENDER_DIR" --to 'ws://localhost:8080/ws'

# Give some time for the file to be sent
sleep 5

# Verify the file transfer
echo "Verifying file transfer..."
if [ -f "$RECEIVER_DIR/test_file.txt" ]; then
    echo "File synchronization test passed."
else
    echo "File synchronization test failed."
fi


# destroy directories
rm -rf "$SENDER_DIR"
rm -rf "$RECEIVER_DIR"

echo "Removing tmp directories"

# Clean up: Stop the receiver process
kill $RECEIVER_PID
