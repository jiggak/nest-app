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

use std::thread;

use anyhow::Result;
use esphome_api::{
    proto::*,
    server::{
        DefaultHandler, MessageSender, MessageStreamProvider,
        MessageThreadError, RequestHandler, ResponseStatus, start_server
    }
};

use crate::{
    events::{Event, EventHandler, EventSender},
    state::{PresetName, ThermostatState}
};

mod home_assistant_options;
pub use home_assistant_options::HomeAssistantOptions;

pub struct HomeAssistant {
    message_sender: MessageSender
}

impl HomeAssistant {
    pub fn new() -> Self {
        Self {
            message_sender: MessageSender::new()
        }
    }

    pub fn start_listener<S>(
        &self,
        options: &HomeAssistantOptions,
        stream_provider: impl MessageStreamProvider<S> + Send + 'static,
        delegate: HvacRequestHandler<impl EventSender + Send + 'static>
    )
        where S: MessageStream + Send + 'static
    {
        let addr = options.listen_addr.clone();

        let connection_observer = self.message_sender.clone();

        let handler = DefaultHandler {
            delegate: delegate,
            server_info: options.server_info.clone(),
            node_name: options.get_node_name(),
            friendly_name: options.friendly_name.clone(),
            manufacturer: options.manufacturer.clone(),
            model: options.model.clone(),
            mac_address: options.get_mac_address()
        };

        thread::spawn(move || {
            loop {
                let result = start_server(
                    &addr,
                    &stream_provider,
                    &connection_observer,
                    &handler
                );

                // Let home assistant server thread try and recover
                // Instead of panicing and crashing
                if let Err(e) = result {
                    log::error!("Restarting HA thread: {e}");
                }
            }
        });
    }
}

impl EventHandler for HomeAssistant {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::State(state) = event {
            let message = ProtoMessage::ClimateStateResponse(state.into());

            let result = self.message_sender.send_message(message);
            match result {
                // Ignoring non-connected errors
                Err(MessageThreadError::NonConnected) => { },
                r => r?
            }
        }

        Ok(())
    }
}

pub struct HvacRequestHandler<S> {
    thermostat_entity: ListEntitiesClimateResponse,
    event_sender: S
}

impl<S: EventSender> HvacRequestHandler<S> {
    pub fn new(thermostat_entity: ListEntitiesClimateResponse, event_sender: S) -> Self {
        Self {
            thermostat_entity,
            event_sender
        }
    }
}

impl<S: EventSender> RequestHandler for HvacRequestHandler<S> {
    fn handle_request<W: MessageWriter>(
        &self,
        message: &ProtoMessage,
        writer: &mut W
    ) -> Result<ResponseStatus> {
        match message {
            ProtoMessage::ListEntitiesRequest(_) => {
                let message = self.thermostat_entity.clone();
                writer.write(&ProtoMessage::ListEntitiesClimateResponse(message))?;

                let message = ListEntitiesDoneResponse::default();
                writer.write(&ProtoMessage::ListEntitiesDoneResponse(message))?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                self.event_sender.send_event(Event::GetState)?;
            }
            ProtoMessage::ClimateCommandRequest(cmd) => {
                if cmd.has_mode {
                    let mode = cmd.mode().try_into()?;
                    self.event_sender.send_event(Event::SetMode(mode))?;
                }
                if cmd.has_target_temperature {
                    let temp = cmd.target_temperature;
                    self.event_sender.send_event(Event::SetTargetTemp(temp))?;
                }
                if cmd.has_preset {
                    let mode = cmd.preset().try_into()
                        .map(|p| Some(p))
                        .unwrap_or(None);

                    self.event_sender.send_event(Event::SetPreset(mode))?;
                }
            }
            _ => { }
        }

        Ok(ResponseStatus::Continue)
    }
}

pub fn thermostat_entity(object_id: String, presets: &[PresetName]) -> ListEntitiesClimateResponse {
    let mut entity = ListEntitiesClimateResponse::default();

    entity.object_id = object_id;
    entity.supported_modes = vec![
        ClimateMode::Off as i32,
        ClimateMode::Heat as i32,
        ClimateMode::Cool as i32,
        ClimateMode::FanOnly as i32,
    ];
    entity.visual_min_temperature = ThermostatState::MIN_TEMP;
    entity.visual_max_temperature = ThermostatState::MAX_TEMP;
    entity.visual_target_temperature_step = 0.5;
    entity.visual_current_temperature_step = 0.5;
    entity.feature_flags =
        ClimateFeature::SUPPORTS_CURRENT_TEMPERATURE |
        ClimateFeature::SUPPORTS_ACTION;

    // Always include "None" which means `None` variant of `Option<Preset>`
    entity.push_supported_presets(ClimatePreset::None);
    for preset in presets {
        let preset: ClimatePreset = (*preset).into();
        entity.push_supported_presets(preset);
    }

    entity
}
