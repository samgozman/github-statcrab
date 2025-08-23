#!/bin/bash

# Test script to demonstrate GitHub API key functionality
echo "Testing GitHub Stats Fetcher functionality..."
echo "=============================================="

# Start the server in background
echo "Starting server..."
cargo run --bin server &
SERVER_PID=$!

# Wait for server to start
sleep 5

echo "1. Testing without GITHUB_TOKEN (should get error card with rate limit message):"
curl -s "http://localhost:3000/api/stats-card?username=octocat" | grep -o "Failed to fetch[^<]*" | head -1

echo ""
echo "2. Testing validation (invalid hide parameter - should get 400 status):"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3000/api/stats-card?username=test&hide=invalid_field")
echo "HTTP Status Code: $HTTP_CODE"

echo ""
echo "3. Testing with fake GITHUB_TOKEN (should still fail but with authentication):"
GITHUB_TOKEN=fake_token_12345 curl -s "http://localhost:3000/api/stats-card?username=octocat" | grep -o "Failed to fetch[^<]*" | head -1

echo ""
echo "4. Testing language card error handling:"
curl -s "http://localhost:3000/api/langs-card?username=nonexistentuser" | grep -o "Failed to fetch[^<]*" | head -1

# Clean up
echo ""
echo "Stopping server..."
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo "Test complete!"
echo ""
echo "Note: In a production environment with a valid GITHUB_TOKEN,"
echo "the API calls would succeed and return actual GitHub data."