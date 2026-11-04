use chrono::{DateTime, TimeZone, Utc};
use clinic_domain::time_slot::{TimeSlot, TimeSlotError};

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}

fn slot(start: DateTime<Utc>, end: DateTime<Utc>) -> TimeSlot {
    TimeSlot::new(start, end).unwrap()
}

fn assert_overlap(a: TimeSlot, b: TimeSlot) {
    assert!(a.overlaps(&b), "expected {a:?} to overlap {b:?}");
    assert!(b.overlaps(&a), "expected {b:?} to overlap {a:?}");
}

fn assert_no_overlap(a: TimeSlot, b: TimeSlot) {
    assert!(!a.overlaps(&b), "expected {a:?} not to overlap {b:?}");
    assert!(!b.overlaps(&a), "expected {b:?} not to overlap {a:?}");
}

#[test]
fn exposes_the_start_and_end_it_was_created_with() {
    let slot = TimeSlot::new(at(9, 0), at(9, 30)).unwrap();

    assert_eq!(slot.start(), at(9, 0));
    assert_eq!(slot.end(), at(9, 30));
}

#[test]
fn rejects_an_end_equal_to_the_start() {
    assert_eq!(
        TimeSlot::new(at(9, 0), at(9, 0)),
        Err(TimeSlotError::EndNotAfterStart),
    );
}

#[test]
fn rejects_an_end_before_the_start() {
    assert_eq!(
        TimeSlot::new(at(9, 30), at(9, 0)),
        Err(TimeSlotError::EndNotAfterStart),
    );
}

#[test]
fn slots_that_partly_share_time_overlap() {
    assert_overlap(slot(at(9, 0), at(10, 0)), slot(at(9, 30), at(10, 30)));
}

#[test]
fn a_slot_overlaps_one_it_contains() {
    assert_overlap(slot(at(9, 0), at(11, 0)), slot(at(9, 30), at(10, 0)));
}

#[test]
fn identical_slots_overlap() {
    assert_overlap(slot(at(9, 0), at(10, 0)), slot(at(9, 0), at(10, 0)));
}

#[test]
fn back_to_back_slots_do_not_overlap() {
    assert_no_overlap(slot(at(9, 0), at(9, 30)), slot(at(9, 30), at(10, 0)));
}

#[test]
fn slots_separated_by_a_gap_do_not_overlap() {
    assert_no_overlap(slot(at(9, 0), at(9, 30)), slot(at(11, 0), at(11, 30)));
}

#[test]
fn error_has_a_readable_message() {
    assert_eq!(
        TimeSlotError::EndNotAfterStart.to_string(),
        "time slot end must be after its start",
    );
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(TimeSlotError::EndNotAfterStart);
}
