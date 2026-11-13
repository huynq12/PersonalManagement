-- Daily work log entries. See SPEC.md section 5 ("Work log").
--
-- Each entry keeps the hourly wage and base hours that were current when it was created, so a
-- later settings change never alters recorded days. Pay is not stored: it is computed from
-- hours and hourly_wage when read.
--
-- Row level security is enabled with no policy, as for the other tables: the app connects as
-- the table owner and bypasses it, while Supabase's Data API roles get no access.

CREATE TABLE work_entries (
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    date date NOT NULL,
    hours numeric(4, 2) NOT NULL CHECK (hours > 0 AND hours <= 24),
    -- Minor units of the account currency, copied from user_settings on creation.
    hourly_wage bigint NOT NULL CHECK (hourly_wage BETWEEN 1 AND 1000000000000),
    base_hours numeric(4, 2) NOT NULL CHECK (base_hours > 0 AND base_hours <= 24),
    note text CHECK (char_length(note) <= 500),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    -- One entry per user and day; the primary key is also the lookup index for month ranges.
    PRIMARY KEY (user_id, date)
);
ALTER TABLE work_entries ENABLE ROW LEVEL SECURITY;
