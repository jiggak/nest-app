+++
title = "Troubleshooting"
template = "docgen.html"

[extra]
toc = true
+++

## Home Assistant cannot connect

On the thermostat, confirm that ReTherm is running:

```sh
pidof retherm
```

Confirm that the configured ESPHome Native API port is listening. The default
is 6053:

```sh
netstat -ltn | grep 6053
```

Then check:

- Home Assistant is using the thermostat's current IP address.
- The Home Assistant host can route to the thermostat's network and no firewall
  blocks the configured TCP port.
- `home_assistant.listen_addr` and the port entered in Home Assistant agree.
- Home Assistant has the exact `encryption_key` from `/retherm/config.toml`, or
  both sides are configured without encryption.
- The stock application was stopped with `/etc/init.d/nestlabs stop` before
  ReTherm was started.

The supplied init script logs to `/var/log/messages`:

```sh
tail /var/log/messages
```

Look for `Listening for HA connection` or an error binding the listen address.

## The ESPHome device does not appear automatically

This is expected. ReTherm does not currently implement mDNS/zeroconf discovery.
In Home Assistant, use **Settings → Devices & services → Add Integration →
ESPHome**, then enter the thermostat's IP address and configured port manually.

## ReTherm fails to start

The supplied init script requires these exact paths:

```text
/retherm/retherm
/retherm/config.toml
```

Check that both files exist and the binary and init script are executable:

```sh
ls -l /retherm/retherm /retherm/config.toml /etc/init.d/retherm
```

Common causes are invalid TOML syntax, an invalid value or value type, an
unreadable configuration file, a missing `/media/data` storage directory, an
invalid base64 encryption key, or device/backplate resources still held by the
stock Nest application. Review `/var/log/messages` for the specific error.

To isolate an init-script problem, run the same command in the foreground and
read its error output:

```sh
/retherm/retherm --config /retherm/config.toml
```

Stop that foreground process before starting ReTherm through the init script.

## Recover the stock thermostat UI

```sh
/etc/init.d/retherm stop
/etc/init.d/nestlabs start
```

Always stop one application before starting the other. If the manual install
procedure was followed without changing boot registration, a reboot restores
the stock application's normal boot behavior.
