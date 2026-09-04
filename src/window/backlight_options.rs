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

use crate::config::config_de;

mod is_default {
    use crate::config::is_field_default;
    use super::*;

    is_field_default!(BacklightOptions, brightness: u32);
    is_field_default!(BacklightOptions, timeout: Duration);
}

/// Backlight
///
/// ```toml
/// [backlight]
/// brightness = 108
/// timeout = "15s"
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct BacklightOptions {
    /// Screen brightness, defaults to 108 (max 120)
    #[serde(skip_serializing_if = "is_default::brightness")]
    pub brightness: u32,

    /// Timeout before screen turns off, defaults to "15s"
    #[serde(with = "config_de::duration", skip_serializing_if = "is_default::timeout")]
    pub timeout: Duration
}

impl Default for BacklightOptions {
    fn default() -> Self {
        Self {
            brightness: 108,
            timeout: Duration::from_secs(15)
        }
    }
}
