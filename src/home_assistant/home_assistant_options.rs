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

use crate::env;

/// Home Assistant
///
/// ```toml
/// [home_assistant]
/// friendly_name = "Hallway"
/// encryption_key = "..."
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct HomeAssistantOptions {
    /// Object ID used internall by home assistant.
    /// Defaults to "climage.{node_name}".
    pub object_id: Option<String>,

    /// Listen address for ESP Home API server, default "0.0.0.0:6053"
    pub listen_addr: String,

    /// Encryption key as 32 byte base64 string. When not provided, the
    /// connection uses plaintext messages.
    /// See [ESP Home Native API](https://esphome.io/components/api/)
    /// for a tool that generates a random key.
    pub encryption_key: Option<String>,

    /// Server info (not typically displayed in Home Assistant).
    /// Defaults to "ReTherm {version}".
    pub server_info: String,

    /// Node name, defaults to the system hostname
    pub node_name: Option<String>,

    /// Friendly name displayed in as label for thermostat control
    pub friendly_name: String,

    /// Manufactuer name, defaults to "Nest"
    pub manufacturer: String,

    /// Model name, defaults to "Gen2 Thermostat"
    pub model: String,

    /// Mac address, defaults to address of system interface address
    pub mac_address: Option<String>
}

impl HomeAssistantOptions {
    pub fn get_object_id(&self) -> String {
        if let Some(object_id) = &self.object_id {
            object_id.clone()
        } else {
            format!("climate.{}", self.get_node_name())
        }
    }

    pub fn get_node_name(&self) -> String {
        let pkg_name = env::get_pkg_name();

        if let Some(node_name) = &self.node_name {
            node_name.clone()
        } else {
            match env::get_hostname() {
                Ok(hostname) => hostname,
                Err(e) => {
                    log::error!("get_hostname: '{e}'; using '{pkg_name}'");
                    pkg_name.into()
                }
            }
        }
    }

    pub fn get_mac_address(&self) -> String {
        const FAKE_MAC: &str = "01:02:03:04:05:06";

        if let Some(mac_addr) = &self.mac_address {
            mac_addr.clone()
        } else {
            match env::get_mac_addr() {
                Ok(mac_addr) => {
                    if let Some(mac_addr) = mac_addr {
                        mac_addr
                    } else {
                        log::warn!("get_mac_addr None; using '{FAKE_MAC}'");
                        FAKE_MAC.into()
                    }
                }
                Err(e) => {
                    log::error!("get_mac_addr: '{e}'; using '{FAKE_MAC}'");
                    FAKE_MAC.into()
                }
            }
        }
    }
}

impl Default for HomeAssistantOptions {
    fn default() -> Self {
        Self {
            object_id: None,
            listen_addr: "0.0.0.0:6053".to_string(),
            encryption_key: None,
            server_info: format!("ReTherm {}", env::get_pkg_ver()),
            node_name: None,
            friendly_name: "ReTherm Thermostat".to_string(),
            manufacturer: "Nest".to_string(),
            model: "Gen2 Thermostat".to_string(),
            mac_address: None
        }
    }
}
