#!/usr/bin/env bash
# -*- coding: utf-8 -*-

# Setup script for pentagame
# Borrowed from oxidized-cms
# Requires grep and diesel cli (with postgresql as features)
# under GPL v3.0 @ Cobalt <cobalt.rocks> (see LICENSE)

if [[ "$2" != "" ]]; then
    FILE="$2"
else
    FILE="./pentagame.toml"
fi

if ! command -v diesel &>/dev/null; then
    echo "You need the diesel cli with features: 'postgres' for using this script"
    exit 1
elif ! command -v grep &>/dev/null; then
    echo "You need GNU grep for using this script"
    exit 1
fi

# Function to get the values from CONFIG_FILE
get_value() {
    # Greps only matches and uses perl regexp
    echo "$(grep -oP "(?<=$1 = ')([^']*)" "$FILE")"
}

get_value_or() {
    value=$(get_value $1)
    if [[ "$value" == "" ]]; then
        echo "$2"
    else
        echo "$3$value" # This supports a prefix as third arg
    fi
}

# Constructing Database url
# This actually is far more reliable than I thought
DATABASE_URL="postgres://$(get_value "user")$(get_value_or "password" "" ":")@$(get_value "host"):$(get_value_or "port" "5432")/$(get_value "database")"

# Checking if --reset was supplied
# WARNING: This will drop & create the database (this may require extended permissions for your specified user)
if [ "$1" == "reset" ]; then
    echo "Resetting Database"
    diesel database reset --database-url "$DATABASE_URL"
elif [ "$1" == "migration" ]; then
    echo "Creating Migration"
    echo "$2"
    diesel migration generate $2 --database-url "$DATABASE_URL" ${@:3}
    exit 0
elif [ "$1" == "print-schema" ]; then
    echo "Printing schema to src/db/schema.rs"
    diesel database reset --database-url "$DATABASE_URL"
fi

echo "Prepping Database"
diesel database setup --database-url "$DATABASE_URL"

echo "Running Migrations"
diesel migration run --database-url "$DATABASE_URL"

echo "Done"
