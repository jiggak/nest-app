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

use std::time::Instant;

use anyhow::Result;
use esphome_api::proto::{
    ClimateAction, ClimateFanMode, ClimateMode, ClimatePreset, ClimateStateResponse
};
use serde::{Deserialize, Serialize};

use crate::{
    config::{ClimateSettings, Config},
    events::{Event, EventHandler, EventSender},
    timer::TimerId
};

#[derive(Debug, Clone)]
pub struct ThermostatState {
    pub target_temp: f32,
    pub current_temp: f32,
    pub mode: HvacMode,
    pub action: HvacAction,
    pub preset: Option<PresetName>,
    pub lockout: bool,
    /// Backplate connected flag
    pub backplate: bool,
}

impl ThermostatState {
    pub const MIN_TEMP: f32 = 9.0;
    pub const MAX_TEMP: f32 = 32.0;

    pub fn temp_percent(temp: f32) -> f32 {
        (temp - Self::MIN_TEMP) / (Self::MAX_TEMP - Self::MIN_TEMP)
    }

    /// Attempt to set target temp and return `true` if successful.
    /// Return `false` if value is outside of min/max range, or if value
    /// equals current target temp.
    pub fn set_target_temp(&mut self, val: f32) -> bool {
        if val > Self::MIN_TEMP && val < Self::MAX_TEMP && val != self.target_temp {
            self.target_temp = val;
            true
        } else {
            false
        }
    }

    fn to_ha_state(&self) -> ClimateStateResponse {
        let mut state = ClimateStateResponse::default();
        state.set_fan_mode(ClimateFanMode::ClimateFanAuto);

        state.set_action(self.action.into());
        state.set_mode(self.mode.into());
        state.current_temperature = self.current_temp;
        state.target_temperature = self.target_temp;
        state.preset = self.preset.map(|p| p.into())
            .unwrap_or(ClimatePreset::None) as i32;

        state
    }

    pub fn is_away(&self) -> bool {
        self.preset == Some(PresetName::Away)
    }
}

impl Default for ThermostatState {
    fn default() -> Self {
        Self {
            target_temp: 19.5,
            current_temp: 20.0,
            action: HvacAction::Idle,
            mode: HvacMode::Heat,
            preset: None,
            lockout: false,
            backplate: false,
        }
    }
}

impl From<ThermostatState> for ClimateStateResponse {
    fn from(value: ThermostatState) -> Self {
        value.to_ha_state()
    }
}

impl From<&ThermostatState> for ClimateStateResponse {
    fn from(value: &ThermostatState) -> Self {
        value.to_ha_state()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub enum HvacMode {
    Off,
    Heat,
    Cool,
    Fan,
}

impl TryFrom<ClimateMode> for HvacMode {
    type Error = anyhow::Error;

    fn try_from(value: ClimateMode) -> anyhow::Result<Self> {
        Ok(match value {
            ClimateMode::Off => Self::Off,
            ClimateMode::Heat => Self::Heat,
            ClimateMode::Cool => Self::Cool,
            ClimateMode::FanOnly => Self::Fan,
            v => return Err(anyhow::anyhow!("Unsupported climate mode {v:?}"))
        })
    }
}

impl From<HvacMode> for ClimateMode {
    fn from(value: HvacMode) -> Self {
        match value {
            HvacMode::Off => Self::Off,
            HvacMode::Heat => Self::Heat,
            HvacMode::Cool => Self::Cool,
            HvacMode::Fan => Self::FanOnly,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub enum HvacAction {
    Idle,
    Heating,
    Cooling,
    Fan,
}

impl From<HvacAction> for ClimateAction {
    fn from(value: HvacAction) -> Self {
        match value {
            HvacAction::Idle => Self::Idle,
            HvacAction::Heating => Self::Heating,
            HvacAction::Cooling => Self::Cooling,
            HvacAction::Fan => Self::Fan,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub enum PresetName {
    Home,
    Sleep,
    Away,
}

impl TryFrom<ClimatePreset> for PresetName {
    type Error = anyhow::Error;

    fn try_from(value: ClimatePreset) -> anyhow::Result<Self> {
        Ok(match value {
            ClimatePreset::Home => Self::Home,
            ClimatePreset::Sleep => Self::Sleep,
            ClimatePreset::Away => Self::Away,
            v => return Err(anyhow::anyhow!("Unsupported climate preset {v:?}"))
        })
    }
}

impl From<PresetName> for ClimatePreset {
    fn from(value: PresetName) -> Self {
        match value {
            PresetName::Home => Self::Home,
            PresetName::Sleep => Self::Sleep,
            PresetName::Away => Self::Away,
        }
    }
}

enum SavedTemp {
    TargetTemp(f32),
    Preset(PresetName),
}

impl From<&ThermostatState> for SavedTemp {
    fn from(value: &ThermostatState) -> Self {
        if let Some(preset) = value.preset {
            SavedTemp::Preset(preset)
        } else {
            SavedTemp::TargetTemp(value.target_temp)
        }
    }
}

pub struct StateManager<S: EventSender> {
    event_sender: S,
    state: ThermostatState,
    config: Config,
    climate_settings: ClimateSettings,
    restore_temp: Option<SavedTemp>,
    restore_mode: Option<HvacMode>,
    last_idle_time: Instant,
}

impl<S: EventSender> StateManager<S> {
    pub fn new(
        config: &Config,
        climate_settings: &ClimateSettings,
        state: ThermostatState,
        event_sender: S
    ) -> Result<Self> {
        if let Some(away_mode) = &climate_settings.away {
            event_sender.send_event(
                Event::TimeoutReset(TimerId::Away, away_mode.timeout)
            )?;
        }
        event_sender.send_event(
            Event::TimeoutReset(TimerId::Backlight, config.backlight.timeout)
        )?;

        Ok(Self {
            event_sender,
            state,
            config: config.clone(),
            climate_settings: climate_settings.clone(),
            restore_temp: None,
            restore_mode: None,
            last_idle_time: Instant::now(),
        })
    }

    fn set_target_temp(&mut self, temp: f32) -> bool {
        let temp = (temp * 10.0).round() / 10.0;
        if temp != self.state.target_temp {
            self.state.target_temp = temp;
            true
        } else {
            false
        }
    }

    fn set_current_temp(&mut self, temp: f32) -> bool {
        let temp = (temp * 10.0).round() / 10.0;
        if temp != self.state.current_temp {
            self.state.current_temp = temp;
            true
        } else {
            false
        }
    }

    fn set_mode(&mut self, mode: HvacMode) -> Result<bool> {
        if mode != self.state.mode {
            // switching from fan mode to some other mode
            if self.state.mode == HvacMode::Fan {
                self.event_sender.send_event(Event::CancelTimer(TimerId::Fan))?;
                self.restore_mode = None;
            }
            // switching from some other mode to fan mode
            if mode == HvacMode::Fan {
                self.event_sender.send_event(
                    Event::StartTickTimer(TimerId::Fan, self.config.default_fan_timeout)
                )?;
                self.restore_mode = Some(self.state.mode);
            }

            // Clear action when switching modes to avoid action previous action
            // persisting due to current temp being inside hysteresis band.
            self.state.action = HvacAction::Idle;

            self.state.mode = mode;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn away_start(&mut self) -> bool {
        if self.state.preset != Some(PresetName::Away) {
            self.restore_temp = Some(SavedTemp::from(&self.state));
            self.set_preset(Some(PresetName::Away))
        } else {
            false
        }
    }

    fn away_stop(&mut self) -> bool {
        if self.state.preset == Some(PresetName::Away) {
            match self.restore_temp {
                Some(SavedTemp::Preset(preset)) => {
                    self.restore_temp = None;
                    self.set_preset(Some(preset))
                }
                Some(SavedTemp::TargetTemp(temp)) => {
                    self.restore_temp = None;
                    self.set_preset(None);
                    self.set_target_temp(temp)
                }
                None => false
            }
        } else {
            false
        }
    }

    fn set_preset(&mut self, preset: Option<PresetName>) -> bool {
        if preset != self.state.preset {
            self.state.preset = preset;

            if let Some(preset) = &preset {
                if let Some(temp) = self.climate_settings.get_preset_temp(preset, &self.state.mode) {
                    self.state.target_temp = temp;
                } else {
                    log::warn!("Missing target temp for preset {preset:?}");
                }
            }

            true
        } else {
            false
        }
    }

    fn set_schedule_preset(&mut self, preset: PresetName) -> bool {
        if self.state.preset == Some(PresetName::Away) {
            self.restore_temp = Some(SavedTemp::Preset(preset));
            false
        } else {
            self.set_preset(Some(preset))
        }
    }

    fn apply_hvac_action(&mut self) -> bool {
        let old_action = self.state.action;

        if !self.state.backplate {
            self.state.action = HvacAction::Idle;
            return old_action != self.state.action;
        }

        let current_temp = self.state.current_temp;

        match self.state.mode {
            HvacMode::Heat => {
                let target_temp_hi = self.state.target_temp + self.config.temp_overrun;
                let target_temp_lo = self.state.target_temp - self.config.temp_deadband;

                if current_temp <= target_temp_lo {
                    self.state.action = HvacAction::Heating;
                } else if current_temp >= target_temp_hi {
                    self.state.action = HvacAction::Idle;
                }
            }
            HvacMode::Cool => {
                let target_temp_hi = self.state.target_temp + self.config.temp_deadband;
                let target_temp_lo = self.state.target_temp - self.config.temp_overrun;

                if current_temp >= target_temp_hi {
                    self.state.action = HvacAction::Cooling;
                } else if current_temp <= target_temp_lo {
                    self.state.action = HvacAction::Idle;
                }
            }
            HvacMode::Fan => {
                self.state.action = HvacAction::Fan;
            }
            HvacMode::Off => {
                self.state.action = HvacAction::Idle;
            }
        };

        old_action != self.state.action
    }

    fn apply_lockout(&mut self) -> Result<()> {
        if self.state.action == HvacAction::Idle {
            // don't reset last idle time until min idle time elapsed
            // i.e. don't re-trigger lockout when it's already active
            if self.last_idle_time.elapsed() > self.config.min_off_time {
                self.last_idle_time = Instant::now();
            }

            self.state.lockout = false;
        } else {
            if self.last_idle_time.elapsed() < self.config.min_off_time {
                let lockout_time = self.config.min_off_time - self.last_idle_time.elapsed();
                self.state.lockout = true;
                self.event_sender.send_event(
                    Event::StartTickTimer(TimerId::HvacLockout, lockout_time)
                )?;
            } else {
                self.state.lockout = false;
            }
        }

        Ok(())
    }
}

impl<S: EventSender> EventHandler for StateManager<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        let did_change = match event {
            Event::SetMode(mode) => {
                self.set_mode(*mode)?
            }
            Event::SetTargetTemp(temp) => {
                self.set_preset(None);
                self.set_target_temp(*temp)
            }
            Event::SetCurrentTemp(temp) => {
                self.set_current_temp(*temp)
            }
            Event::ProximityNear | Event::ProximityFar | Event::Dial(_) => {
                if let Some(away_mode) = &self.climate_settings.away {
                    self.event_sender.send_event(
                        Event::TimeoutReset(TimerId::Away, away_mode.timeout)
                    )?;
                }
                self.away_stop()
            }
            Event::SetPreset(Some(PresetName::Away)) | Event::TimeoutReached(TimerId::Away) => {
                self.away_start()
            }
            Event::SetPreset(preset) => {
                self.set_preset(*preset)
            }
            Event::SchedulePreset(preset) => {
                self.set_schedule_preset(*preset)
            }
            Event::TimeoutReached(TimerId::HvacLockout) => {
                self.state.lockout = false;
                true
            }
            Event::TimeoutReached(TimerId::Fan) => {
                let mode = self.restore_mode.unwrap_or(HvacMode::Off);
                self.set_mode(mode)?
            }
            Event::BackplateConnected => {
                self.state.backplate = true;
                true
            }
            Event::BackplateDisconnected => {
                self.state.backplate = false;
                true
            }
            _ => false
        };

        if did_change {
            if self.apply_hvac_action() {
                self.apply_lockout()?;
            }

            self.event_sender.send_event(Event::State(self.state.clone()))?;
        }

        if event.is_wakeup_event() {
            self.event_sender.send_event(
                Event::TimeoutReset(TimerId::Backlight, self.config.backlight.timeout)
            )?;
        }

        if event == &Event::GetState {
            self.event_sender.send_event(Event::State(self.state.clone()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc::Sender, time::Duration};

    use super::*;
    use crate::events::{DefaultEventSource, EventSource};

    fn state_manager(
        state: ThermostatState
    ) -> (DefaultEventSource, StateManager<Sender<Event>>)
    {
        let mut config = Config::default();
        let climate_settings = ClimateSettings::default();
        config.temp_deadband = 0.4;
        config.temp_overrun = 0.2;

        let event_source = DefaultEventSource::new();
        let state_manager = StateManager::new(
            &config,
            &climate_settings,
            state,
            event_source.event_sender()
        ).unwrap();

        (event_source, state_manager)
    }

    fn simulate<S>(
        mut state: StateManager<S>,
        steps: &[(f32, HvacAction)]
    ) -> Result<()>
        where S: EventSender
    {
        for (temp, action) in steps {
            state.handle_event(&Event::SetCurrentTemp(*temp))?;
            assert_eq!(
                state.state.action,
                *action,
                "temp {} expected {:?}, found {:?}",
                temp, *action, state.state.action
            );
        }

        Ok(())
    }

    #[test]
    fn temp_hysteresis_heat_on() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Heat,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Idle,
            backplate: true,
            ..ThermostatState::default()
        };

        let (_x, mgr) = state_manager(state);

        simulate(mgr, &[
            (20.0, HvacAction::Idle),
            (19.9, HvacAction::Idle),
            (19.8, HvacAction::Idle),
            (19.7, HvacAction::Idle),
            (19.6, HvacAction::Heating)
        ])
    }

    #[test]
    fn temp_hysteresis_heat_off() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Heat,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Heating,
            backplate: true,
            ..ThermostatState::default()
        };

        let (_x, mgr) = state_manager(state);

        simulate(mgr, &[
            (20.0, HvacAction::Heating),
            (20.1, HvacAction::Heating),
            (20.2, HvacAction::Idle)
        ])
    }

    #[test]
    fn temp_hysteresis_cool_on() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Cool,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Idle,
            backplate: true,
            ..ThermostatState::default()
        };

        let (_x, mgr) = state_manager(state);

        simulate(mgr, &[
            (20.0, HvacAction::Idle),
            (20.1, HvacAction::Idle),
            (20.2, HvacAction::Idle),
            (20.3, HvacAction::Idle),
            (20.4, HvacAction::Cooling)
        ])
    }

    #[test]
    fn temp_hysteresis_cool_off() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Cool,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Cooling,
            backplate: true,
            ..ThermostatState::default()
        };

        let (_x, mgr) = state_manager(state);

        simulate(mgr, &[
            (20.0, HvacAction::Cooling),
            (19.9, HvacAction::Cooling),
            (19.8, HvacAction::Idle)
        ])
    }

    #[test]
    fn min_off_time() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Cool,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Idle,
            backplate: true,
            ..ThermostatState::default()
        };

        let (_x, mut mgr) = state_manager(state);

        // idle -> cooling = lockout
        mgr.handle_event(&Event::SetCurrentTemp(21.0))?;
        assert!(mgr.state.action == HvacAction::Cooling);
        assert!(mgr.state.lockout);

        // lockout timer elapsed = no lockout
        mgr.handle_event(&Event::TimeoutReached(TimerId::HvacLockout))?;
        assert!(mgr.state.action == HvacAction::Cooling);
        assert!(!mgr.state.lockout);

        // cooling -> idle = no lockout
        mgr.handle_event(&Event::SetCurrentTemp(19.0))?;
        assert!(mgr.state.action == HvacAction::Idle);
        assert!(!mgr.state.lockout);

        // idle -> cooling = lockout
        mgr.handle_event(&Event::SetCurrentTemp(21.0))?;
        assert!(mgr.state.action == HvacAction::Cooling);
        assert!(mgr.state.lockout);

        // cooling -> idle = no lockout
        mgr.handle_event(&Event::SetCurrentTemp(19.0))?;
        assert!(mgr.state.action == HvacAction::Idle);
        assert!(!mgr.state.lockout);

        // idle -> long delay -> cooling = no lockout
        mgr.last_idle_time = Instant::now() - Duration::from_mins(10);
        mgr.handle_event(&Event::SetCurrentTemp(21.0))?;
        assert!(mgr.state.action == HvacAction::Cooling);
        assert!(!mgr.state.lockout);

        Ok(())
    }

    #[test]
    fn transition_idle() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Cool,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Idle,
            backplate: true,
            ..ThermostatState::default()
        };

        let (_x, mut mgr) = state_manager(state);

        // Begin cooling
        mgr.handle_event(&Event::SetCurrentTemp(21.0))?;
        assert!(mgr.state.action == HvacAction::Cooling);

        // Temp decreased inside hysteresis range, still cooling
        mgr.handle_event(&Event::SetCurrentTemp(20.0))?;
        assert!(mgr.state.action == HvacAction::Cooling);

        // Switch mode to heat, current temp within target temp, go idle
        mgr.handle_event(&Event::SetMode(HvacMode::Heat))?;
        assert!(mgr.state.action == HvacAction::Idle);

        // Begin heating
        mgr.handle_event(&Event::SetCurrentTemp(19.0))?;
        assert!(mgr.state.action == HvacAction::Heating);

        // Temp decreased inside hysteresis range, still heating
        mgr.handle_event(&Event::SetCurrentTemp(20.0))?;
        assert!(mgr.state.action == HvacAction::Heating);

        // Switch mode to cool, current temp within target temp, go idle
        mgr.handle_event(&Event::SetMode(HvacMode::Cool))?;
        assert!(mgr.state.action == HvacAction::Idle);

        Ok(())
    }
}
