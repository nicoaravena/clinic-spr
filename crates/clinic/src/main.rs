//! The clinic app. Until step 2.5 it runs a scripted day at one clinic, and each roadmap step
//! extends the script to show what that step added. In step 2.5 it becomes the server.
//!
//! The day uses fixed UTC times and no clock, so every run prints the same output.
//! `tests/expected-output.txt` holds that output.

use std::error::Error;

use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc, Weekday};
use chrono_tz::America::Santiago;
use clinic_app::agenda::{AgendaQuery, practitioner_agenda};
use clinic_app::booking::{BookingRequest, book_appointment};
use clinic_app::in_memory::{InMemoryAppointmentTypes, InMemorySchedules};
use clinic_app::ports::ScheduleRepository;
use clinic_app::weekly_hours::{WeeklyHours, WeeklyShift};
use clinic_domain::access::{AuditAccess, ClinicalAccess, Role, ScheduleAccess};
use clinic_domain::appointment::AppointmentType;
use clinic_domain::clinical_note::ClinicalNote;
use clinic_domain::email::Email;
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::{BookingError, Schedule};
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

/// Runs the scripted day. An error that the script doesn't expect ends the run with exit code 1.
fn main() -> Result<(), Box<dyn Error>> {
    let span = |slot: TimeSlot| {
        format!(
            "{}-{}",
            slot.start().format("%H:%M"),
            slot.end().format("%H:%M"),
        )
    };

    let clinic = ClinicId::new(1);
    println!(
        "Clinic {} opens on {}. All times are UTC.",
        clinic.value(),
        at(0, 0).format("%A %Y-%m-%d"),
    );

    println!("\n[1.2] Staff");
    let ana = PractitionerId::new(1);
    let ana_email = Email::parse("ana@example.com")?;
    println!(
        "  Practitioner {} registered as {}",
        ana.value(),
        ana_email.as_str(),
    );
    if let Err(error) = Email::parse("ana@example") {
        println!("  Email \"ana@example\" refused: {error}");
    }

    println!("\n[1.3] Time slots");
    if let Err(error) = TimeSlot::new(at(10, 0), at(9, 0)) {
        println!("  Slot 10:00-09:00 refused: {error}");
    }

    println!("\n[1.4] Appointment types");
    let consultation = AppointmentType::new("Consultation", TimeDelta::minutes(30))?;
    let follow_up = AppointmentType::new("Follow-up", TimeDelta::minutes(15))?;
    for appointment_type in [&consultation, &follow_up] {
        println!(
            "  {} lasts {} min",
            appointment_type.name(),
            appointment_type.duration().num_minutes(),
        );
    }
    if let Err(error) = AppointmentType::new("Check-up", TimeDelta::zero()) {
        println!("  Check-up lasting 0 min refused: {error}");
    }

    println!("\n[1.5] Working hours");
    let hours = WorkingHours::new(vec![
        TimeSlot::new(at(9, 0), at(12, 0))?,
        TimeSlot::new(at(13, 0), at(17, 0))?,
    ])?;
    for shift in hours.shifts() {
        println!("  Practitioner {} works {}", ana.value(), span(*shift));
    }

    println!("\n[1.6] Booking");
    let patient = PatientId::new(1);
    let mut schedule = Schedule::new(hours.clone());
    let requests = [
        (at(9, 0), &consultation),
        (at(9, 15), &follow_up),
        (at(11, 45), &consultation),
        (at(13, 0), &follow_up),
    ];
    for (start, appointment_type) in requests {
        let request = format!("{} at {}", appointment_type.name(), start.format("%H:%M"),);
        match schedule.book(patient, start, appointment_type.clone()) {
            Ok(appointment) => println!("  {request} booked for {}", span(appointment.slot())),
            Err(error @ BookingError::Overlaps { existing }) => {
                println!("  {request} refused: {error} ({})", span(existing));
            }
            Err(error) => println!("  {request} refused: {error}"),
        }
    }
    println!("  Agenda for practitioner {}:", ana.value());
    for appointment in schedule.appointments() {
        println!(
            "    {} {} ({:?})",
            span(appointment.slot()),
            appointment.appointment_type().name(),
            appointment.status(),
        );
    }

    println!("\n[1.7] Access");
    for (who, role) in [
        ("Receptionist", Role::Receptionist),
        ("Administrator", Role::Administrator),
    ] {
        if let Err(denied) = role.grant::<ClinicalAccess>() {
            println!("  {who} asks for clinical content: refused: {denied}");
        }
    }
    Role::Administrator.grant::<AuditAccess>()?;
    println!("  Administrator asks for the audit log: granted");
    let clinical = Role::Practitioner(ana).grant::<ClinicalAccess>()?;
    println!(
        "  Practitioner {} asks for clinical content: granted to practitioner {}",
        ana.value(),
        clinical.practitioner().value(),
    );

    println!("\n[1.8] Clinical notes");
    let mut note = ClinicalNote::write(&clinical, patient, "Mild fever.", at(9, 30))?;
    println!(
        "  Practitioner {} writes a note for patient {} at 09:30",
        ana.value(),
        patient.value(),
    );
    note.correct(&clinical, "Mild fever, 38.2 C.", at(9, 45))?;
    println!("  Practitioner {} corrects it at 09:45", ana.value());
    let refused_corrections = [
        ("with the same text", "Mild fever, 38.2 C.", at(10, 0)),
        ("dated 09:00", "No fever.", at(9, 0)),
    ];
    for (what, text, written_at) in refused_corrections {
        if let Err(error) = note.correct(&clinical, text, written_at) {
            println!("  Correction {what} refused: {error}");
        }
    }
    let ben = PractitionerId::new(2);
    let ben_clinical = Role::Practitioner(ben).grant::<ClinicalAccess>()?;
    if let Err(error) = note.correct(&ben_clinical, "No fever.", at(10, 0)) {
        println!(
            "  Practitioner {} corrects it: refused: {error}",
            ben.value()
        );
    }
    println!("  History of the note, oldest first:");
    for version in note.history(&clinical) {
        println!(
            "    {} by practitioner {}: {}",
            version.written_at().format("%H:%M"),
            version.author().value(),
            version.text(),
        );
    }

    println!("\n[2.1] Booking through the app");
    let mut schedules = InMemorySchedules::new();
    schedules.save(clinic, ana, Schedule::new(hours));
    let mut types = InMemoryAppointmentTypes::new();
    types.add(clinic, consultation.clone())?;
    types.add(clinic, follow_up.clone())?;
    if let Err(error) = types.add(clinic, consultation.clone()) {
        println!(
            "  Clinic {} adds Consultation again: refused: {error}",
            clinic.value()
        );
    }
    let reception: ScheduleAccess = Role::Receptionist.grant()?;
    let other_clinic = ClinicId::new(2);
    types.add(other_clinic, consultation.clone())?;
    let requests = [
        (clinic, "Consultation", at(10, 0)),
        (clinic, "Check-up", at(10, 30)),
        (clinic, "Follow-up", at(10, 15)),
        (other_clinic, "Consultation", at(11, 0)),
    ];
    for (at_clinic, appointment_type, start) in requests {
        let request = format!(
            "Clinic {} books {appointment_type} with practitioner {} at {}",
            at_clinic.value(),
            ana.value(),
            start.format("%H:%M"),
        );
        let booking = BookingRequest {
            clinic: at_clinic,
            practitioner: ana,
            patient,
            appointment_type,
            start,
        };
        match book_appointment(&mut schedules, &types, &reception, booking) {
            Ok(appointment) => println!("  {request}: booked for {}", span(appointment.slot())),
            Err(error) => println!("  {request}: refused: {error}"),
        }
    }
    if let Some(saved) = schedules.get(clinic, ana) {
        println!("  Saved agenda for practitioner {}:", ana.value());
        for appointment in saved.appointments() {
            println!(
                "    {} {} for patient {}",
                span(appointment.slot()),
                appointment.appointment_type().name(),
                appointment.patient().value(),
            );
        }
    }

    println!("\n[2.2] Agenda");
    let second_patient = PatientId::new(2);
    let booking = BookingRequest {
        clinic,
        practitioner: ana,
        patient: second_patient,
        appointment_type: "Follow-up",
        start: at(13, 0),
    };
    let appointment = book_appointment(&mut schedules, &types, &reception, booking)?;
    println!(
        "  Clinic {} books Follow-up with practitioner {} at 13:00 for patient {}: booked for {}",
        clinic.value(),
        ana.value(),
        second_patient.value(),
        span(appointment.slot()),
    );
    let queries = [
        (clinic, TimeSlot::new(at(9, 0), at(12, 0))?),
        (clinic, TimeSlot::new(at(10, 15), at(13, 15))?),
        (clinic, TimeSlot::new(at(12, 0), at(13, 0))?),
        (other_clinic, TimeSlot::new(at(9, 0), at(17, 0))?),
    ];
    for (at_clinic, range) in queries {
        let heading = format!(
            "Agenda for practitioner {} at clinic {}, {}",
            ana.value(),
            at_clinic.value(),
            span(range),
        );
        let query = AgendaQuery {
            clinic: at_clinic,
            practitioner: ana,
            range,
        };
        match practitioner_agenda(&schedules, &reception, query) {
            Ok(agenda) if agenda.is_empty() => println!("  {heading}: no appointments"),
            Ok(agenda) => {
                println!("  {heading}:");
                for appointment in agenda {
                    println!(
                        "    {} {} for patient {}",
                        span(appointment.slot()),
                        appointment.appointment_type().name(),
                        appointment.patient().value(),
                    );
                }
            }
            Err(error) => println!("  {heading}: refused: {error}"),
        }
    }

    println!("\n[2.3] Weekly hours");
    let mut weekly = Vec::new();
    for weekday in [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ] {
        weekly.push(WeeklyShift::new(weekday, local(9, 0), local(13, 0))?);
        weekly.push(WeeklyShift::new(weekday, local(14, 0), local(18, 0))?);
    }
    weekly.push(WeeklyShift::new(Weekday::Sat, local(20, 0), local(23, 30))?);
    weekly.push(WeeklyShift::new(Weekday::Sun, local(0, 0), local(2, 0))?);
    let weekly = WeeklyHours::new(Santiago, weekly)?;
    println!(
        "  Practitioner {} works Mon-Fri 09:00-13:00 and 14:00-18:00, on call Sat 20:00-23:30 and Sun 00:00-02:00, in {}",
        ana.value(),
        weekly.time_zone(),
    );
    let ranges = [
        (
            "Clocks go back at the end of Sat 2026-04-04, so 23:00-24:00 happens twice",
            day(2026, 4, 3),
            day(2026, 4, 7),
        ),
        (
            "Clocks jump forward at the start of Sun 2026-09-06, so 00:00-01:00 doesn't exist",
            day(2026, 9, 5),
            day(2026, 9, 7),
        ),
    ];
    for (heading, from, until) in ranges {
        println!("  {heading}:");
        for shift in weekly.working_hours(from, until).shifts() {
            let start = shift.start().with_timezone(&Santiago);
            let end = shift.end().with_timezone(&Santiago);
            println!(
                "    {}-{} at {} is {} UTC",
                start.format("%a %Y-%m-%d %H:%M"),
                end.format("%H:%M"),
                start.format("%:z"),
                span(*shift),
            );
        }
    }

    Ok(())
}

/// A time on the scripted day, Monday 2026-03-02, in UTC.
fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}

/// A local time of day, for weekly hours.
fn local(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
}

/// A local calendar date, for expanding weekly hours.
fn day(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}
