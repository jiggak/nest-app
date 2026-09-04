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

use serde::{Deserialize, Serialize};

use crate::{
    schedule::ScheduleEntry,
    state::{HvacMode, PresetName},
};

use super::{AwayMode, Preset};

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
