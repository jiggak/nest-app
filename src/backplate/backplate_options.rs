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

mod is_default {
    use crate::config::is_field_default;
    use super::*;

    is_field_default!(BackplateOptions, near_pir_threshold: u16);
    is_field_default!(BackplateOptions, serial_port: String);
    is_field_default!(BackplateOptions, wiring: WireConfig);
}

/// Backplate
///
/// ```toml
/// [backplate]
/// near_pir_threshold = 15
/// serial_port = "/dev/ttyO2"
/// wiring = { heat_wire: "W1", cool_wire: "Y1" }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct BackplateOptions {
    /// Minimum near proximity value to be considered as movement, default 15
    #[serde(skip_serializing_if = "is_default::near_pir_threshold")]
    pub near_pir_threshold: u16,

    /// Path to backplate serial device file, default "/dev/ttyO2"
    #[serde(skip_serializing_if = "is_default::serial_port")]
    pub serial_port: String,

    /// HVAC wiring configuration, default `{ heat_wire: "W1", cool_wire: "Y1" }`.
    /// Valid wire names: W1, Y1, G, OB, W2, Y2, Star.
    #[serde(skip_serializing_if = "is_default::wiring")]
    pub wiring: WireConfig
}

impl Default for BackplateOptions {
    fn default() -> Self {
        Self {
            near_pir_threshold: 15,
            serial_port: String::from("/dev/ttyO2"),
            wiring: WireConfig::HeatAndCool {
                heat_wire: WireId::W1,
                cool_wire: WireId::Y1,
                fan_wire: WireId::G,
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WireId {
    W1, Y1, G, OB, W2, Y2, Star
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum WireConfig {
    HeatAndCool {
        heat_wire: WireId,
        cool_wire: WireId,
        fan_wire: WireId,
    }
}
