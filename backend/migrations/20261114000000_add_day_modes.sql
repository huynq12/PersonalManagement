-- Day modes: habits can stay on low-energy days, and a day can be marked as a low-energy day.
-- See SPEC.md, user stories C7 to C9 and the "Checklist" tables.

ALTER TABLE habits
    -- Whether the habit stays visible on low-energy days.
    ADD COLUMN low_energy boolean NOT NULL DEFAULT false,
    -- Easier version shown on low-energy days; null means the normal name is shown.
    ADD COLUMN low_energy_name text,
    ADD CONSTRAINT habits_low_energy_name_requires_low_energy
        CHECK (low_energy OR low_energy_name IS NULL);

-- A row marks the date as a low-energy day; every other day is a normal day.
CREATE TABLE low_energy_days (
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- Calendar date in the user's time zone.
    date date NOT NULL,
    PRIMARY KEY (user_id, date)
);
ALTER TABLE low_energy_days ENABLE ROW LEVEL SECURITY;
