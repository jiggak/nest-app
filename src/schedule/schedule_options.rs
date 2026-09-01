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

use chrono::{NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::{config::config_de, state::PresetName};

/// Schedule
///
/// ```toml
/// [[schedule]]
/// days_of_week = "EveryDay"
/// set_points = [
///    { time = "08:00", preset = "Home" },
///    { time = "22:00", preset = "Sleep" },
/// ]
/// ```
///
/// You can define more than one schedule entry, and it will overlap the
/// previous. In the example below, the perset will be change to "Home"
/// at 8am everyday, and change to "Work" at 9am Monday and Wednsday.
///
/// ```toml
/// [[schedule]]
/// days_of_week = "EveryDay"
/// set_points = [
///    { time = "08:00", preset = "Home" }
/// ]
///
/// [[schedule]]
/// days_of_week = ["Monday", "Wednsday"]
/// set_points = [
///    { time = "09:00", preset = "Work" }
/// ]
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScheduleEntry {
    /// Days of the week.
    ///
    /// One of "EveryDay", "WeekDays", "WeekEnd"
    ///
    /// Or...
    ///
    /// List of weekdays ["Monday", "Tuesday", ...]
    pub days_of_week: DaysOfWeek,

    /// List of set points with time of day and name of preset
    pub set_points: Vec<SetPoint>
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum DaysOfWeek {
    Range(WeekDayRange),
    List(Vec<WeekDay>)
}

impl DaysOfWeek {
    pub fn normalize(&self) -> Vec<Weekday> {
        match self {
            DaysOfWeek::List(days) => days.iter()
                .map(|d| d.to_chrono())
                .collect(),
            DaysOfWeek::Range(range) => {
                match range {
                    WeekDayRange::EveryDay => vec![
                        Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
                    ],
                    WeekDayRange::WeekDays => vec![
                        Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri
                    ],
                    WeekDayRange::WeekEnd => vec![
                        Weekday::Sat, Weekday::Sun
                    ]
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WeekDayRange {
    EveryDay,
    WeekDays,
    WeekEnd
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WeekDay {
    Mon,
    Tue,
    Wed,
    Thur,
    Fri,
    Sat,
    Sun
}

impl WeekDay {
    pub fn to_chrono(&self) -> Weekday {
        match self {
            Self::Mon => Weekday::Mon,
            Self::Tue => Weekday::Tue,
            Self::Wed => Weekday::Wed,
            Self::Thur => Weekday::Thu,
            Self::Fri => Weekday::Fri,
            Self::Sat => Weekday::Sat,
            Self::Sun => Weekday::Sun
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SetPoint {
    #[serde(with = "config_de::time_of_day")]
    pub time: NaiveTime,
    pub preset: PresetName
}
