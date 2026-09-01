/*
 * ReTherm - Home Assistant native interface for Gen2 Nest thermostat
 * Copyright (C) 2026 Josh Kropf <josh@slashdev.ca>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    backplate::BackplateOptions,
    home_assistant::HomeAssistantOptions,
    schedule::ScheduleEntry,
    state::{HvacMode, PresetName},
    window::BacklightOptions
};

pub mod config_de;
mod preset;

pub use preset::*;
pub(crate) use config_de::is_field_default;

pub mod is_default {
    use super::*;

    is_field_default!(Config, temp_deadband: f32);
    is_field_default!(Config, temp_overrun: f32);
    is_field_default!(Config, min_off_time: Duration);
    is_field_default!(Config, default_fan_timeout: Duration);
    is_field_default!(Config, backplate: BackplateOptions);
    is_field_default!(Config, home_assistant: HomeAssistantOptions);
    is_field_default!(Config, backlight: BacklightOptions);
}

/// App Config file
///
/// ReTherm will load configuration from `$RETHERM_STORAGE_DIR/$RETHERM_CONFIG_FILE`.
///
/// Defaults to `$PWD/.retherm/config.toml`.
///
/// You can also launch retherm with the path to your storage directory.
/// ```bash
/// # Load config file from /media/data/retherm/config.toml
/// retherm --storage-dir /media/data/retherm
///
/// # Load config file from /media/data/retherm/foo.toml
/// RETHERM_CONFIG_FILE=foo.toml retherm --storage-dir /media/data/retherm
/// ```
///
/// All config options have a default; you only need to create a `config.toml`
/// file and include options you would like to override.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Config {
    /// The temperature difference from the setpoint required to trigger an action.
    ///
    /// For example, with a target heat temp of 20, and deadband set to 0.4,
    /// the hvac system will turn heat on when temp drops to 19.6.
    ///
    /// Defaults to 0.6
    #[serde(skip_serializing_if = "is_default::temp_deadband")]
    pub temp_deadband: f32,

    /// The temperature difference past the setpoint required to stop an action.
    ///
    /// For example, with a target heat temp of 20, and overrun set to 0.2,
    /// the hvac system will turn heat off when temp reaches 20.2.
    ///
    /// Defaults to 0.4
    #[serde(skip_serializing_if = "is_default::temp_overrun")]
    pub temp_overrun: f32,

    /// Minimum off time for cooling to allow AC refrigerant pressures to equalize.
    ///
    /// Defaults to "5m"
    #[serde(with = "config_de::duration")]
    #[serde(skip_serializing_if = "is_default::min_off_time")]
    pub min_off_time: Duration,

    /// Default amount of time to run fan, when fan mode is activated.
    ///
    /// Defaults to "15m"
    #[serde(with = "config_de::duration")]
    #[serde(skip_serializing_if = "is_default::default_fan_timeout")]
    pub default_fan_timeout: Duration,

    #[serde(skip_serializing_if = "is_default::backplate")]
    pub backplate: BackplateOptions,

    #[serde(skip_serializing_if = "is_default::home_assistant")]
    pub home_assistant: HomeAssistantOptions,

    #[serde(skip_serializing_if = "is_default::backlight")]
    pub backlight: BacklightOptions,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backplate: BackplateOptions::default(),
            home_assistant: HomeAssistantOptions::default(),
            backlight: BacklightOptions::default(),
            temp_deadband: 0.6,
            temp_overrun: 0.4,
            min_off_time: Duration::from_mins(5),
            default_fan_timeout: Duration::from_mins(15),
        }
    }
}

/// Climate settings file
///
/// The climate settings is where you setup presets, create a weekly schedule
/// to switch preset, and setup "Away Mode" (activate "Away" preset when
/// thermostat does not detect movement for a period of time).
///
/// ReTherm will load climate settings from `$RETHERM_STORAGE_DIR/$RETHERM_CLIMATE_FILE`.
///
/// Defaults to `$PWD/.retherm/climate.toml`.
///
/// You can also launch retherm with the path to your storage directory.
/// ```bash
/// # Load config file from /media/data/retherm/climate.toml
/// retherm --storage-dir /media/data/retherm
///
/// # Load config file from /media/data/retherm/foo.toml
/// RETHERM_CLIMATE_FILE=foo.toml retherm --storage-dir /media/data/retherm
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// FIXME not sold on the name - ClimateOptions, ClimateProgram, Program, Policy, Profile
pub struct ClimateSettings {
    pub presets: Vec<Preset>,
    pub away: Option<AwayMode>,
    pub schedule: Vec<ScheduleEntry>,
}

impl Default for ClimateSettings {
    fn default() -> Self {
        Self {
            presets: vec![],
            away: None,
            schedule: vec![],
        }
    }
}

impl ClimateSettings {
    pub fn get_preset_temp(&self, preset: &PresetName, mode: &HvacMode) -> Option<f32> {
        let preset = self.presets.iter()
            .find(|p| &p.name == preset);
        preset.map_or(None, |p| p.get_temp(mode))
    }

    pub fn preset_names(&self) -> Vec<PresetName> {
        self.presets.iter()
            .map(|p| p.name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ser_skip_defaults() {
        let mut config = Config::default();
        config.temp_deadband = 1.0;
        let s = toml::to_string(&config).unwrap();
        assert_eq!(s, "temp_deadband = 1.0\n");
    }
}
