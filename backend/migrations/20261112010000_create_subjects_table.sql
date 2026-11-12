-- Subjects a user studies in Pomodoro sessions. See SPEC.md section 5 ("Pomodoro").
--
-- Row level security is enabled with no policy, as for the other tables: the app connects as
-- the table owner and bypasses it, while Supabase's Data API roles get no access.

CREATE TABLE subjects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name text NOT NULL,
    -- Hex color in the form #RRGGBB; the palette to choose from is a UI concern.
    color text NOT NULL CHECK (color ~ '^#[0-9A-Fa-f]{6}$'),
    -- Set when the user hides the subject; it keeps its name, so the name stays taken.
    archived_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
-- Names are unique per user ignoring case, and archived subjects count too.
CREATE UNIQUE INDEX subjects_user_id_lower_name_idx ON subjects (user_id, lower(name));
ALTER TABLE subjects ENABLE ROW LEVEL SECURITY;
