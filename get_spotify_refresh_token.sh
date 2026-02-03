#!/usr/bin/env bash

get_env_value() {
  local key=$1
  local value=""

  value="${!key}"

  if [ -z "$value" ] && [ -f ".env" ]; then
    value=$(grep "^${key}=" .env | cut -d'=' -f2- | xargs)
  fi

  echo "$value"
}

main() {
  CLIENT_ID=$(get_env_value "SPOTIFY_CLIENT_ID")
  CLIENT_SECRET=$(get_env_value "SPOTIFY_CLIENT_SECRET")

  if [ -z "$CLIENT_ID" ] || [ -z "$CLIENT_SECRET" ]; then
    echo "Error: SPOTIFY_CLIENT_ID and SPOTIFY_CLIENT_SECRET must be set"
    exit 1
  fi

  REDIRECT_URI="http://127.0.0.1:8888/callback"

  echo "Spotify Refresh Token Setup"
  echo ""
  echo "Step 1: Visit this URL in your browser:"
  echo ""
  SCOPES="user-read-private user-top-read user-read-recently-played user-library-read"
  ENCODED_REDIRECT_URI="http%3A%2F%2F127.0.0.1%3A8888%2Fcallback"
  ENCODED_SCOPES=$(printf %s "$SCOPES" | sed 's/ /%20/g')
  echo "https://accounts.spotify.com/authorize?client_id=${CLIENT_ID}&response_type=code&redirect_uri=${ENCODED_REDIRECT_URI}&scope=${ENCODED_SCOPES}"
  echo ""
  echo "Step 2: Paste the authorization code from the redirect URL:"
  echo ""
  read -r -p "Authorization code: " AUTH_CODE

  if [ -z "$AUTH_CODE" ]; then
    echo "Error: No authorization code provided"
    exit 1
  fi

  echo ""
  echo "Exchanging authorization code for refresh token..."
  echo ""

  RESPONSE=$(curl -s -X POST "https://accounts.spotify.com/api/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=authorization_code" \
    -d "code=${AUTH_CODE}" \
    -d "redirect_uri=${REDIRECT_URI}" \
    -d "client_id=${CLIENT_ID}" \
    -d "client_secret=${CLIENT_SECRET}")

  REFRESH_TOKEN=$(echo "$RESPONSE" | grep -o '"refresh_token":"[^"]*' | cut -d'"' -f4)

  if [ -z "$REFRESH_TOKEN" ]; then
    echo "Error: Could not get refresh token"
    echo ""
    echo "Response:"
    echo "$RESPONSE"
    exit 1
  fi

  echo "Success! Got your refresh token"
  echo ""
  echo "Add this line to your .env file:"
  echo ""
  echo "SPOTIFY_REFRESH_TOKEN=${REFRESH_TOKEN}"
  echo ""
}

main
