# SQM human-play server on Beast

This is a separate persistent **Canary** world, not SQM's unfinished server or a
QA fixture. Install it through Beast's Dotfiles Linux setup. Existing Factorio,
runner, QA and desktop services are outside this installer's ownership.

## Install

Requires Beast's installed Docker/Compose, Bun, Node, Caddy, Python, Git, UFW and
passwordless sudo. Check disk space first. Run from the owning Dotfiles checkout:

```sh
just sqm-server /path/to/verified/canary-source /path/to/otservbr.otbm /path/to/beast-canary.receipt.json
```

`versions.env` pins the source revision, binary and full global map hashes, and
official SlenderAAC revision. The installer checks them and makes stable copies;
the production world never depends on the reference cache's lifetime. No Canary
build, reseal, fixture SQL or privileged QA actor is involved. SlenderAAC's
official frozen lockfile, migrations, asset installer and library importer are
used. Asset ZIPs and manifests receive the upstream installer's hash validation.

The first install imports the stock Canary schema, disables its public `god/god`
account with a random unusable password, and retains its stock sample definitions.
Human accounts and characters use `/account/signup` and normal AAC forms.
`AUTO_ADMIN_EMAIL` is empty: first signup never grants administration.

The small tracked `optional-verification-mail.patch` skips signup mail only when
confirmation is disabled **and** no SMTP host is configured. Its banner explicitly
says that verification is disabled and no email was sent. Configured SMTP or
required verification retains the upstream email path. Email confirmation is
disabled for the initial playtest; password recovery needs working SMTP.

## Ownership and endpoints

| Path / endpoint | Purpose |
| --- | --- |
| `/srv/sqm-server/canary` | Installed binary, complete stock scripts/world, writable game files |
| `/srv/sqm-server/slenderaac` | Pinned official source, local deployment patch and production build |
| `/srv/sqm-server/mysql` | Durable MySQL data; never delete for deployment |
| `/srv/sqm-server/assets`, `library.json` | Verified website artwork and imported real server definitions |
| `/etc/sqm-server` | Private DB passwords, actual game config, AAC environment, Compose configuration |
| `https://sqm-server.luan.sh/account/signup` | Public account creation after Cloudflare routing is configured |
| `https://sqm-server.luan.sh/api/login` | Public client login; distinct from game TCP |
| `https://192.168.1.191:3443` | LAN website, using Caddy's private CA |
| `127.0.0.1:3081` | Caddy loopback origin for Cloudflare or encrypted SSH forwarding |
| `127.0.0.1:3080`, `127.0.0.1:33080` | Private Node and MySQL listeners |
| `192.168.1.191:7172` | Initial advertised modern game endpoint |

The dedicated `sqm-server` system account runs Canary and Node without a login
shell. MySQL is the separate `sqm-human` Compose project. System services and
Docker restart policies survive reboot. Canary uses session authentication,
no-PvP, free premium, ordinary stock rates and five-minute saves. Daily stock
save does not shut down the world. Initial characters are level 1/no vocation;
the AAC offers stock Dawnport/Rookgaard choices. Actual temple coordinates come
from the loaded global map. Older Canary compatibility ports exist but are not
published by this setup; use the modern client endpoint.

Only the owned `3443/tcp` LAN and `7172/tcp` game UFW rules are added. Existing
firewall rules, Caddy sites, Cloudflare tunnel and router policy are preserved.
No NAT or public DNS rule is inferred by the installer.

Cloudflare must route `sqm-server.luan.sh` to `http://127.0.0.1:3081`. HTTPS
does not carry raw game TCP. Arrange a DNS-only game hostname and router TCP
7172 forwarding to Beast, then change **both** `SERVER_ADDRESS` in the AAC
environment and `ip` in Canary config to that verified endpoint. The AAC login
response compiles its address at build time: rebuild/restart AAC after changes.
Never advertise a Cloudflare-proxied HTTP hostname for arbitrary game TCP.

For private QA, forward 3081 through the verified SSH alias. Caddy preserves the
loopback Host/Origin and uses HTTP only inside the encrypted SSH path; the public
hostname uses HTTPS. The LAN TLS route uses HTTPS throughout. Never disable TLS
verification to test the public package. Caddy's public CA certificate, if needed
for a browser, is `/var/lib/caddy/.local/share/caddy/pki/authorities/local/root.crt`;
its private key must remain on Beast.

Use `http://localhost:LOCALPORT/account/signup` for a forwarded browser session:
this pinned SvelteKit release sets Secure cookies for HTTP `127.0.0.1`, while
explicitly permitting HTTP cookies on `localhost`. The client JSON login endpoint
can use `http://127.0.0.1:LOCALPORT/api/login` because it does not rely on cookies.

## Operate

```sh
sudo sqm-server health
sudo systemctl status sqm-database sqm-canary sqm-aac
sudo journalctl --namespace=sqm-server -u sqm-canary -u sqm-aac -n 100
sudo systemctl restart sqm-canary  # SIGTERM saves before the replacement starts
sudo systemctl restart sqm-aac
```

The service journal namespace retains at most 100 MB and seven days. MySQL's
Docker local logs retain three 10 MB files. `sqm-health.timer` checks the DB,
website, login API and game socket every two minutes; failures remain visible
in `systemctl --failed` and its journal. It does not reboot unrelated services.

The small Node entrypoint delegates to the upstream generated AAC handler and
limits authentication POSTs per client address: 120 API requests or 30 web
login/signup/recovery submissions per minute, with HTTP 429 and Retry-After.
Caddy overwrites the address header, trusts Cloudflare's client identity only
on the loopback tunnel origin, and caps request bodies at 1 MB. Counters live
only in bounded memory; a website restart resets them.

SMTP settings and administrator credentials belong in private files under
`/etc/sqm-server`, never in Git, issues or chat. To enable delivery, configure the
actual provider's host/port/user/password/from and rebuild AAC. With confirmation
required, test real email delivery before opening signup. Keep payment flags
disabled unless the operator intentionally configures payment integrations.

To bootstrap a website operator, create its account with a unique private email
and strong generated password through the normal signup form, then promote only
that account's `accounts.type` to 6. Its game character remains an ordinary group-1
character. Store credentials in `/etc/sqm-server/operator.json` (0600). Leave
`AUTO_ADMIN_EMAIL` empty; no public signup may choose administrative authority.

## Saved-state checkpoints and backups

After ordinary logout/save, take a read-only, consistent **complete database**
checkpoint. All character rows, items, binary attributes, storage and KV state
are included, together with the website/account schema. Treat dumps as secrets.

```sh
sudo sqm-server snapshot before-play
# Or stream privately to the controlling Mac; this prints nothing to chat:
umask 077
ssh beast sudo /usr/local/sbin/sqm-server dump > /private/path/before.sql
```

Online snapshots reflect persisted DB state; they do not force an in-memory game
save. The daily 04:15 Pacific `sqm-backup.timer` and manual `backup` perform a
brief graceful stop, dump the full database, save private config/provenance and
record every table's extended checksum. Previously active owned services restart
even when backup fails. The newest seven complete backups are retained.

```sh
sudo sqm-server backup
sudo sqm-server verify-backup /srv/sqm-server/backups/TIMESTAMP
```

Verification checks archive hashes, restores into a uniquely named disposable
schema in the **owned** DB, compares every restored table checksum and drops only
that schema. It never imports over the live world. Backups on the same disk cover
operator mistakes; copy a complete private backup off Beast for disk-loss recovery.

For actual recovery, schedule downtime, stop AAC/Canary, take an extra backup,
and verify the selected archive. Import `world.sql.gz` into a fresh schema, check
it with `verify-backup`, then intentionally point the owned DB configuration at
the recovered schema. Recover private configuration from `config.tar.gz` to a
private staging directory for review before replacing current configuration.
Do not overwrite unrelated databases or delete the persistent MySQL directory.

## Updates

Run `sudo sqm-server backup`, stop AAC and Canary, and update the owning setup
revision. The installer refuses to replace a running world, preserves existing
configuration/data and checks pinned installed source/binary identities. Changing
an upstream pin requires an intentional reviewed upgrade with backup and normal
client acceptance. Never update another owner's live checkout or reuse a CI
target. Rebuild AAC after changing compiled `.env` values. Review the deployment
patch against the new upstream before upgrading it.
