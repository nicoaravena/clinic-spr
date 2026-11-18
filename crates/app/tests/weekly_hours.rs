use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc, Weekday};
use chrono_tz::America::Santiago;
use chrono_tz::Australia::Lord_Howe;
use chrono_tz::Europe::Madrid;
use chrono_tz::Tz;
use clinic_app::weekly_hours::{WeeklyHours, WeeklyHoursError, WeeklyShift};
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::PatientId;
use clinic_domain::schedule::{BookingError, Schedule};
use clinic_domain::time_slot::TimeSlot;

fn time(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .unwrap()
}

fn slot(start: DateTime<Utc>, end: DateTime<Utc>) -> TimeSlot {
    TimeSlot::new(start, end).unwrap()
}

fn shift(weekday: Weekday, start: (u32, u32), end: (u32, u32)) -> WeeklyShift {
    WeeklyShift::new(weekday, time(start.0, start.1), time(end.0, end.1)).unwrap()
}

fn hours(time_zone: Tz, shifts: Vec<WeeklyShift>) -> WeeklyHours {
    WeeklyHours::new(time_zone, shifts).unwrap()
}

/// The UTC shifts `hours` expands to on the local dates in `[from, until)`.
fn expand(hours: &WeeklyHours, from: NaiveDate, until: NaiveDate) -> Vec<TimeSlot> {
    hours.working_hours(from, until).shifts().to_vec()
}

/// Monday to Friday, 09:00-13:00.
fn weekday_mornings(time_zone: Tz) -> WeeklyHours {
    let days = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ];
    hours(
        time_zone,
        days.map(|day| shift(day, (9, 0), (13, 0))).to_vec(),
    )
}

#[test]
fn a_shift_keeps_its_weekday_and_times() {
    let shift = shift(Weekday::Mon, (9, 0), (13, 0));

    assert_eq!(shift.weekday(), Weekday::Mon);
    assert_eq!(shift.start(), time(9, 0));
    assert_eq!(shift.end(), time(13, 0));
}

#[test]
fn a_shift_must_end_after_it_starts() {
    assert_eq!(
        WeeklyShift::new(Weekday::Mon, time(9, 0), time(9, 0)),
        Err(WeeklyHoursError::EndNotAfterStart),
    );
    assert_eq!(
        WeeklyShift::new(Weekday::Mon, time(22, 0), time(6, 0)),
        Err(WeeklyHoursError::EndNotAfterStart),
    );
}

#[test]
fn shifts_on_the_same_weekday_must_not_overlap() {
    let shifts = vec![
        shift(Weekday::Mon, (9, 0), (13, 0)),
        shift(Weekday::Mon, (12, 0), (14, 0)),
    ];

    assert_eq!(
        WeeklyHours::new(Madrid, shifts),
        Err(WeeklyHoursError::ShiftsNotSeparated {
            weekday: Weekday::Mon
        }),
    );
}

#[test]
fn shifts_on_the_same_weekday_must_not_touch() {
    let shifts = vec![
        shift(Weekday::Tue, (13, 0), (17, 0)),
        shift(Weekday::Tue, (9, 0), (13, 0)),
    ];

    assert_eq!(
        WeeklyHours::new(Madrid, shifts),
        Err(WeeklyHoursError::ShiftsNotSeparated {
            weekday: Weekday::Tue
        }),
    );
}

#[test]
fn the_same_times_on_different_weekdays_are_allowed() {
    let shifts = vec![
        shift(Weekday::Mon, (9, 0), (13, 0)),
        shift(Weekday::Tue, (9, 0), (13, 0)),
    ];

    assert!(WeeklyHours::new(Madrid, shifts).is_ok());
}

#[test]
fn shifts_are_stored_sorted_by_weekday_from_monday_then_start() {
    let hours = hours(
        Madrid,
        vec![
            shift(Weekday::Sun, (1, 0), (2, 0)),
            shift(Weekday::Tue, (9, 0), (10, 0)),
            shift(Weekday::Mon, (14, 0), (15, 0)),
            shift(Weekday::Mon, (9, 0), (10, 0)),
        ],
    );

    assert_eq!(
        hours.shifts(),
        [
            shift(Weekday::Mon, (9, 0), (10, 0)),
            shift(Weekday::Mon, (14, 0), (15, 0)),
            shift(Weekday::Tue, (9, 0), (10, 0)),
            shift(Weekday::Sun, (1, 0), (2, 0)),
        ],
    );
    assert_eq!(hours.time_zone(), Madrid);
}

#[test]
fn no_shifts_are_allowed_and_give_no_working_hours() {
    let hours = hours(Madrid, Vec::new());

    assert_eq!(expand(&hours, date(2026, 3, 2), date(2026, 3, 9)), []);
}

#[test]
fn an_empty_or_reversed_range_gives_no_shifts() {
    let hours = weekday_mornings(Madrid);

    assert_eq!(expand(&hours, date(2026, 3, 2), date(2026, 3, 2)), []);
    assert_eq!(expand(&hours, date(2026, 3, 9), date(2026, 3, 2)), []);
}

#[test]
fn the_range_includes_its_first_date_and_excludes_its_last() {
    let hours = weekday_mornings(Madrid);

    // Madrid is one hour ahead of UTC in winter.
    assert_eq!(
        expand(&hours, date(2026, 3, 2), date(2026, 3, 4)),
        [
            slot(utc(2026, 3, 2, 8, 0), utc(2026, 3, 2, 12, 0)),
            slot(utc(2026, 3, 3, 8, 0), utc(2026, 3, 3, 12, 0)),
        ],
    );
}

#[test]
fn only_dates_on_a_template_weekday_get_shifts() {
    let hours = hours(Madrid, vec![shift(Weekday::Wed, (9, 0), (13, 0))]);

    assert_eq!(
        expand(&hours, date(2026, 3, 2), date(2026, 3, 9)),
        [slot(utc(2026, 3, 4, 8, 0), utc(2026, 3, 4, 12, 0))],
    );
}

#[test]
fn the_same_local_hours_move_in_utc_when_the_clocks_change() {
    let hours = hours(Madrid, vec![shift(Weekday::Mon, (9, 0), (13, 0))]);

    // Madrid moves from UTC+1 to UTC+2 on Sunday 2026-03-29.
    assert_eq!(
        expand(&hours, date(2026, 3, 23), date(2026, 4, 6)),
        [
            slot(utc(2026, 3, 23, 8, 0), utc(2026, 3, 23, 12, 0)),
            slot(utc(2026, 3, 30, 7, 0), utc(2026, 3, 30, 11, 0)),
        ],
    );
}

#[test]
fn santiago_in_march_is_three_hours_behind_utc() {
    let hours = weekday_mornings(Santiago);

    assert_eq!(
        expand(&hours, date(2026, 3, 2), date(2026, 3, 3)),
        [slot(utc(2026, 3, 2, 12, 0), utc(2026, 3, 2, 16, 0))],
    );
}

// On Sunday 2026-03-29 Madrid's clocks jump from 02:00 to 03:00, at 01:00 UTC.

#[test]
fn a_shift_starting_in_a_skipped_hour_starts_when_the_clocks_jump() {
    let hours = hours(Madrid, vec![shift(Weekday::Sun, (2, 30), (5, 0))]);

    assert_eq!(
        expand(&hours, date(2026, 3, 29), date(2026, 3, 30)),
        [slot(utc(2026, 3, 29, 1, 0), utc(2026, 3, 29, 3, 0))],
    );
}

#[test]
fn a_shift_ending_in_a_skipped_hour_ends_when_the_clocks_jump() {
    let hours = hours(Madrid, vec![shift(Weekday::Sun, (0, 30), (2, 30))]);

    assert_eq!(
        expand(&hours, date(2026, 3, 29), date(2026, 3, 30)),
        [slot(utc(2026, 3, 28, 23, 30), utc(2026, 3, 29, 1, 0))],
    );
}

#[test]
fn a_shift_inside_a_skipped_hour_is_dropped_that_day_only() {
    let hours = hours(Madrid, vec![shift(Weekday::Sun, (2, 10), (2, 50))]);

    assert_eq!(
        expand(&hours, date(2026, 3, 22), date(2026, 4, 6)),
        [
            slot(utc(2026, 3, 22, 1, 10), utc(2026, 3, 22, 1, 50)),
            slot(utc(2026, 4, 5, 0, 10), utc(2026, 4, 5, 0, 50)),
        ],
    );
}

#[test]
fn shifts_separated_only_by_a_skipped_hour_are_merged() {
    let hours = hours(
        Madrid,
        vec![
            shift(Weekday::Sun, (1, 0), (2, 15)),
            shift(Weekday::Sun, (2, 45), (4, 0)),
        ],
    );

    assert_eq!(
        expand(&hours, date(2026, 3, 29), date(2026, 3, 30)),
        [slot(utc(2026, 3, 29, 0, 0), utc(2026, 3, 29, 2, 0))],
    );
}

// On Sunday 2026-10-25 Madrid's clocks go back from 03:00 to 02:00, at 01:00 UTC, so 02:00-03:00
// happens twice: first at UTC+2, then at UTC+1.

#[test]
fn a_shift_ending_in_a_repeated_hour_ends_at_its_first_occurrence() {
    let hours = hours(Madrid, vec![shift(Weekday::Sun, (0, 0), (2, 30))]);

    assert_eq!(
        expand(&hours, date(2026, 10, 25), date(2026, 10, 26)),
        [slot(utc(2026, 10, 24, 22, 0), utc(2026, 10, 25, 0, 30))],
    );
}

#[test]
fn a_shift_starting_in_a_repeated_hour_starts_at_its_first_occurrence() {
    let hours = hours(Madrid, vec![shift(Weekday::Sun, (2, 30), (4, 0))]);

    assert_eq!(
        expand(&hours, date(2026, 10, 25), date(2026, 10, 26)),
        [slot(utc(2026, 10, 25, 0, 30), utc(2026, 10, 25, 3, 0))],
    );
}

#[test]
fn in_santiago_a_skipped_midnight_starts_the_shift_at_one() {
    // Santiago's clocks jump from 00:00 to 01:00 on Sunday 2026-09-06, at 04:00 UTC.
    let hours = hours(Santiago, vec![shift(Weekday::Sun, (0, 0), (4, 0))]);

    assert_eq!(
        expand(&hours, date(2026, 9, 6), date(2026, 9, 7)),
        [slot(utc(2026, 9, 6, 4, 0), utc(2026, 9, 6, 7, 0))],
    );
}

#[test]
fn in_santiago_a_repeated_late_evening_ends_the_shift_at_its_first_occurrence() {
    // Santiago's clocks go back from 00:00 to 23:00 on Saturday 2026-04-04, at 03:00 UTC.
    let hours = hours(Santiago, vec![shift(Weekday::Sat, (20, 0), (23, 30))]);

    assert_eq!(
        expand(&hours, date(2026, 4, 4), date(2026, 4, 5)),
        [slot(utc(2026, 4, 4, 23, 0), utc(2026, 4, 5, 2, 30))],
    );
}

#[test]
fn a_whole_year_of_clock_changes_expands_to_sorted_separated_shifts() {
    // Boundaries inside every change window: Madrid's 02:00-03:00, Santiago's around midnight,
    // and Lord Howe's half-hour changes at 02:00.
    let times = [
        ((0, 0), (0, 30)),
        ((0, 40), (1, 40)),
        ((1, 45), (2, 15)),
        ((2, 20), (2, 25)),
        ((2, 35), (3, 30)),
        ((22, 30), (23, 20)),
        ((23, 30), (23, 59)),
    ];
    let every_day = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    for time_zone in [Madrid, Santiago, Lord_Howe] {
        let shifts = every_day
            .iter()
            .flat_map(|day| times.map(|(start, end)| shift(*day, start, end)))
            .collect();
        let hours = hours(time_zone, shifts);

        let expanded = expand(&hours, date(2026, 1, 1), date(2027, 1, 1));

        assert!(expanded.len() > 365 * 6, "{time_zone}: {}", expanded.len());
        assert!(
            expanded
                .windows(2)
                .all(|pair| pair[0].end() < pair[1].start()),
            "{time_zone}: shifts out of order or touching",
        );
    }
}

#[test]
fn expanded_hours_accept_bookings_at_local_times() {
    let hours = hours(Madrid, vec![shift(Weekday::Mon, (9, 0), (13, 0))]);
    let mut schedule = Schedule::new(hours.working_hours(date(2026, 3, 30), date(2026, 3, 31)));
    let consultation = AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap();
    let nine_in_madrid = Madrid
        .with_ymd_and_hms(2026, 3, 30, 9, 0, 0)
        .unwrap()
        .with_timezone(&Utc);

    let booked = schedule
        .book(PatientId::new(1), nine_in_madrid, consultation.clone())
        .map(|appointment| appointment.slot());

    assert_eq!(
        booked,
        Ok(slot(utc(2026, 3, 30, 7, 0), utc(2026, 3, 30, 7, 30))),
    );
    assert_eq!(
        schedule
            .book(
                PatientId::new(1),
                nine_in_madrid - TimeDelta::minutes(30),
                consultation
            )
            .map(|appointment| appointment.slot()),
        Err(BookingError::OutsideWorkingHours),
    );
}

#[test]
fn errors_have_readable_messages() {
    assert_eq!(
        WeeklyHoursError::EndNotAfterStart.to_string(),
        "weekly shift must end after it starts, on the same day",
    );
    assert_eq!(
        WeeklyHoursError::ShiftsNotSeparated {
            weekday: Weekday::Mon
        }
        .to_string(),
        "shifts on Mon must not overlap or touch",
    );
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(WeeklyHoursError::EndNotAfterStart);
}
