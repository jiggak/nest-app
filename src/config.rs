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

use std::{fs, path::{Path, PathBuf}, time::Duration};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    backplate::BackplateOptions,
    home_assistant::HomeAssistantOptions,
    schedule::ScheduleEntry,
    state::HvacMode,
    window::BacklightOptions
};

pub mod config_de;
mod preset;
mod schedule_config;

pub use preset::*;
pub use schedule_config::*;
pub(crate) use config_de::is_field_default;

pub mod is_default {
    use super::*;

    is_field_default!(Config, temp_deadband: f32);
    is_field_default!(Config, temp_overrun: f32);
    is_field_default!(Config, min_off_time: Duration);
    is_field_default!(Config, default_fan_timeout: Duration);
    is_field_default!(Config, storage_dir: PathBuf);
    is_field_default!(Config, away_mode: AwayConfig);
    is_field_default!(Config, backplate: BackplateOptions);
    is_field_default!(Config, home_assistant: HomeAssistantOptions);
    is_field_default!(Config, backlight: BacklightOptions);
}

/// Config file
///
/// Launch retherm with the path to your custom configuration.
///
/// ```bash
/// retherm --config ./your_config.toml
/// ```
///
/// All config options have a default; you only need to include options
/// you would like to override in your configuration file.
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

    /// Directory to store app state.
    ///
    /// Defaults to "/media/data"
    #[serde(skip_serializing_if = "is_default::storage_dir")]
    pub storage_dir: PathBuf,

    #[serde(skip_serializing_if = "is_default::away_mode")]
    pub away_mode: AwayConfig,

    #[serde(skip_serializing_if = "is_default::backplate")]
    pub backplate: BackplateOptions,

    #[serde(skip_serializing_if = "is_default::home_assistant")]
    pub home_assistant: HomeAssistantOptions,

    #[serde(skip_serializing_if = "is_default::backlight")]
    pub backlight: BacklightOptions,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub schedule_heat: Vec<ScheduleConfig>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub schedule_cool: Vec<ScheduleConfig>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let toml_src = fs::read_to_string(file_path)?;
        let config = toml::from_str(&toml_src)?;
        Ok(config)
    }

    pub fn schedule_for_mode(&self, mode: &HvacMode) -> Option<&[ScheduleConfig]> {
        match mode {
            HvacMode::Heat => {
                if self.schedule_heat.len() > 0 {
                    Some(&self.schedule_heat)
                } else {
                    None
                }
            }
            HvacMode::Cool => {
                if self.schedule_cool.len() > 0 {
                    Some(&self.schedule_cool)
                } else {
                    None
                }
            }
            _ => None
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            away_mode: AwayConfig::default(),
            backplate: BackplateOptions::default(),
            home_assistant: HomeAssistantOptions::default(),
            backlight: BacklightOptions::default(),
            schedule_heat: Vec::new(),
            schedule_cool: Vec::new(),
            temp_deadband: 0.6,
            temp_overrun: 0.4,
            min_off_time: Duration::from_mins(5),
            default_fan_timeout: Duration::from_mins(15),
            storage_dir: PathBuf::from("/media/data"),
        }
    }
}

/// Away Mode
///
/// ```toml
/// [away_mode]
/// temp_heat = 16.0
/// temp_cool = 20.0
/// timeout = "0s"
/// ```
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct AwayConfig {
    /// Away temp for heating mode, default 16.0
    pub temp_heat: f32,

    /// Away temp for cooling mode, default 22.0
    pub temp_cool: f32,

    /// Duration of no proximity movement before going into away mode,
    /// or set to zero to disable away mode. Default "30m".
    #[serde(with = "config_de::duration")]
    pub timeout: Duration
}

impl Default for AwayConfig {
    fn default() -> Self {
        Self {
            temp_heat: 16.0,
            temp_cool: 22.0,
            timeout: Duration::from_mins(30)
        }
    }
}

// ClimateOptions, ClimateProgram, Program, Policy, Profile
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClimateSettings {
    pub away: AwayMode,
    pub presets: Vec<Preset>,
    pub schedule: Vec<ScheduleEntry>,
}

impl Default for ClimateSettings {
    fn default() -> Self {
        Self {
            away: AwayMode::default(),
            presets: vec![
                Preset {
                    name: PresetName::Away,
                    temp: PresetTemp::Both { heat: 16.0, cool: 24.0 },
                }
            ],
            schedule: vec![],
        }
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
