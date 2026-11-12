-- Focus sessions of the Pomodoro timer; breaks are not stored. See SPEC.md section 5.
--
-- Row level security is enabled with no policy, as for the other tables: the app connects as
-- the table owner and bypasses it, while Supabase's Data API roles get no access.

CREATE TABLE study_sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- Deleting a subject keeps its sessions.
    subject_id uuid REFERENCES subjects (id) ON DELETE SET NULL,
    status text NOT NULL CHECK (status IN ('running', 'completed', 'cancelled')),
    planned_minutes smallint NOT NULL CHECK (planned_minutes > 0),
    started_at timestamptz NOT NULL DEFAULT now(),
    ended_at timestamptz,
    -- Set when the session completes: elapsed time, capped at planned_minutes * 60.
    duration_seconds integer CHECK (duration_seconds >= 0),
    -- Start date in the user's time zone.
    local_date date NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX study_sessions_user_id_local_date_idx ON study_sessions (user_id, local_date);
-- At most one running session per user.
CREATE UNIQUE INDEX study_sessions_one_running_idx ON study_sessions (user_id)
    WHERE status = 'running';
ALTER TABLE study_sessions ENABLE ROW LEVEL SECURITY;
