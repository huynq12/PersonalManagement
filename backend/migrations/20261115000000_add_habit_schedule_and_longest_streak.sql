-- Shared scheduling rule and the longest streak, for the weekly statistics.
-- See SPEC.md, calculation rules "Scheduled day of a habit" and "Streak".

-- A day is scheduled for a habit when its ISO weekday is in `p_weekdays` and it is either a
-- normal day or a low-energy day of the habit's owner while the habit is kept (`p_low_energy`).
-- The schedule is passed in rather than read from `habits`, like in `habit_current_streak`.
CREATE FUNCTION habit_scheduled_on(
    p_user_id uuid,
    p_weekdays smallint[],
    p_low_energy boolean,
    p_date date
)
RETURNS boolean
LANGUAGE sql STABLE
AS $$
    SELECT EXTRACT(ISODOW FROM p_date)::smallint = ANY (p_weekdays)
       AND (
           p_low_energy
           OR NOT EXISTS (
               SELECT 1 FROM low_energy_days l
               WHERE l.user_id = p_user_id AND l.date = p_date
           )
       )
$$;

-- The longest run of consecutive scheduled days the habit was completed, up to `p_today`.
-- Unscheduled days are skipped, like in `habit_current_streak`.
--
-- Every scheduled day that was not completed starts a new group, so the completed days of
-- one group are one run.
CREATE FUNCTION habit_longest_streak(
    p_habit_id uuid,
    p_user_id uuid,
    p_weekdays smallint[],
    p_low_energy boolean,
    p_today date
)
RETURNS integer
LANGUAGE sql STABLE
AS $$
    WITH scheduled AS (
        SELECT day::date AS day,
               EXISTS (
                   SELECT 1 FROM habit_completions c
                   WHERE c.habit_id = p_habit_id AND c.date = day::date
               ) AS done
        FROM generate_series(
            (SELECT min(c.date) FROM habit_completions c WHERE c.habit_id = p_habit_id)::timestamp,
            p_today::timestamp,
            interval '1 day'
        ) AS day
        WHERE habit_scheduled_on(p_user_id, p_weekdays, p_low_energy, day::date)
    ),
    grouped AS (
        SELECT done,
               count(*) FILTER (WHERE NOT done) OVER (ORDER BY day) AS run
        FROM scheduled
    )
    SELECT COALESCE(max(done_days), 0)::integer
    FROM (
        SELECT count(*) FILTER (WHERE done) AS done_days
        FROM grouped
        GROUP BY run
    ) AS runs
$$;

-- These helpers are for the app only; keep them out of reach of other database roles.
REVOKE EXECUTE ON FUNCTION habit_scheduled_on(uuid, smallint[], boolean, date) FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION habit_longest_streak(uuid, uuid, smallint[], boolean, date) FROM PUBLIC;
