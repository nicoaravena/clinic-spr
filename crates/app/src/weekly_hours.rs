//! A practitioner's weekly hours in their clinic's time zone, and the dated UTC shifts they make.
//!
//! The domain's [`WorkingHours`] only knows UTC. A clinic thinks in a week of local times, such as
//! "Monday to Friday, 09:00-13:00, Europe/Madrid". [`WeeklyHours::working_hours`] turns one into
//! the other for a range of dates, reading each date with the offset in force on it.
//!
//! Daylight saving makes some local times special, and the rules are:
//!
//! - A local time the clocks skip (02:30 when they jump from 02:00 to 03:00) becomes the instant
//!   they jump, so a shift starting then starts when the clocks show 03:00.
//! - A local time the clocks repeat (02:30 when they go back from 03:00 to 02:00) means its first
//!   occurrence.
//!
//! Neither rule ever moves a later local time to an earlier instant, so expanded shifts stay in
//! order. A shift that lies entirely in a skipped hour has no time left and is dropped for that
//! date. Two shifts separated only by a skipped hour end up touching, and are merged into one.
//!
//! The fields are private, so the only way to build weekly hours is through [`WeeklyHours::new`]:
//!
//! ```compile_fail,E0451
//! use clinic_app::weekly_hours::WeeklyHours;
//!
//! WeeklyHours {
//!     time_zone: chrono_tz::Europe::Madrid,
//!     shifts: Vec::new(),
//! };
//! ```

use chrono::{
    DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeDelta,
    TimeZone, Utc, Weekday,
};
use chrono_tz::Tz;
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;
use thiserror::Error;

/// A span of local time worked every week on one weekday, such as Monday 09:00-13:00.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeeklyShift {
    weekday: Weekday,
    start: NaiveTime,
    end: NaiveTime,
}

impl WeeklyShift {
    /// Fails unless `start` is strictly before `end`, so a shift can't cross midnight.
    pub fn new(
        weekday: Weekday,
        start: NaiveTime,
        end: NaiveTime,
    ) -> Result<Self, WeeklyHoursError> {
        if start >= end {
            return Err(WeeklyHoursError::EndNotAfterStart);
        }

        Ok(Self {
            weekday,
            start,
            end,
        })
    }

    /// The day of the week the shift is worked.
    pub fn weekday(&self) -> Weekday {
        self.weekday
    }

    /// When the shift starts, in local time (inclusive).
    pub fn start(&self) -> NaiveTime {
        self.start
    }

    /// When the shift ends, in local time (exclusive).
    pub fn end(&self) -> NaiveTime {
        self.end
    }
}

/// The shifts one practitioner works each week, in a time zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklyHours {
    time_zone: Tz,
    shifts: Vec<WeeklyShift>,
}

impl WeeklyHours {
    /// Sorts the shifts by weekday, from Monday, then by start. Fails if two shifts on the same
    /// weekday overlap or touch, for the same reason [`WorkingHours::new`] does.
    ///
    /// An empty list is allowed and expands to no shifts.
    pub fn new(time_zone: Tz, mut shifts: Vec<WeeklyShift>) -> Result<Self, WeeklyHoursError> {
        shifts.sort_by_key(|shift| (shift.weekday.num_days_from_monday(), shift.start));

        if let Some(pair) = shifts
            .windows(2)
            .find(|pair| pair[0].weekday == pair[1].weekday && pair[1].start <= pair[0].end)
        {
            return Err(WeeklyHoursError::ShiftsNotSeparated {
                weekday: pair[0].weekday,
            });
        }

        Ok(Self { time_zone, shifts })
    }

    /// The time zone the shifts are written in.
    pub fn time_zone(&self) -> Tz {
        self.time_zone
    }

    /// The shifts, sorted by weekday from Monday, then by start.
    pub fn shifts(&self) -> &[WeeklyShift] {
        &self.shifts
    }

    /// The shifts worked on the local dates from `from` up to, but not including, `until`, as UTC
    /// working hours. A range that is empty or reversed gives no shifts.
    ///
    /// The module documentation describes how skipped and repeated local times are handled.
    pub fn working_hours(&self, from: NaiveDate, until: NaiveDate) -> WorkingHours {
        let mut shifts: Vec<TimeSlot> = Vec::new();

        for date in from.iter_days().take_while(|date| *date < until) {
            let worked = self
                .shifts
                .iter()
                .filter(|shift| shift.weekday == date.weekday());

            for shift in worked {
                let start = to_utc(self.time_zone, date.and_time(shift.start));
                let end = to_utc(self.time_zone, date.and_time(shift.end));

                // The whole shift lies in a skipped hour.
                if start == end {
                    continue;
                }

                // A later local time never maps to an earlier instant, and template shifts are
                // separated, so `start` is never before the previous end and `end` is after
                // `start`. The `expect`s below can't fire.
                match shifts.last_mut() {
                    Some(previous) if previous.end() == start => {
                        *previous = TimeSlot::new(previous.start(), end)
                            .expect("a shift's end is after its start");
                    }
                    _ => shifts
                        .push(TimeSlot::new(start, end).expect("a shift's end is after its start")),
                }
            }
        }

        WorkingHours::new(shifts).expect("expanded shifts are sorted and separated")
    }
}

/// The instant a clock in `time_zone` reads `local`, by the rules in the module documentation.
fn to_utc(time_zone: Tz, local: NaiveDateTime) -> DateTime<Utc> {
    match time_zone.from_local_datetime(&local) {
        LocalResult::Single(instant) => instant.with_timezone(&Utc),
        LocalResult::Ambiguous(earliest, _latest) => earliest.with_timezone(&Utc),
        LocalResult::None => gap_end(time_zone, local),
    }
}

/// The instant the clocks jump past the skipped local time `local`: the first instant at which a
/// clock in `time_zone` reads `local` or later.
fn gap_end(time_zone: Tz, local: NaiveDateTime) -> DateTime<Utc> {
    // The offsets in force a day either side of the gap. Reading `local` with each of them gives
    // two instants, and the jump lies between them: as many seconds apart as the gap is long.
    let offset_at = |naive: NaiveDateTime| {
        time_zone
            .offset_from_utc_datetime(&naive)
            .fix()
            .local_minus_utc()
    };
    let before = offset_at(local - TimeDelta::days(1));
    let after = offset_at(local + TimeDelta::days(1));
    let earliest = local.and_utc() - TimeDelta::seconds(after.into());
    let latest = local.and_utc() - TimeDelta::seconds(before.into());

    std::iter::successors(Some(earliest), |instant| {
        Some(*instant + TimeDelta::seconds(1))
    })
    .take_while(|instant| *instant <= latest)
    .find(|instant| instant.with_timezone(&time_zone).naive_local() >= local)
    // `latest` reads `local` plus the gap's length, so the search only falls through if the
    // offsets a day either side don't describe the gap. Then `latest` is the best guess.
    .unwrap_or(latest)
}

/// Why [`WeeklyShift::new`] or [`WeeklyHours::new`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum WeeklyHoursError {
    /// The shift's end was equal to or before its start. Shifts that cross midnight aren't
    /// supported.
    #[error("weekly shift must end after it starts, on the same day")]
    EndNotAfterStart,
    /// Two shifts on the same weekday overlap, or one ends exactly when the next starts.
    #[error("shifts on {weekday} must not overlap or touch")]
    ShiftsNotSeparated {
        /// The weekday with the clashing shifts.
        weekday: Weekday,
    },
}
