use chrono::{DateTime, TimeZone, Utc};
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::{WorkingHours, WorkingHoursError};

fn on(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, day, hour, minute, 0).unwrap()
}

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    on(2, hour, minute)
}

fn slot(start: DateTime<Utc>, end: DateTime<Utc>) -> TimeSlot {
    TimeSlot::new(start, end).unwrap()
}

/// 09:00-13:00 and 14:00-18:00, with a lunch break between them.
fn day_with_lunch_break() -> WorkingHours {
    WorkingHours::new(vec![slot(at(9, 0), at(13, 0)), slot(at(14, 0), at(18, 0))]).unwrap()
}

#[test]
fn a_slot_inside_a_shift_fits() {
    assert!(day_with_lunch_break().contains(&slot(at(9, 30), at(10, 0))));
}

#[test]
fn a_slot_in_the_second_shift_fits() {
    assert!(day_with_lunch_break().contains(&slot(at(15, 0), at(15, 30))));
}

#[test]
fn a_slot_ending_exactly_at_the_end_of_a_shift_fits() {
    assert!(day_with_lunch_break().contains(&slot(at(17, 30), at(18, 0))));
}

#[test]
fn a_slot_running_past_the_end_of_a_shift_does_not_fit() {
    assert!(!day_with_lunch_break().contains(&slot(at(17, 30), at(18, 30))));
}

#[test]
fn a_slot_during_the_gap_between_shifts_does_not_fit() {
    assert!(!day_with_lunch_break().contains(&slot(at(13, 15), at(13, 45))));
}

#[test]
fn a_slot_spanning_two_shifts_does_not_fit() {
    assert!(!day_with_lunch_break().contains(&slot(at(12, 30), at(14, 30))));
}

#[test]
fn a_slot_on_a_day_without_a_shift_does_not_fit() {
    assert!(!day_with_lunch_break().contains(&slot(on(3, 9, 30), on(3, 10, 0))));
}

#[test]
fn empty_hours_accept_nothing() {
    let hours = WorkingHours::new(Vec::new()).unwrap();

    assert!(!hours.contains(&slot(at(9, 0), at(10, 0))));
}

#[test]
fn rejects_overlapping_shifts() {
    assert_eq!(
        WorkingHours::new(vec![slot(at(9, 0), at(13, 0)), slot(at(12, 0), at(18, 0)),]),
        Err(WorkingHoursError::ShiftsNotSeparated),
    );
}

#[test]
fn rejects_touching_shifts() {
    assert_eq!(
        WorkingHours::new(vec![slot(at(9, 0), at(12, 0)), slot(at(12, 0), at(17, 0)),]),
        Err(WorkingHoursError::ShiftsNotSeparated),
    );
}

#[test]
fn rejects_overlapping_shifts_given_out_of_order() {
    assert_eq!(
        WorkingHours::new(vec![
            slot(at(12, 0), at(18, 0)),
            slot(at(15, 0), at(16, 0)),
            slot(at(9, 0), at(13, 0)),
        ]),
        Err(WorkingHoursError::ShiftsNotSeparated),
    );
}

#[test]
fn exposes_the_shifts_sorted_by_start() {
    let morning = slot(at(9, 0), at(13, 0));
    let afternoon = slot(at(14, 0), at(18, 0));
    let next_day = slot(on(3, 9, 0), on(3, 13, 0));

    let hours = WorkingHours::new(vec![next_day, afternoon, morning]).unwrap();

    assert_eq!(hours.shifts(), [morning, afternoon, next_day]);
}

#[test]
fn error_has_a_readable_message() {
    assert_eq!(
        WorkingHoursError::ShiftsNotSeparated.to_string(),
        "working hours shifts must not overlap or touch",
    );
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(WorkingHoursError::ShiftsNotSeparated);
}

fn hours(shifts: Vec<TimeSlot>) -> WorkingHours {
    WorkingHours::new(shifts).unwrap()
}

#[test]
fn replacing_from_a_cut_keeps_earlier_shifts_and_takes_later_ones_from_the_replacement() {
    let old = hours(vec![
        slot(on(2, 9, 0), on(2, 13, 0)),
        slot(on(3, 9, 0), on(3, 13, 0)),
    ]);
    let new = hours(vec![
        slot(on(3, 14, 0), on(3, 18, 0)),
        slot(on(4, 14, 0), on(4, 18, 0)),
    ]);

    let replaced = old.replace_from(on(3, 0, 0), &new);

    assert_eq!(
        replaced.shifts(),
        [
            slot(on(2, 9, 0), on(2, 13, 0)),
            slot(on(3, 14, 0), on(3, 18, 0)),
            slot(on(4, 14, 0), on(4, 18, 0)),
        ]
    );
}

#[test]
fn replacing_drops_replacement_shifts_before_the_cut() {
    let old = hours(vec![slot(on(2, 9, 0), on(2, 13, 0))]);
    let new = hours(vec![
        slot(on(2, 14, 0), on(2, 18, 0)),
        slot(on(3, 9, 0), on(3, 13, 0)),
    ]);

    let replaced = old.replace_from(on(3, 0, 0), &new);

    assert_eq!(
        replaced.shifts(),
        [
            slot(on(2, 9, 0), on(2, 13, 0)),
            slot(on(3, 9, 0), on(3, 13, 0)),
        ]
    );
}

#[test]
fn shifts_that_straddle_the_cut_are_cut_short() {
    let old = hours(vec![slot(on(2, 20, 0), on(3, 2, 0))]);
    let new = hours(vec![slot(on(2, 23, 0), on(3, 4, 0))]);

    let replaced = old.replace_from(on(3, 1, 0), &new);

    // Both pieces meet at the cut, so they become one shift.
    assert_eq!(replaced.shifts(), [slot(on(2, 20, 0), on(3, 4, 0))]);
}

#[test]
fn a_straddling_old_shift_is_cut_short_when_nothing_meets_it() {
    let old = hours(vec![slot(on(2, 20, 0), on(3, 2, 0))]);
    let new = hours(vec![slot(on(3, 9, 0), on(3, 13, 0))]);

    let replaced = old.replace_from(on(3, 1, 0), &new);

    assert_eq!(
        replaced.shifts(),
        [
            slot(on(2, 20, 0), on(3, 1, 0)),
            slot(on(3, 9, 0), on(3, 13, 0)),
        ]
    );
}

#[test]
fn a_straddling_new_shift_is_cut_short_when_nothing_meets_it() {
    let old = hours(vec![slot(on(2, 9, 0), on(2, 13, 0))]);
    let new = hours(vec![slot(on(2, 20, 0), on(3, 2, 0))]);

    let replaced = old.replace_from(on(3, 1, 0), &new);

    assert_eq!(
        replaced.shifts(),
        [
            slot(on(2, 9, 0), on(2, 13, 0)),
            slot(on(3, 1, 0), on(3, 2, 0)),
        ]
    );
}

#[test]
fn replacing_with_empty_hours_clears_everything_from_the_cut() {
    let old = hours(vec![
        slot(on(2, 9, 0), on(2, 13, 0)),
        slot(on(3, 9, 0), on(3, 13, 0)),
    ]);

    let replaced = old.replace_from(on(3, 0, 0), &hours(Vec::new()));

    assert_eq!(replaced.shifts(), [slot(on(2, 9, 0), on(2, 13, 0))]);
}

#[test]
fn replacing_empty_hours_takes_the_replacement_from_the_cut() {
    let new = hours(vec![
        slot(on(2, 9, 0), on(2, 13, 0)),
        slot(on(3, 9, 0), on(3, 13, 0)),
    ]);

    let replaced = hours(Vec::new()).replace_from(on(3, 0, 0), &new);

    assert_eq!(replaced.shifts(), [slot(on(3, 9, 0), on(3, 13, 0))]);
}
