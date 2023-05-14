#!/usr/bin/env zsh

# Check if the config file is provided
if [ -z "$1" ]; then
    echo "Please provide a configuration file."
    exit 1
fi

# Check if the config file exists
if [ ! -f "$1" ]; then
    echo "Configuration file not found: $1"
    exit 1
fi

config=$1

while IFS='=' read -r key value; do
    echo "Setting secret key: $key"
    flyctl --config $config secrets set "$key=$value"
done < .env
