# Multi-tenant clinic management platform

## The problem

Many independent medical clinics manage appointments by phone and keep patient records across paper files and disconnected spreadsheets. Each one suffers the same problems:

- **Scheduling conflicts.** Appointments are double-booked for the same practitioner, or booked outside the practitioner's working hours.
- **No-shows and late changes.** Patients forget appointments, and cancellations are not reflected in time for the slot to be offered to someone else.
- **Fragmented records.** A patient's history (visits, diagnoses, notes, allergies) is spread across sources, so practitioners do not have the full picture during a visit.
- **Uncontrolled access to sensitive data.** Anyone with access to the files can read any record, and there is no trace of who viewed or changed what.
- **Errors are hard to correct safely.** When a record is wrong, it is overwritten, and the previous information is lost.

Solving this alone is expensive, and most clinics cannot afford to build or operate their own system. A company wants to offer one platform that serves all of these clinics at once, as a subscription service. The challenge is that the clinics are very different from each other, while sharing the same product:

- **Size varies widely.** Some customers are a single practitioner with a few dozen patients; others are clinic networks with many locations, hundreds of practitioners and heavy daily traffic.
- **Data must never leak between clinics.** A clinic's patients, staff and records must be invisible to every other clinic, and any breach of that boundary is a serious legal and reputational failure.
- **Requirements differ by customer.** Larger clinics may demand stronger guarantees: their own data residency, dedicated capacity, stricter compliance or custom integrations. Smaller clinics want a low price and zero setup.
- **One customer must not hurt the others.** A heavy clinic, a traffic spike or a failure affecting one customer should not degrade the service for the rest.
- **Onboarding must be fast.** A new clinic should be able to sign up and start working without manual intervention, while large customers may need a guided setup.
- **Operating cost must stay sustainable.** The company needs to know what each customer costs to serve, so pricing tiers stay profitable.

## Who is affected

- **Patients** need to book, reschedule or cancel appointments, and trust that their data is private.
- **Receptionists** need to manage the daily schedule of every practitioner in their clinic.
- **Practitioners** need to see their agenda and the full, accurate record of each patient they treat.
- **Clinic administrators** need to know who accessed or modified patient data.
- **The platform company** needs to onboard, operate, bill and support all clinics with a small team.
- **The platform support team** needs one place to manage every clinic: onboard and suspend clinics, change their plans, manage their administrator accounts and investigate problems, without being able to read patients' clinical information.
- **Regulators and auditors** need evidence that health data is protected and traceable.

## Rules that must be enforced

Within each clinic:

- A practitioner cannot have two appointments that overlap, nor appointments outside their working hours.
- Each appointment type has a defined duration.
- Only practitioners can read or write clinical notes; receptionists can see schedules and contact details, but not clinical content.
- Every read and change of a patient record is recorded: who, what and when.
- Clinical records are never silently overwritten; corrections keep the previous version.

Across the platform:

- No clinic can see or affect another clinic's data.
- Support staff can act on any clinic's account and configuration, but never on clinical content, and every support action is recorded.
- Patient data is personal health information and must be handled under the applicable data protection law, including access control, auditability, retention rules, and exporting or deleting a patient's data on request.

## What a solution must answer

How should the platform be structured so that it serves clinics of every size from one product, keeps each clinic's data isolated, prevents customers from affecting each other, meets regulatory obligations, and remains economical to run as the number of clinics grows?

## Optional extras

- Add docs that allow to be rendered in a static page (like gitbooks)
- Add CI/CD using gitlab-ci and/or github actions
- Add use of containers (docker or k8s)
- Add infrastructure as a code (terraform)
- Add opentelemetry (netdata, grafana or other)
