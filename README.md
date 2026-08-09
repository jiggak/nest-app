# (Re)Therm

ReTherm is a replacement thermostat application for a rooted second-generation
Nest Thermostat. It runs **on the thermostat itself**, drives the Nest display
and backplate, and provides remote control to Home Assistant through an
ESPHome Native API server.

ReTherm is not a plugin for the stock Nest application. The stock application
must be stopped while ReTherm is running, so test the installation before
changing anything that starts automatically at boot.

## How Home Assistant connects

```text
+----------------+       ESPHome Native API        +--------------------------+
| Home Assistant | ------------------------------> | ReTherm on the rooted Nest|
|                |          TCP 6053               | Gen 2 thermostat          |
+----------------+                                  +--------------------------+
```

Home Assistant connects straight to ReTherm at the thermostat's IP address.
This path does not use Google's Nest API, the stock Nest application,
NoLongerEvil's Home Assistant integration, or MQTT. ReTherm currently does not
implement mDNS/zeroconf discovery, so add it to Home Assistant manually by IP.

ReTherm exposes one ESPHome climate entity with:

- current and target temperature;
- `off`, `heat`, `cool`, and `fan only` modes;
- the current HVAC action (`idle`, `heating`, `cooling`, or `fan`); and
- `none` and `away` presets.

Home Assistant can change the target temperature, mode, and away preset. The
entity ID is assigned from the configured ESPHome object/node naming, so do not
assume a fixed entity ID.

## Install on a thermostat

The complete installation guide is on the
[documentation site](https://retherm.kropf.io/install/). In outline:

1. Root the Gen 2 Nest and confirm SSH access.
2. Copy the release binary to `/retherm/retherm` and make it executable.
3. Install [`init.sh`](init.sh) as `/etc/init.d/retherm`.
4. Create `/retherm/config.toml`. This file is required by the supplied init
   script; [`config.example.toml`](config.example.toml) is a working starting
   point.
5. Stop the stock application with `/etc/init.d/nestlabs stop`.
6. Start ReTherm with `/etc/init.d/retherm start`.
7. Confirm the process is running and TCP port 6053 is listening.
8. Add the thermostat to Home Assistant manually with the ESPHome integration.

A minimal configuration is:

```toml
[home_assistant]
friendly_name = "Hallway Nest"
node_name = "hallway-nest"
```

All configuration values are optional, but the configuration **file** is not
optional when using the supplied init script. By default the ESPHome server
listens on `0.0.0.0:6053` without encryption. See
[Configuration](https://retherm.kropf.io/configuration/) for every setting and
[Home Assistant](https://retherm.kropf.io/home-assistant/) for optional ESPHome
Noise encryption.

To return safely to the stock Nest software:

```sh
/etc/init.d/retherm stop
/etc/init.d/nestlabs start
```

## Add it to Home Assistant

1. Start ReTherm and determine the thermostat's IP address.
2. In Home Assistant, open **Settings → Devices & services → Add Integration →
   ESPHome**.
3. Enter the thermostat's IP address and port `6053` (unless `listen_addr` was
   changed).
4. If `encryption_key` is not set in ReTherm, leave encryption/password fields
   blank. If it is set, enter the same key in Home Assistant.
5. Complete the integration and test temperature and HVAC mode changes.

See the [troubleshooting guide](https://retherm.kropf.io/troubleshooting/) if
Home Assistant cannot connect or ReTherm does not start.

## Development

Run `cargo run` to start ReTherm in a simulated SDL window. Device-specific
features do not work in simulation.

For a device build, add the ARM target and obtain the Nest toolchain:

```sh
rustup target add armv7-unknown-linux-gnueabihf
just get-toolchain
just build
```

The binary is written to
`target/armv7-unknown-linux-gnueabihf/release/retherm`. Alternatively, use
`just build-docker` to build with Docker.

For development deployment, start `build_recv.sh` on the thermostat and use
`just push` to build, send the binary with netcat on port 51234, and restart it.

## Stretch goals

- [ ] Fancy animations for screen transtions
- [ ] Extensible list of screens through configuration
  - Possible use case would be controlling other HA devices through
    some sort of configurable menu system
  - Example: Screen for turning on/off other devices

## What about power management?

I'm not sure how important this is since the Nest is always powered.
But I can see how getting the most out of the battery in the event of a power
outage could be important.

This is the behaviour I've observed with the stock Nest app.

* I'm assuming that when the device can no longer be pinged, it has gone into
  some sort of sleep mode
* Waking the screen screen causes network reconnect
* With the display connected to USB for power, it _usually_ remains network
  accessible for some time after the screen turns off, but it will eventually
  stop responding to ping
* With display disconnected from power, it disconnects from network soon after
  display turns off (I have not timed how long it remains accessible)
  * However, if you happen to open an SSH session befor the screen turns off,
    the session will remain active for a while (few minutes at most)
  * SSH session hangs, it doesn't disconnnect
  * If you are quick about it, you can wake the display and resume a hung session
  * If it hangs too long, waking device doesn't help, session remains hung
* When the stock app is **stopped**, and the display is **not** connected to
  power, the device seems to remain network accessible for much longer
  (presummably until battery dies)
* Need to look into what Nest app does; could it be as simple setting kernel
  power state?

## Current limitations

- No mDNS/zeroconf discovery; configure the ESPHome integration by IP.
- No Wi-Fi configuration UI. Keep a tested way to restore the stock Nest
  application so that network settings and SSH access remain recoverable.
- ReTherm is intended for rooted Gen 2 hardware and is not a Google Nest cloud
  integration.
