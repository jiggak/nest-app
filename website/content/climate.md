+++
title = "Climate"
template = "docgen.html"

[extra]
toc = true

# DO NOT EDIT
# Use `make_docs.sh` to generate content
+++
# Climate settings file

The climate settings is where you setup presets, create a weekly schedule
to switch preset, and setup "Away Mode" (activate "Away" preset when
thermostat does not detect movement for a period of time).

ReTherm will load climate settings from `$RETHERM_STORAGE_DIR/$RETHERM_CLIMATE_FILE`.

Defaults to `$PWD/.retherm/climate.toml`.

You can also launch retherm with the path to your storage directory.
```bash
# Load config file from /media/data/retherm/climate.toml
retherm --storage-dir /media/data/retherm

# Load config file from /media/data/retherm/foo.toml
RETHERM_CLIMATE_FILE=foo.toml retherm --storage-dir /media/data/retherm
```

# Away Mode

```toml
[away_mode]
preset = "Away"
timeout = "30m"
```

## preset

Named preset to activate for away mode, default "Away"

## timeout

Duration of no proximity movement before going into away mode,
or set to zero to disable away mode. Default "30m".

# Presets

```toml
[[presets]]
name = "Away"
temp = { heat = 15.0, cool = 25.0 }

[[presets]]
name = "Sleep"
temp = { heat = 16.0, cool = 24.0 }

[[presets]]
name = "Home"
temp = { heat = 20.0, cool = 22.0 }
```

## name

Name of preset; must be one of "Away", "Home", "Sleep"

## temp

Preset temperature.

`{ heat = 20.0 }` or `{ cool = 22.0 }` or `{ heat = 20.0, cool = 22.0 }`

# Schedule

```toml
[[schedule]]
days_of_week = "EveryDay"
set_points = [
   { time = "08:00", preset = "Home" },
   { time = "22:00", preset = "Sleep" },
]
```

You can define more than one schedule entry, and it will overlap the
previous. In the example below, the perset will be change to "Home"
at 8am everyday, and change to "Work" at 9am Monday and Wednsday.

```toml
[[schedule]]
days_of_week = "EveryDay"
set_points = [
   { time = "08:00", preset = "Home" }
]

[[schedule]]
days_of_week = ["Monday", "Wednsday"]
set_points = [
   { time = "09:00", preset = "Work" }
]
```

## days_of_week

Days of the week.

One of "EveryDay", "WeekDays", "WeekEnd"

Or...

List of weekdays ["Monday", "Tuesday", ...]

## set_points

List of set points with time of day and name of preset

