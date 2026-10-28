#!/usr/bin/env bash
set -euo pipefail
umask 077
cd /etc/sqm-server
compose=(docker compose)
sql() {
    mysql_db sqm "$@"
}
# The container shell expands secrets and arguments, never the host shell.
# shellcheck disable=SC2016
mysql_db() {
    if (($# > 1)); then
        # Queries supplied as CLI arguments must not consume an SSH script's stdin.
        "${compose[@]}" exec -T database sh -c 'MYSQL_PWD="$(cat /run/secrets/mysql-root)" exec mysql -uroot "$@"' sh "$@" </dev/null
    else
        "${compose[@]}" exec -T database sh -c 'MYSQL_PWD="$(cat /run/secrets/mysql-root)" exec mysql -uroot "$@"' sh "$@"
    fi
}
checksums() {
    local database=$1 table tables checksum
    tables=$(mysql_db "$database" -Nse 'SHOW TABLES' | sort)
    [[ -n $tables ]]
    while IFS= read -r table; do
        checksum=$(mysql_db "$database" -Nse "CHECKSUM TABLE \`$table\` EXTENDED" | cut -f2)
        [[ $checksum =~ ^[0-9]+$ ]]
        printf '%s\t%s\n' "$table" "$checksum"
    done <<< "$tables"
}
# shellcheck disable=SC2016
dump() {
    "${compose[@]}" exec -T database sh -c 'MYSQL_PWD="$(cat /run/secrets/mysql-root)" exec mysqldump -uroot --single-transaction --routines --triggers --events --hex-blob --order-by-primary --skip-comments --set-gtid-purged=OFF --no-tablespaces sqm' </dev/null
}
case ${1:-help} in
    sql) shift; sql "$@" ;;
    dump) dump ;;
    snapshot)
        tag=${2:-snapshot}
        [[ $tag =~ ^[a-zA-Z0-9_-]+$ ]] || exit 2
        file="/srv/sqm-server/checkpoints/$(date -u +%Y%m%dT%H%M%S)-$tag-$$.sql.gz"
        dump | gzip > "$file.partial"
        mv "$file.partial" "$file"
        sha256sum "$file"
        ;;
    health)
        systemctl is-active --quiet sqm-database sqm-canary sqm-aac
        sql -Nse 'SELECT 1' | grep -qx 1
        curl --fail --silent --max-time 15 -H 'Host: sqm-server.luan.sh' http://127.0.0.1:3080/account/login >/dev/null
        curl --fail --silent --max-time 15 -H 'Content-Type: application/json' -d '{"type":"cacheinfo"}' http://127.0.0.1:3080/api/login | python3 -c 'import json,sys; assert isinstance(json.load(sys.stdin)["playersonline"], int)'
        timeout 5 bash -c 'exec 3<>/dev/tcp/192.168.1.191/7172'
        echo 'database, AAC and game listener healthy'
        ;;
    backup)
        exec 9>/srv/sqm-server/maintenance.lock
        flock -n 9 || { echo 'Another maintenance operation is active' >&2; exit 1; }
        resume=()
        for unit in sqm-aac sqm-canary; do
            if systemctl is-active --quiet "$unit"; then resume+=("$unit"); fi
        done
        trap 'if ((${#resume[@]})); then systemctl start "${resume[@]}"; fi' EXIT
        systemctl stop sqm-aac sqm-canary
        target="/srv/sqm-server/backups/$(date -u +%Y%m%dT%H%M%S)-$$"
        mkdir "$target.partial"
        dump | gzip > "$target.partial/world.sql.gz"
        checksums sqm > "$target.partial/table-checksums.txt"
        tar -czf "$target.partial/config.tar.gz" /etc/sqm-server /srv/sqm-server/canary/build.receipt.json /srv/sqm-server/library.json 2>/dev/null
        (cd "$target.partial"; sha256sum world.sql.gz config.tar.gz table-checksums.txt > SHA256SUMS)
        mv "$target.partial" "$target"
        # Keep the newest seven complete daily/operator backups.
        mapfile -t expired < <(find /srv/sqm-server/backups -mindepth 1 -maxdepth 1 -type d -name '20*' ! -name '*.partial' | sort -r | tail -n +8)
        if ((${#expired[@]})); then rm -rf -- "${expired[@]}"; fi
        echo "$target"
        ;;
    verify-backup)
        target=$(realpath "${2:?Supply a backup directory}")
        [[ $target == /srv/sqm-server/backups/20* && -f $target/SHA256SUMS ]] || exit 2
        (cd "$target"; sha256sum -c SHA256SUMS)
        database="sqm_restore_$(date +%s)_$$"
        trap 'sql -e "DROP DATABASE IF EXISTS \`$database\`"' EXIT
        sql -e "CREATE DATABASE \`$database\`"
        gzip -dc "$target/world.sql.gz" | mysql_db "$database"
        checksums "$database" > "$target/restored-checksums.txt"
        cmp "$target/table-checksums.txt" "$target/restored-checksums.txt"
        echo 'All database table checksums match after isolated restore.'
        ;;
    *) echo 'Usage: sudo sqm-server {health|snapshot [tag]|dump|sql [mysql options]|backup|verify-backup DIRECTORY}' ;;
esac
