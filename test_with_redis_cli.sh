#!/bin/bash

set -e

echo "Building neural-redis..."
cargo build

echo "Starting neural-redis server..."
./target/debug/neural-redis &
SERVER_PID=$!

# Wait for server to start
sleep 2

echo "Testing PING..."
OUTPUT=$(redis-cli -p 6379 PING 2>&1)
if [ "$OUTPUT" = "PONG" ]; then
    echo "✓ PING test passed"
else
    echo "✗ PING test failed: $OUTPUT"
    kill $SERVER_PID
    exit 1
fi

echo "Testing SET..."
OUTPUT=$(redis-cli -p 6379 SET mykey myvalue 2>&1)
if [ "$OUTPUT" = "OK" ]; then
    echo "✓ SET test passed"
else
    echo "✗ SET test failed: $OUTPUT"
    kill $SERVER_PID
    exit 1
fi

echo "Testing GET..."
OUTPUT=$(redis-cli -p 6379 GET mykey 2>&1)
if [ "$OUTPUT" = "myvalue" ]; then
    echo "✓ GET test passed"
else
    echo "✗ GET test failed: $OUTPUT"
    kill $SERVER_PID
    exit 1
fi

echo "Testing GET for non-existent key..."
OUTPUT=$(redis-cli -p 6379 GET nonexistent 2>&1)
if [ -z "$OUTPUT" ]; then
    echo "✓ GET nil test passed"
else
    echo "✗ GET nil test failed: '$OUTPUT'"
    kill $SERVER_PID
    exit 1
fi

echo "All tests passed! Neural-redis is RESP compliant for basic commands."

# Clean up
kill $SERVER_PID
echo "Server stopped."