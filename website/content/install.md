+++
title = "Install"
template = "docgen.html"

[extra]
toc = true
+++

ReTherm runs directly on a rooted second-generation Nest Thermostat and takes
over the thermostat UI and HVAC control path while it is running. It is not a
plugin for the stock Nest application.

> ReTherm does not configure Wi-Fi. Before stopping the stock application,
> confirm that the thermostat is on the correct network and that you can log in
> over SSH. Keep the recovery commands below available.

## Prerequisites

- A rooted Nest Gen 2 thermostat with working SSH access. Rooting methods
  include [NestDFUAttack](https://github.com/exploiteers/NestDFUAttack),
  [Cuckoo Loader](https://github.com/cuckoo-nest/cuckoo_loader), and
  [NoLongerEvil](https://nolongerevil.com/).
- The thermostat and Home Assistant on networks that can reach each other.
- A current ReTherm binary from the
  [GitHub Releases page](https://github.com/jiggak/retherm/releases), or a binary
  built from source as described in the project
  [README](https://github.com/jiggak/retherm).

The commands below run on the thermostat over SSH unless noted otherwise.

## 1. Install ReTherm

Create the application directory:

```sh
mkdir -p /retherm
```

Download a release. Replace `<VERSION>` with the release tag you selected:

```sh
curl -L -o /retherm/retherm \
  https://github.com/jiggak/retherm/releases/download/<VERSION>/retherm
chmod +x /retherm/retherm
```

You can instead copy the binary from another computer with `scp`; the required
destination is `/retherm/retherm` when using the supplied init script.

## 2. Install the init script

```sh
curl -L -o /etc/init.d/retherm \
  https://raw.githubusercontent.com/jiggak/retherm/refs/heads/main/init.sh
chmod +x /etc/init.d/retherm
```

The script starts this exact command in the background:

```sh
/retherm/retherm --config /retherm/config.toml --syslog INFO
```

Consequently, both `/retherm/retherm` and `/retherm/config.toml` are required.

## 3. Create the configuration

Download the working example:

```sh
curl -L -o /retherm/config.toml \
  https://raw.githubusercontent.com/jiggak/retherm/refs/heads/main/config.example.toml
```

Edit `friendly_name` and `node_name` for the thermostat. The minimal file is:

```toml
[home_assistant]
friendly_name = "Hallway Nest"
node_name = "hallway-nest"
```

Every setting has a default, but the init script still requires the file to
exist. The default ESPHome Native API endpoint is `0.0.0.0:6053` and encryption
is off unless `encryption_key` is configured. See [Configuration](/configuration/)
before changing the listen address, HVAC wiring, or other defaults.

## 4. Stop Nest and start ReTherm

```sh
/etc/init.d/nestlabs stop
/etc/init.d/retherm start
```

Stopping `nestlabs` releases the thermostat UI/control path. Starting ReTherm
then replaces that application for as long as ReTherm is running; the two are
not intended to operate together.

## 5. Verify startup

Confirm the process exists:

```sh
pidof retherm
```

Confirm that the default ESPHome port is listening:

```sh
netstat -ltn | grep 6053
```

The init script sends INFO-level logs to syslog. Inspect recent messages with:

```sh
tail /var/log/messages
```

If you changed `listen_addr`, check its configured port instead. Then follow
the [Home Assistant setup guide](/home-assistant/).

## 6. Test before changing startup behavior

From Home Assistant, confirm that the current temperature updates and test each
HVAC mode your system uses. Verify the physical equipment behaves correctly.

The supplied init script does not register ReTherm to start automatically, and
this guide intentionally leaves the stock Nest application as the reboot
default. ReTherm currently has no Wi-Fi settings UI; keeping the stock boot path
reduces the chance of losing network and SSH access. Only change the thermostat's
boot configuration after you have a tested recovery method appropriate to your
rooting environment.

## Return to the stock Nest application

Stop ReTherm before restarting the stock application:

```sh
/etc/init.d/retherm stop
/etc/init.d/nestlabs start
```

This returns the display and thermostat control path to the normal Nest
software. If ReTherm was only started manually as described above, rebooting
also returns to the stock boot behavior.

## Logging to syslogd

The supplied init script already launches ReTherm with `--syslog INFO`, writing
to `/var/log/messages`. Valid log levels are `OFF`, `ERROR`, `WARN`, `INFO`,
`DEBUG`, and `TRACE`.

For remote log forwarding, append the appropriate remote option to
`/etc/syslogd.options`; for example, where `192.168.1.42` is the log server:

```text
-O /var/log/messages -s 384 -b 15 -u -R 192.168.1.42:514 -L
```

See [Troubleshooting](/troubleshooting/) for startup and connection checks.
