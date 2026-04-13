#!/bin/bash
set -e

CONFIG_DIR_FILE="/e2e/config_dir"

if [ ! -f "$CONFIG_DIR_FILE" ]; then
    echo "No config directory specified."
    echo "Waiting for test to write config path to $CONFIG_DIR_FILE..."
    exec sleep infinity
fi

CONFIG_DIR=$(cat "$CONFIG_DIR_FILE")

if [ ! -d "$CONFIG_DIR" ]; then
    echo "ERROR: Config directory does not exist: $CONFIG_DIR"
    exec sleep infinity
fi

exec /app/client "$CONFIG_DIR"
