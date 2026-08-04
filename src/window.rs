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

use anyhow::Result;

mod backlight_options;
pub use backlight_options::BacklightOptions;

#[cfg(feature = "device")]
mod backlight;
#[cfg(feature = "device")]
mod window_linuxfb;

#[cfg(feature = "device")]
pub fn new_window(options: &BacklightOptions) -> Result<window_linuxfb::FramebufferWindow> {
    window_linuxfb::FramebufferWindow::new(options)
}

#[cfg(feature = "device")]
pub fn new_event_source() -> Result<crate::events::DefaultEventSource> {
    Ok(crate::events::DefaultEventSource::new())
}

#[cfg(feature = "simulate")]
mod window_sdl;

#[cfg(feature = "simulate")]
pub fn new_window(_options: &BacklightOptions) -> Result<window_sdl::SdlWindow> {
    window_sdl::SdlWindow::new()
}

#[cfg(feature = "simulate")]
pub fn new_event_source() -> Result<window_sdl::SdlEventSource> {
    window_sdl::SdlEventSource::new()
}
