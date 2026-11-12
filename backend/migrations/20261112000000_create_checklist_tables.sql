-- Checklist: habits, their completions, and to-dos. See SPEC.md section 5 ("Checklist").
--
-- Row level security is enabled with no policy, as for the account tables: the app connects as
-- the table owner and bypasses it, while Supabase's Data API roles get no access.

CREATE TABLE habits (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name text NOT NULL,
    -- ISO weekdays the habit applies to: 1 (Monday) to 7 (Sunday).
    weekdays smallint[] NOT NULL DEFAULT '{1,2,3,4,5,6,7}'
        CHECK (cardinality(weekdays) > 0 AND weekdays <@ '{1,2,3,4,5,6,7}'::smallint[]),
    sort_order integer NOT NULL DEFAULT 0,
    -- Set when the user hides the habit; its history is kept.
    archived_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX habits_user_id_sort_order_idx ON habits (user_id, sort_order);
ALTER TABLE habits ENABLE ROW LEVEL SECURITY;

CREATE TABLE habit_completions (
    habit_id uuid NOT NULL REFERENCES habits (id) ON DELETE CASCADE,
    -- Calendar date in the user's time zone.
    date date NOT NULL,
    completed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (habit_id, date)
);
ALTER TABLE habit_completions ENABLE ROW LEVEL SECURITY;

CREATE TABLE tasks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- Calendar date in the user's time zone the to-do belongs to.
    date date NOT NULL,
    title text NOT NULL,
    is_done boolean NOT NULL DEFAULT false,
    done_at timestamptz,
    sort_order integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX tasks_user_id_date_idx ON tasks (user_id, date);
ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;

-- The calendar date it is for a user at the instant `at`, in the user's time zone.
-- `at` is a parameter rather than now() so one request uses a single clock reading.
CREATE FUNCTION user_today(p_user_id uuid, p_at timestamptz) RETURNS date
LANGUAGE sql STABLE
AS $$
    SELECT (p_at AT TIME ZONE u.timezone)::date FROM users u WHERE u.id = p_user_id
$$;

-- Current streak of a habit: the number of consecutive scheduled days completed, counting back
-- from `p_today` (SPEC.md, "Calculation rules").
--
-- * Only days whose ISO weekday is in `p_weekdays` count; other days are skipped, and
--   completions recorded on them are ignored.
-- * `p_today` itself not being completed yet does not break the streak.
-- * The current schedule is applied to the past as well.
--
-- The streak is the number of scheduled completions after the most recent scheduled day
-- before `p_today` that was missed. Days before the first completion need no scan: they are
-- all missed, and nothing was completed before them.
CREATE FUNCTION habit_current_streak(p_habit_id uuid, p_weekdays smallint[], p_today date)
RETURNS integer
LANGUAGE sql STABLE
AS $$
    SELECT count(*)::integer
    FROM habit_completions c
    WHERE c.habit_id = p_habit_id
      AND c.date <= p_today
      AND EXTRACT(ISODOW FROM c.date)::smallint = ANY (p_weekdays)
      AND c.date > COALESCE(
          (
              SELECT max(day::date)
              FROM generate_series(
                  (SELECT min(first.date) FROM habit_completions first
                   WHERE first.habit_id = p_habit_id)::timestamp,
                  (p_today - 1)::timestamp,
                  interval '1 day'
              ) AS day
              WHERE EXTRACT(ISODOW FROM day)::smallint = ANY (p_weekdays)
                AND NOT EXISTS (
                    SELECT 1 FROM habit_completions done
                    WHERE done.habit_id = p_habit_id AND done.date = day::date
                )
          ),
          '-infinity'::date
      )
$$;

-- These helpers are for the app only; keep them out of reach of other database roles.
REVOKE EXECUTE ON FUNCTION user_today(uuid, timestamptz) FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION habit_current_streak(uuid, smallint[], date) FROM PUBLIC;
