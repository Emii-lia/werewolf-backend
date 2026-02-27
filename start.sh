#!/bin/bash
set -e

if ! redis-cli -u "$REDIS_URL" EXISTS roles:initialized > /dev/null 2>&1; then
    echo "First run detected. Populating roles..."
    ./populate_roles.sh

    redis-cli -u "$REDIS_URL" SET roles:initialized true
    echo "Roles populated successfully!"
else
    echo "Roles already populated. Skipping..."
fi

exec ./target/release/werewolf-backend