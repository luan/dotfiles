#!/usr/bin/env python3
"""Create private configuration once; subsequent installs preserve it."""
import os
from pathlib import Path
import re
import secrets
import shutil

root = Path('/etc/sqm-server')
runtime = Path('/srv/sqm-server')
root.mkdir(mode=0o750, exist_ok=True)
os.umask(0o077)
for name in ('mysql-root', 'mysql-app'):
    path = root / name
    if not path.exists():
        path.write_text(secrets.token_hex(32) + '\n')

config = root / 'config.lua'
if not config.exists():
    text = (runtime / 'canary/config.lua.dist').read_text()
    values = {
        'serverName': '"SQM Canary"',
        'serverMotd': '"Welcome to SQM Canary"',
        'ip': '"192.168.1.191"',
        'bindOnlyGlobalAddress': 'false',
        'worldType': '"no-pvp"',
        'mysqlPort': '33080',
        'mysqlUser': '"sqm"',
        'mysqlPass': '"' + (root / 'mysql-app').read_text().strip() + '"',
        'mysqlDatabase': '"sqm"',
        'mysqlDatabaseBackup': 'false',
        'authType': '"session"',
        'toggleDownloadMap': 'false',
        'freePremium': 'true',
        'saveIntervalType': '"minute"',
        'saveIntervalTime': '5',
        'toggleSaveIntervalCleanMap': 'false',
        'globalServerSaveShutdown': 'false',
        'globalServerSaveTime': '"04:00:00"',
        'generateLuaApiDocs': 'false',
    }
    for key, value in values.items():
        text, count = re.subn(r'^' + key + r'\s*=.*$', key + ' = ' + value,
                              text, flags=re.MULTILINE)
        if count != 1:
            raise SystemExit('Expected one config setting: ' + key)
    config.write_text(text)

env = root / 'aac.env'
if not env.exists():
    text = (runtime / 'slenderaac/.env.dist').read_text()
    values = {
        'DATABASE_URL': '"mysql://sqm:' + (root / 'mysql-app').read_text().strip()
                        + '@127.0.0.1:33080/sqm?connection_limit=5"',
        'SERVER_CONFIG_FILE': '/etc/sqm-server/config.lua',
        'PUBLIC_BASE_URL': '"https://sqm-server.luan.sh"',
        'PUBLIC_TITLE': '"SQM Canary"',
        'PUBLIC_DOWNLOAD_URL': '"https://github.com/skrldev/sqm/releases"',
        'PUBLIC_SUPPORT_EMAIL': '"sqm@luan.sh"',
        'PUBLIC_WIKI_URL': '""',
        'PUBLIC_DISCORD_URL': '""',
        'SERVER_ADDRESS': '"192.168.1.191"',
        'SERVER_NAME': '"SQM Canary"',
        'AUTO_ADMIN_EMAIL': '""',
        'ALLOW_LEGACY_SHA1_PASSWORDS': 'false',
        'ENABLE_STRIPE_CUSTOM': 'false',
        'ENABLE_STRIPE_CHECKOUT': 'false',
        'PUBLIC_STRIPE_KEY': '""',
        'SECRET_STRIPE_KEY': '""',
        'SMTP_SERVER': '""',
        'SMTP_USER': '""',
        'SMTP_PASSWORD': '""',
        'NO_REPLY_EMAIL': '"SQM Canary <no-reply@luan.sh>"',
        'SERVER_SAVE_TIME': '04:00:00',
        'SERVER_DATA_FILE': '/srv/sqm-server/library.json',
        'THEME_ASSETS_ROOT': '/srv/sqm-server/assets',
        'OUTFIT_ASSETS_ROOT': '/srv/sqm-server/assets/outfits',
        'ITEM_ASSETS_ROOT': '/srv/sqm-server/assets/items',
        'STORE_ASSETS_ROOT': '/srv/sqm-server/assets/store',
    }
    for key, value in values.items():
        text, count = re.subn(r'^' + key + r'=.*$', key + '=' + value,
                              text, flags=re.MULTILINE)
        if count != 1:
            raise SystemExit('Expected one AAC setting: ' + key)
    env.write_text(text)

for name in ('config.lua', 'aac.env'):
    path = root / name
    shutil.chown(path, user='root', group='sqm-server')
    path.chmod(0o640)
shutil.chown(root, user='root', group='sqm-server')
for link, target in ((runtime / 'canary/config.lua', config),
                     (runtime / 'slenderaac/.env', env)):
    if not link.exists():
        link.symlink_to(target)
