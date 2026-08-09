+++
title = "Home Assistant"
template = "docgen.html"

[extra]
toc = true
+++

## Architecture

ReTherm runs directly on the rooted Nest thermostat. While it is running, it
replaces the stock Nest application and exposes an ESPHome Native API server
from the thermostat itself.

```text
+----------------+       ESPHome Native API        +--------------------------+
| Home Assistant | ------------------------------> | ReTherm on the rooted Nest|
|                |          TCP 6053               | Gen 2 thermostat          |
+----------------+                                  +--------------------------+
```

Home Assistant does **not** communicate with Google's Nest API, the stock Nest
application, NoLongerEvil's Home Assistant integration, or MQTT. No separate
ESPHome device or broker is involved.

## Connect Home Assistant

ReTherm listens for ESPHome Native API connections on TCP port 6053 by default.
It does not currently implement mDNS/zeroconf discovery, so add it manually by
IP address:

1. Start ReTherm on the thermostat.
2. Determine the thermostat's IP address from your router's DHCP client list.
   On the thermostat, `ifconfig` can also show the IPv4 address; the wireless
   interface name depends on the installed root environment.
3. In Home Assistant, go to **Settings → Devices & services → Add Integration →
   ESPHome**.
4. Enter the thermostat's IP address.
5. Use port `6053` unless you changed `home_assistant.listen_addr` in
   `/retherm/config.toml`.
6. If `home_assistant.encryption_key` is not configured, leave encryption and
   password fields blank. ReTherm does not configure an API password.
7. If encryption is configured, enter the same encryption key in Home
   Assistant.
8. Complete the integration.

The configured default `0.0.0.0` is ReTherm's server bind address; do not enter
`0.0.0.0` in Home Assistant. Enter the thermostat's actual IP address.

Use a stable DHCP lease or address reservation if the thermostat's IP address
can change. Give each ReTherm thermostat a unique `node_name`.

## What appears in Home Assistant

ReTherm exposes one climate entity. Its displayed device/friendly name follows
`home_assistant.node_name` and `home_assistant.friendly_name`; Home Assistant
derives the final entity ID, so this guide does not assume a fixed one.

The climate entity reports:

- current temperature;
- target temperature;
- mode: `off`, `heat`, `cool`, or `fan only`;
- HVAC action: `idle`, `heating`, `cooling`, or `fan`; and
- preset: `none` or `away`.

Home Assistant can set the target temperature, change between the four modes,
and enable or clear the away preset. ReTherm advertises a 9–32 °C visual range
with 0.5 °C display steps. Fan-only mode uses ReTherm's configured fan timer;
it is a climate mode, not a separately exposed fan entity.

## Plaintext configuration (default)

```toml
[home_assistant]
friendly_name = "Hallway Nest"
node_name = "hallway-nest"
```

With no `encryption_key`, ReTherm uses plaintext ESPHome Native API messages.
The connection still stays on the network path between Home Assistant and the
thermostat, but it is not encrypted at the ESPHome protocol layer.

## Optional ESPHome encryption

ReTherm supports ESPHome Native API Noise encryption with a base64-encoded
32-byte pre-shared key:

```toml
[home_assistant]
friendly_name = "Hallway Nest"
node_name = "hallway-nest"
encryption_key = "<base64 ESPHome Noise PSK>"
```

For example, generate a key on a trusted computer with:

```sh
openssl rand -base64 32
```

Restart ReTherm after changing the file:

```sh
/etc/init.d/retherm restart
```

Supply exactly the same key when Home Assistant asks for the ESPHome encryption
key. A missing, malformed, or mismatched key prevents the connection. Changing
encryption settings does not change the default TCP port.

See [Configuration](/configuration/) for all ESPHome server naming and listen
options, or [Troubleshooting](/troubleshooting/) if the connection fails. The
[Home Assistant ESPHome integration documentation](https://www.home-assistant.io/integrations/esphome/)
describes the current integration dialog and its Host, Port, Noise PSK, and
deprecated Password fields.
