+++
title = ""
template = "docgen.html"

[extra]
toc = true

# DO NOT EDIT
# Use `make_docs.sh` to generate content
+++
# App Config file

ReTherm will load configuration from `$RETHERM_STORAGE_DIR/$RETHERM_CONFIG_FILE`.

Defaults to `$PWD/.retherm/config.toml`.

You can also launch retherm with the path to your storage directory.
```bash
# Load config file from /media/data/retherm/config.toml
retherm --storage-dir /media/data/retherm

# Load config file from /media/data/retherm/foo.toml
RETHERM_CONFIG_FILE=foo.toml retherm --storage-dir /media/data/retherm
```

All config options have a default; you only need to create a `config.toml`
file and include options you would like to override.

## temp_deadband

The temperature difference from the setpoint required to trigger an action.

For example, with a target heat temp of 20, and deadband set to 0.4,
the hvac system will turn heat on when temp drops to 19.6.

Defaults to 0.6

## temp_overrun

The temperature difference past the setpoint required to stop an action.

For example, with a target heat temp of 20, and overrun set to 0.2,
the hvac system will turn heat off when temp reaches 20.2.

Defaults to 0.4

## min_off_time

Minimum off time for cooling to allow AC refrigerant pressures to equalize.

Defaults to "5m"

## default_fan_timeout

Default amount of time to run fan, when fan mode is activated.

Defaults to "15m"

# Backplate

```toml
[backplate]
near_pir_threshold = 15
serial_port = "/dev/ttyO2"
wiring = { heat_wire: "W1", cool_wire: "Y1" }
```

## near_pir_threshold

Minimum near proximity value to be considered as movement, default 15

## serial_port

Path to backplate serial device file, default "/dev/ttyO2"

## wiring

HVAC wiring configuration, default `{ heat_wire: "W1", cool_wire: "Y1" }`.
Valid wire names: W1, Y1, G, OB, W2, Y2, Star.

# Home Assistant

```toml
[home_assistant]
friendly_name = "Hallway"
encryption_key = "..."
```

## object_id

Object ID used internall by home assistant.
Defaults to "climage.{node_name}".

## listen_addr

Listen address for ESP Home API server, default "0.0.0.0:6053"

## encryption_key

Encryption key as 32 byte base64 string. When not provided, the
connection uses plaintext messages.
See [ESP Home Native API](https://esphome.io/components/api/)
for a tool that generates a random key.

## server_info

Server info (not typically displayed in Home Assistant).
Defaults to "ReTherm {version}".

## node_name

Node name, defaults to the system hostname

## friendly_name

Friendly name displayed in as label for thermostat control

## manufacturer

Manufactuer name, defaults to "Nest"

## model

Model name, defaults to "Gen2 Thermostat"

## mac_address

Mac address, defaults to address of system interface address

# Backlight

```toml
[backlight]
brightness = 108
timeout = "15s"
```

## brightness

Screen brightness, defaults to 108 (max 120)

## timeout

Timeout before screen turns off, defaults to "15s"

