-- The platform schema: what the platform knows about every clinic. Each clinic's own data lives
-- in its own schema, `clinic_<id>`, created when the clinic is provisioned.

CREATE SCHEMA platform;

CREATE TABLE platform.clinics (
    -- A ClinicId. Rust holds it as a u64, which is never negative.
    id          bigint PRIMARY KEY CHECK (id >= 0),
    -- The same rule as Slug::parse in the domain.
    slug        text NOT NULL UNIQUE
                CHECK (length(slug) <= 63 AND slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    -- The same rule as SchemaName in the db crate: a function of the id, never chosen freely.
    schema_name text NOT NULL CHECK (schema_name = 'clinic_' || id),
    status      text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended')),
    created_at  timestamptz NOT NULL DEFAULT now()
);
