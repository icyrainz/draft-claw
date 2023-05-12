#!/usr/bin/env zsh

while IFS='=' read -r key value; do
    echo "Setting secret key: $key"
    flyctl secrets set "$key=$value"
done < .env
