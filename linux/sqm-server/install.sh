#!/usr/bin/env bash
# Run as the machine owner; sudo is used only for the owned installation.
set -euo pipefail
source_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=linux/sqm-server/versions.env
source "$source_dir/versions.env"
if [[ $# != 3 ]]; then
    echo 'Usage: install.sh CANARY_SOURCE GLOBAL_OTBM CANARY_RECEIPT' >&2
    exit 1
fi
canary_source=$(realpath "$1")
map=$(realpath "$2")
receipt=$(realpath "$3")
[[ $(uname -s) == Linux ]]
for tool in docker git python3 bun node caddy curl sha256sum; do command -v "$tool" >/dev/null; done
docker compose version
[[ $(git -C "$canary_source" rev-parse HEAD) == "$CANARY_COMMIT" ]]
printf '%s  %s\n' "$CANARY_SHA256" "$canary_source/canary" "$MAP_SHA256" "$map" | sha256sum -c -
python3 - "$receipt" "$CANARY_SHA256" <<'PY'
import json, sys
assert json.load(open(sys.argv[1]))['sha256'] == sys.argv[2]
PY
df -h /srv
sudo -n true
if ! id sqm-server >/dev/null 2>&1; then
    sudo useradd --system --home-dir /srv/sqm-server --shell /usr/bin/nologin sqm-server
fi
sudo install -d -m 0750 -o sqm-server -g sqm-server /srv/sqm-server
sudo install -d -m 0700 /srv/sqm-server/backups /srv/sqm-server/checkpoints
sudo install -d -m 0750 -g sqm-server /etc/sqm-server

# Existing worlds require an explicit maintenance stop and backup before update.
if sudo systemctl is-active --quiet sqm-canary.service || sudo systemctl is-active --quiet sqm-aac.service; then
    echo 'World is running; run sudo sqm-server backup, then stop sqm-aac and sqm-canary before updating.' >&2
    exit 1
fi
if ! sudo test -e /srv/sqm-server/canary/canary; then
    sudo install -d -o sqm-server -g sqm-server /srv/sqm-server/canary
    git -C "$canary_source" archive "$CANARY_COMMIT" data data-canary data-otservbr-global config.lua.dist schema.sql key.pem LICENSE \
        | sudo tar -x -C /srv/sqm-server/canary
    sudo cp --reflink=auto "$canary_source/canary" /srv/sqm-server/canary/canary
    sudo cp --reflink=auto "$map" /srv/sqm-server/canary/data-otservbr-global/world/otservbr.otbm
    sudo cp "$receipt" /srv/sqm-server/canary/build.receipt.json
    sudo chown -R sqm-server:sqm-server /srv/sqm-server/canary
fi
printf '%s  %s\n' "$CANARY_SHA256" /srv/sqm-server/canary/canary "$MAP_SHA256" /srv/sqm-server/canary/data-otservbr-global/world/otservbr.otbm | sudo sha256sum -c -
if ! sudo test -d /srv/sqm-server/slenderaac/.git; then
    sudo git clone https://github.com/opentibiabr/slenderaac.git /srv/sqm-server/slenderaac
    sudo git -C /srv/sqm-server/slenderaac checkout --detach "$SLENDER_COMMIT"
    sudo chown -R sqm-server:sqm-server /srv/sqm-server/slenderaac
fi
[[ $(sudo -u sqm-server git -C /srv/sqm-server/slenderaac rev-parse HEAD) == "$SLENDER_COMMIT" ]]
# Read as the owner: the service account cannot traverse the owner's home.
# shellcheck disable=SC2024
if sudo -u sqm-server git -C /srv/sqm-server/slenderaac apply --check < "$source_dir/optional-verification-mail.patch" 2>/dev/null; then
    sudo -u sqm-server git -C /srv/sqm-server/slenderaac apply < "$source_dir/optional-verification-mail.patch"
else
    sudo -u sqm-server git -C /srv/sqm-server/slenderaac apply --reverse --check < "$source_dir/optional-verification-mail.patch"
fi
sudo python3 "$source_dir/configure.py"
sudo install -m 0644 "$source_dir/compose.yaml" /etc/sqm-server/compose.yaml
sudo install -m 0600 "$source_dir/versions.env" /etc/sqm-server/.env
sudo install -m 0755 "$source_dir/manage.sh" /usr/local/sbin/sqm-server
sudo install -D -m 0644 "$source_dir/serve-aac.mjs" /usr/local/lib/sqm-server/serve-aac.mjs
for unit in "$source_dir"/*.service "$source_dir"/*.timer; do
    sudo install -m 0644 "$unit" /etc/systemd/system/
done
sudo install -d /etc/systemd/journald@sqm-server.conf.d
sudo install -m 0644 "$source_dir/journald.conf" /etc/systemd/journald@sqm-server.conf.d/retention.conf
sudo systemctl daemon-reload
sudo systemctl enable --now sqm-database.service
if ! sudo test -e /etc/sqm-server/schema-imported; then
    sudo cat /srv/sqm-server/canary/schema.sql | sudo /usr/local/sbin/sqm-server sql
    # Disable the stock public god/god account; retain stock sample definitions.
    sudo /usr/local/sbin/sqm-server sql <<'SQL'
UPDATE accounts SET password=SHA2(UUID(), 256), email='disabled-stock-account', type=1 WHERE id=1;
SQL
    sudo touch /etc/sqm-server/schema-imported
fi
# The asset installer intentionally refuses symlinked .env files. Give it a
# private regular copy, then put the resulting configuration back in /etc.
sudo rm -f /srv/sqm-server/slenderaac/.env
sudo install -m 0600 -o sqm-server -g sqm-server /etc/sqm-server/aac.env /srv/sqm-server/slenderaac/.env
sudo -u sqm-server bash -s <<'BUILD'
set -euo pipefail
cd /srv/sqm-server/slenderaac
bun install --frozen-lockfile
bun run generate
if [[ ! -e /srv/sqm-server/.aac-migrated ]]; then
    bun run migrate:resolve
fi
bun run migrate
touch /srv/sqm-server/.aac-migrated
bun run import:server-data -- --server-dir /srv/sqm-server/canary --output /srv/sqm-server/library.json
bun run install:assets -- --root /srv/sqm-server/assets
bun run build
BUILD
sudo install -m 0640 -o root -g sqm-server /srv/sqm-server/slenderaac/.env /etc/sqm-server/aac.env
sudo rm /srv/sqm-server/slenderaac/.env
sudo ln -s /etc/sqm-server/aac.env /srv/sqm-server/slenderaac/.env
sudo install -m 0644 "$source_dir/sqm-server.caddy" /etc/caddy/conf.d/sqm-server.caddy
sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
sudo systemctl reload caddy
sudo ufw allow from 192.168.1.0/24 to 192.168.1.191 port 3443 proto tcp comment 'SQM AAC LAN TLS'
sudo ufw allow to 192.168.1.191 port 7172 proto tcp comment 'SQM Canary game'
sudo systemctl enable --now sqm-canary.service sqm-aac.service sqm-health.timer sqm-backup.timer
echo 'Installed. See README.md for TLS routing, firewall and private SMTP configuration.'
