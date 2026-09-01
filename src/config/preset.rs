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

use crate::{config::config_de, state::{HvacMode, PresetName}};

/// Presets
///
/// ```toml
/// [[presets]]
/// name = "Away"
/// temp = { heat = 15.0, cool = 25.0 }
///
/// [[presets]]
/// name = "Sleep"
/// temp = { heat = 16.0, cool = 24.0 }
///
/// [[presets]]
/// name = "Home"
/// temp = { heat = 20.0, cool = 22.0 }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Preset {
    /// Name of preset; must be one of "Away", "Home", "Sleep"
    pub name: PresetName,

    /// Preset temperature.
    ///
    /// `{ heat = 20.0 }` or `{ cool = 22.0 }` or `{ heat = 20.0, cool = 22.0 }`
    pub temp: PresetTemp,
}

impl Preset {
    pub fn get_temp(&self, mode: &HvacMode) -> Option<f32> {
        match mode {
            HvacMode::Cool => {
                match self.temp {
                    PresetTemp::Cool { cool } => Some(cool),
                    PresetTemp::Both { cool, .. } => Some(cool),
                    _ => None
                }
            }
            HvacMode::Heat => {
                match self.temp {
                    PresetTemp::Heat { heat } => Some(heat),
                    PresetTemp::Both { heat, .. } => Some(heat),
                    _ => None
                }
            }
            _ => None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum PresetTemp {
    Heat { heat: f32 },
    Cool { cool: f32 },
    Both { heat: f32, cool: f32 },
}

/// Away Mode
///
/// ```toml
/// [away_mode]
/// preset = "Away"
/// timeout = "30m"
/// ```
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct AwayMode {
    /// Named preset to activate for away mode, default "Away"
    pub preset: PresetName,

    /// Duration of no proximity movement before going into away mode,
    /// or set to zero to disable away mode. Default "30m".
    #[serde(with = "config_de::duration")]
    pub timeout: Duration,
}

impl Default for AwayMode {
    fn default() -> Self {
        Self {
            preset: PresetName::Away,
            timeout: Duration::from_mins(30),
        }
    }
}
