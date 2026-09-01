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

mod backplate;
mod cli;
mod config;
mod drawable;
mod env;
mod events;
mod home_assistant;
mod input_events;
mod schedule;
mod screen;
mod sound;
mod state;
mod storage;
mod theme;
mod timer;
mod widgets;
mod window;

use anyhow::Result;
use esphome_api::server::{EncryptedStreamProvider, PlaintextStreamProvider};
use log::{error, info};

use crate::{
    events::{Event, EventHandler, EventSource},
    home_assistant::{HomeAssistant, HvacRequestHandler, thermostat_entity},
    screen::{MainScreen, ScreenManager}
};

fn main() -> Result<()> {
    let cli = cli::Cli::load();

    if let Some(log_level) = cli.syslog {
        init_syslog(log_level)?;
    } else {
        env_logger::init();
    }

    install_panic_logging();

    let storage_dir = cli.storage_dir
        .unwrap_or(env::default_storage_dir());

    let theme = if let Some(file_path) = cli.theme {
        theme::Theme::load(file_path)?
    } else {
        theme::Theme::default()
    };

    let mut event_source = window::new_event_source()?;

    let mut storage = storage::Storage::new(storage_dir)?;
    let state = storage.read_state()?;
    let config = storage.read_config()?;
    let climate_stettings = storage.read_climate_settings()?;

    let mut state_manager = state::StateManager::new(
        &config,
        &climate_stettings,
        state.clone(),
        event_source.event_sender()
    )?;

    let mut schedule = schedule::ScheduleManager::new(event_source.event_sender());
    schedule.start_schedule(&climate_stettings.schedule);

    let mut backplate = backplate::Backplate::new(&config.backplate, event_source.event_sender())?;
    let mut timers = timer::Timers::new(event_source.event_sender());
    let mut sound = sound::Sound::new()?;

    let mut window = window::new_window(&config.backlight)?;

    let main_screen = MainScreen::new(theme.thermostat.clone(), state, event_source.event_sender());
    let mut screen_manager = ScreenManager::new(theme, main_screen, event_source.event_sender());

    input_events::start_threads(&event_source)?;

    let mut home_assistant = HomeAssistant::new();
    let hass_delegate = HvacRequestHandler::new(
        thermostat_entity(config.home_assistant.get_object_id(), &climate_stettings.preset_names()),
        event_source.event_sender()
    );

    if let Some(key) = &config.home_assistant.encryption_key {
        let stream_factory = EncryptedStreamProvider::new(
            key,
            &config.home_assistant.get_node_name(),
            &config.home_assistant.get_mac_address()
        )?;

        home_assistant.start_listener(
            &config.home_assistant,
            stream_factory,
            hass_delegate
        );
    } else {
        home_assistant.start_listener(
            &config.home_assistant,
            PlaintextStreamProvider::new(),
            hass_delegate
        );
    }

    'running: loop {
        window.draw_screen(screen_manager.active_screen())?;

        let event = event_source.wait_event()?;
        if matches!(event, Event::Quit) {
            break 'running;
        }

        let mut handlers: [&mut dyn EventHandler; _] = [
            &mut storage,
            &mut state_manager,
            &mut schedule,
            &mut backplate,
            &mut timers,
            &mut sound,
            &mut window,
            &mut screen_manager,
            &mut home_assistant
        ];

        let mut event = Some(event);
        while let Some(e) = event {
            info!("{:?}", e);

            for handler in handlers.iter_mut() {
                handler.handle_event(&e)?;
            }

            event = event_source.poll_event()?;
        }
    }

    Ok(())
}

fn init_syslog(log_level: log::LevelFilter) -> Result<()> {
    use syslog::{Facility, Formatter3164, BasicLogger};

    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: env::get_pkg_name().into(),
        pid: 0
    };

    let logger = syslog::unix(formatter)?;
    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(log_level))?;

    Ok(())
}

fn install_panic_logging() {
    use std::{panic, thread};

    panic::set_hook(Box::new(|info| {
        let thread = thread::current();
        let thread = thread.name().unwrap_or("<unnamed>");

        let reason = info.payload_as_str().unwrap_or("unknown");
        error!("Panic; thread:{thread} reason:{reason}");

        if let Some(loc) = info.location() {
            // error!("Location; {}:{}", loc.file(), loc.line());
            error!("Location; {}", loc);
        }
    }));
}
