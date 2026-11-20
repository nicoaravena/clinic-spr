use chrono::{DateTime, Days, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc, Weekday};
use chrono_tz::America::Santiago;
use chrono_tz::{Etc, Pacific};
use clinic_app::hours::{
    HORIZON_DAYS, HoursOutcome, SetHoursError, SetHoursRequest, set_weekly_hours,
};
use clinic_app::in_memory::InMemorySchedules;
use clinic_app::ports::ScheduleRepository;
use clinic_app::weekly_hours::{WeeklyHours, WeeklyShift};
use clinic_domain::access::{Role, ScheduleAccess};
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::{ReplaceHoursError, Schedule};
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

fn clinic() -> ClinicId {
    ClinicId::new(1)
}

fn ana() -> PractitionerId {
    PractitionerId::new(1)
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn utc(month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, month, day, hour, minute, 0)
        .unwrap()
}

fn slot(start: DateTime<Utc>, end: DateTime<Utc>) -> TimeSlot {
    TimeSlot::new(start, end).unwrap()
}

fn access() -> ScheduleAccess {
    Role::Receptionist.grant().unwrap()
}

/// Mondays from `start` to `end` o'clock, in Santiago.
///
/// Santiago is at -03:00 until its clocks go back on Saturday 2026-04-04, and at -04:00 after.
fn mondays(start: u32, end: u32) -> WeeklyHours {
    let shift = WeeklyShift::new(
        Weekday::Mon,
        NaiveTime::from_hms_opt(start, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(end, 0, 0).unwrap(),
    )
    .unwrap();
    WeeklyHours::new(Santiago, vec![shift]).unwrap()
}

fn request(hours: WeeklyHours, from: NaiveDate) -> SetHoursRequest {
    SetHoursRequest {
        clinic: clinic(),
        practitioner: ana(),
        hours,
        from,
    }
}

/// Ana's saved shifts at clinic 1.
async fn saved_shifts(schedules: &InMemorySchedules) -> Vec<TimeSlot> {
    schedules
        .get(clinic(), ana())
        .await
        .unwrap()
        .unwrap()
        .hours()
        .shifts()
        .to_vec()
}

/// Books a 30-minute consultation for Ana at `start`, straight into the saved schedule.
async fn book(schedules: &mut InMemorySchedules, start: DateTime<Utc>) {
    let mut schedule = schedules.get(clinic(), ana()).await.unwrap().unwrap();
    let consultation = AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap();
    schedule
        .book(PatientId::new(7), start, consultation)
        .unwrap();
    schedules.save(clinic(), ana(), schedule).await.unwrap();
}

/// Ana works Mondays 09:00-13:00 from 2026-03-30, with a consultation on 2026-04-06 at 09:00
/// local time (13:00 UTC).
async fn booked() -> InMemorySchedules {
    let mut schedules = InMemorySchedules::new();
    set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(9, 13), date(2026, 3, 30)),
    )
    .await
    .unwrap();
    book(&mut schedules, utc(4, 6, 13, 0)).await;
    schedules
}

#[test]
fn the_horizon_is_four_weeks() {
    assert_eq!(HORIZON_DAYS, 28);
}

#[tokio::test]
async fn creates_a_schedule_when_the_practitioner_has_none() {
    let mut schedules = InMemorySchedules::new();

    let set = set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(9, 13), date(2026, 3, 30)),
    )
    .await
    .unwrap();

    assert_eq!(set.outcome, HoursOutcome::Created);
    let saved = schedules.get(clinic(), ana()).await.unwrap().unwrap();
    assert_eq!(saved.hours(), &set.shifts);
    assert!(saved.appointments().is_empty());
}

#[tokio::test]
async fn expands_the_template_over_the_horizon_across_daylight_saving() {
    let mut schedules = InMemorySchedules::new();

    let set = set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(9, 13), date(2026, 3, 30)),
    )
    .await
    .unwrap();

    assert_eq!(set.until, date(2026, 4, 27));
    assert_eq!(
        set.shifts.shifts(),
        [
            slot(utc(3, 30, 12, 0), utc(3, 30, 16, 0)),
            slot(utc(4, 6, 13, 0), utc(4, 6, 17, 0)),
            slot(utc(4, 13, 13, 0), utc(4, 13, 17, 0)),
            slot(utc(4, 20, 13, 0), utc(4, 20, 17, 0)),
        ]
    );
}

#[tokio::test]
async fn replacing_keeps_the_shifts_before_from_and_answers_replaced() {
    let mut schedules = InMemorySchedules::new();
    set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(9, 13), date(2026, 3, 30)),
    )
    .await
    .unwrap();

    let set = set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(14, 18), date(2026, 4, 13)),
    )
    .await
    .unwrap();

    assert_eq!(set.outcome, HoursOutcome::Replaced);
    assert_eq!(set.until, date(2026, 5, 11));
    assert_eq!(
        saved_shifts(&schedules).await,
        [
            slot(utc(3, 30, 12, 0), utc(3, 30, 16, 0)),
            slot(utc(4, 6, 13, 0), utc(4, 6, 17, 0)),
            slot(utc(4, 13, 18, 0), utc(4, 13, 22, 0)),
            slot(utc(4, 20, 18, 0), utc(4, 20, 22, 0)),
            slot(utc(4, 27, 18, 0), utc(4, 27, 22, 0)),
            slot(utc(5, 4, 18, 0), utc(5, 4, 22, 0)),
        ]
    );
}

#[tokio::test]
async fn the_answer_holds_only_the_new_shifts() {
    let mut schedules = booked().await;

    let set = set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(14, 18), date(2026, 4, 27)),
    )
    .await
    .unwrap();

    assert_eq!(
        set.shifts.shifts(),
        [
            slot(utc(4, 27, 18, 0), utc(4, 27, 22, 0)),
            slot(utc(5, 4, 18, 0), utc(5, 4, 22, 0)),
            slot(utc(5, 11, 18, 0), utc(5, 11, 22, 0)),
            slot(utc(5, 18, 18, 0), utc(5, 18, 22, 0)),
        ]
    );
}

#[tokio::test]
async fn the_same_request_twice_leaves_the_same_schedule() {
    let mut schedules = booked().await;
    let before = schedules.get(clinic(), ana()).await.unwrap().unwrap();

    let set = set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(9, 13), date(2026, 3, 30)),
    )
    .await
    .unwrap();

    assert_eq!(set.outcome, HoursOutcome::Replaced);
    assert_eq!(
        schedules.get(clinic(), ana()).await.unwrap().unwrap(),
        before
    );
}

#[tokio::test]
async fn booked_appointments_before_from_are_kept() {
    let mut schedules = booked().await;

    set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(14, 18), date(2026, 4, 13)),
    )
    .await
    .unwrap();

    let saved = schedules.get(clinic(), ana()).await.unwrap().unwrap();
    let slots: Vec<TimeSlot> = saved.appointments().iter().map(|a| a.slot()).collect();
    assert_eq!(slots, [slot(utc(4, 6, 13, 0), utc(4, 6, 13, 30))]);
}

#[tokio::test]
async fn refuses_hours_that_leave_a_booked_appointment_outside_and_saves_nothing() {
    let mut schedules = booked().await;
    let before = schedules.get(clinic(), ana()).await.unwrap().unwrap();

    let result = set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(14, 18), date(2026, 3, 30)),
    )
    .await;

    assert!(
        matches!(
            result,
            Err(SetHoursError::Hours(ReplaceHoursError::AppointmentOutsideHours { existing }))
                if existing == slot(utc(4, 6, 13, 0), utc(4, 6, 13, 30))
        ),
        "{result:?}"
    );
    assert_eq!(
        schedules.get(clinic(), ana()).await.unwrap().unwrap(),
        before
    );
}

#[tokio::test]
async fn an_appointment_past_the_new_horizon_counts_as_outside() {
    let mut schedules = booked().await;
    book(&mut schedules, utc(4, 20, 13, 0)).await;

    // Starting a week earlier, the horizon ends before Monday 2026-04-20.
    let result = set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(9, 13), date(2026, 3, 23)),
    )
    .await;

    assert!(
        matches!(
            result,
            Err(SetHoursError::Hours(ReplaceHoursError::AppointmentOutsideHours { existing }))
                if existing == slot(utc(4, 20, 13, 0), utc(4, 20, 13, 30))
        ),
        "{result:?}"
    );
}

#[tokio::test]
async fn when_midnight_is_skipped_the_old_hours_end_when_the_clocks_jump() {
    // Santiago's clocks jump from 00:00 to 01:00 on Sunday 2026-09-06, at 04:00 UTC.
    let mut schedules = InMemorySchedules::new();
    let old = WorkingHours::new(vec![slot(utc(9, 6, 2, 0), utc(9, 6, 6, 0))]).unwrap();
    schedules
        .save(clinic(), ana(), Schedule::new(old))
        .await
        .unwrap();

    set_weekly_hours(
        &mut schedules,
        &access(),
        request(mondays(9, 13), date(2026, 9, 6)),
    )
    .await
    .unwrap();

    assert_eq!(
        saved_shifts(&schedules).await[0],
        slot(utc(9, 6, 2, 0), utc(9, 6, 4, 0))
    );
}

#[tokio::test]
async fn a_from_at_the_edge_of_the_calendar_is_refused_without_saving() {
    for from in [NaiveDate::MAX, NaiveDate::MIN] {
        let mut schedules = InMemorySchedules::new();

        let result =
            set_weekly_hours(&mut schedules, &access(), request(mondays(9, 13), from)).await;

        assert!(
            matches!(result, Err(SetHoursError::OutOfRange)),
            "from {from}: {result:?}"
        );
        assert_eq!(schedules.get(clinic(), ana()).await.unwrap(), None);
    }
}

#[tokio::test]
async fn the_dates_nearest_the_edge_of_the_calendar_are_accepted() {
    let earliest = NaiveDate::MIN.checked_add_days(Days::new(2)).unwrap();
    let latest = NaiveDate::MAX
        .checked_sub_days(Days::new(HORIZON_DAYS + 2))
        .unwrap();
    let zones = [Santiago, Pacific::Kiritimati, Etc::GMTPlus12];
    let every_day = |time_zone| {
        let shifts = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ]
        .map(|day| {
            WeeklyShift::new(
                day,
                NaiveTime::MIN,
                NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
            )
        })
        .map(Result::unwrap);
        WeeklyHours::new(time_zone, shifts.to_vec()).unwrap()
    };

    for time_zone in zones {
        for from in [earliest, latest] {
            let mut schedules = InMemorySchedules::new();

            let result = set_weekly_hours(
                &mut schedules,
                &access(),
                request(every_day(time_zone), from),
            )
            .await;

            assert!(result.is_ok(), "{time_zone} from {from}: {result:?}");
        }
    }
}

#[tokio::test]
async fn each_clinic_sets_its_own_practitioners_hours() {
    let mut schedules = booked().await;
    let before = schedules.get(clinic(), ana()).await.unwrap().unwrap();

    let set = set_weekly_hours(
        &mut schedules,
        &access(),
        SetHoursRequest {
            clinic: ClinicId::new(2),
            ..request(mondays(14, 18), date(2026, 3, 30))
        },
    )
    .await
    .unwrap();

    assert_eq!(set.outcome, HoursOutcome::Created);
    assert_eq!(
        schedules.get(clinic(), ana()).await.unwrap().unwrap(),
        before
    );
}

#[test]
fn errors_have_readable_messages() {
    assert_eq!(
        SetHoursError::OutOfRange.to_string(),
        "the hours would run past the earliest or latest supported date",
    );
    assert_eq!(
        SetHoursError::Hours(ReplaceHoursError::AppointmentOutsideHours {
            existing: slot(utc(4, 6, 13, 0), utc(4, 6, 13, 30)),
        })
        .to_string(),
        "an existing appointment would be outside the new working hours",
    );
}
