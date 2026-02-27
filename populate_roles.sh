#!/bin/bash

# Werewolf Game Role Population Script
# This script populates the database with default roles

API_URL="http://localhost:3001/api"
TOKEN="${1:-}"

if [ -z "$TOKEN" ]; then
    echo "Usage: ./populate_roles.sh <JWT_TOKEN>"
    echo "Please login and provide your JWT token"
    exit 1
fi

echo "Populating roles..."

# Werewolf (Beast)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Werewolf",
    "slug": "werewolf",
    "description": "You are a werewolf. Each night, you wake up with the other werewolves to devour a villager.",
    "role_type": "Beast",
    "image": "/images/roles/werewolf.png"
  }'

echo ""

# Villager (Citizen)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Villager",
    "slug": "villager",
    "description": "You are a simple villager. Your objective is to eliminate all the werewolves.",
    "role_type": "Citizen",
    "image": "/images/roles/villager.png"
  }'

echo ""

# Seer (Citizen - Special)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Seer",
    "slug": "seer",
    "description": "Each night, you can spy on a player and discover their true identity.",
    "role_type": "Citizen",
    "image": "/images/roles/seer.png",
    "priority": 4
  }'

echo ""

# Witch (Citizen - Special)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Witch",
    "slug": "witch",
    "description": "You have two potions: one to resurrect and one to poison.",
    "role_type": "Citizen",
    "image": "/images/roles/witch.png",
    "priority": 0
  }'

echo ""

# Hunter (Citizen - Special)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Hunter",
    "slug": "hunter",
    "description": "If you are killed, you can eliminate another player in retaliation.",
    "role_type": "Citizen",
    "image": "/images/roles/hunter.png",
    "priority": 5
  }'

echo ""

# Cupid (Citizen - Special)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Cupid",
    "slug": "cupid",
    "description": "At the start of the game, you choose two players who fall in love. If one dies, the other dies too.",
    "role_type": "Citizen",
    "image": "/images/roles/cupid.png",
    "priority": 2
  }'

echo ""

# Little Girl (Citizen - Special)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Little Girl",
    "slug": "little-girl",
    "description": "You can spy on the werewolves during their night turn, but be careful not to get caught!",
    "role_type": "Citizen",
    "image": "/images/roles/little-girl.png",
    "priority": 1
  }'

echo ""

# Guard (Citizen - Special)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Guard",
    "slug": "guard",
    "description": "Each night, you can protect a player against the werewolves. You cannot protect the same player two consecutive nights.",
    "role_type": "Citizen",
    "image": "/images/roles/guard.png",
    "priority": 3
  }'

echo ""

# Little Red Riding Hood (Citizen - Special)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Little Red Riding Hood",
    "slug": "little-red-riding-hood",
    "description": "You cannot be killed by the werewolves as long as the Hunter is alive. If the Hunter dies, you become vulnerable.",
    "role_type": "Citizen",
    "image": "/images/roles/little-red-riding-hood.png",
    "priority": 6
  }'

echo ""

# Spy (Neutral)
curl -X POST "$API_URL/roles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Spy",
    "slug": "spy",
    "description": "You can infiltrate among the werewolves and attend their nightly meetings without being detected.",
    "role_type": "Neutral",
    "image": "/images/roles/spy.png",
    "priority": 7
  }'

echo ""

echo "✅ All roles have been populated!"