#!/bin/bash

set -e

echo "Setting keys..."
redis-cli -p 6379 SET burger "A juicy meat patty served between two distinct bun halves."
redis-cli -p 6379 SET salad "A mixture of raw vegetables, usually served with a dressing."
redis-cli -p 6379 SET king "The male ruler of an independent state."
redis-cli -p 6379 SET queen "The female ruler of an independent state."
redis-cli -p 6379 SET laptop "A portable personal computer suitable for mobile use."

echo "Waiting for embeddings to be generated..."
sleep 10

echo "Getting value of burger..."
redis-cli -p 6379 GET burger

echo "Finding similar items to 'burger' (top 3)..."
redis-cli -p 6379 SIMILARITY burger 3

echo "Finding similar items to 'king' (top 2)..."
redis-cli -p 6379 SIMILARITY king 2

echo "Demo complete."