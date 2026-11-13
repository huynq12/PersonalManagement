-- Streaks now skip low-energy days for habits that are not kept on such days.
-- See SPEC.md, calculation rules "Scheduled day of a habit" and "Streak".
--
-- A day is scheduled for a habit when its ISO weekday is in `p_weekdays` and it is either a
-- normal day or a low-energy day of the habit's owner while the habit is kept (`p_low_energy`).
-- Everything else about the streak is unchanged:
--
-- * Unscheduled days are skipped, and completions recorded on them are ignored.
-- * `p_today` itself not being completed yet does not break the streak.
-- * The current settings are applied to the past as well.
--
-- `p_weekdays` and `p_low_energy` are parameters rather than read from `habits`: when the
-- function is called from `UPDATE ... RETURNING`, it would otherwise see the old row.
DROP FUNCTION habit_current_streak(uuid, smallint[], date);

CREATE FUNCTION habit_current_streak(
    p_habit_id uuid,
    p_user_id uuid,
    p_weekdays smallint[],
    p_low_energy boolean,
    p_today date
)
RETURNS integer
LANGUAGE sql STABLE
AS $$
    SELECT count(*)::integer
    FROM habit_completions c
    WHERE c.habit_id = p_habit_id
      AND c.date <= p_today
      AND EXTRACT(ISODOW FROM c.date)::smallint = ANY (p_weekdays)
      AND (
          p_low_energy
          OR NOT EXISTS (
              SELECT 1 FROM low_energy_days l
              WHERE l.user_id = p_user_id AND l.date = c.date
          )
      )
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
                AND (
                    p_low_energy
                    OR NOT EXISTS (
                        SELECT 1 FROM low_energy_days l
                        WHERE l.user_id = p_user_id AND l.date = day::date
                    )
                )
                AND NOT EXISTS (
                    SELECT 1 FROM habit_completions done
                    WHERE done.habit_id = p_habit_id AND done.date = day::date
                )
          ),
          '-infinity'::date
      )
$$;

-- This helper is for the app only; keep it out of reach of other database roles.
REVOKE EXECUTE ON FUNCTION habit_current_streak(uuid, uuid, smallint[], boolean, date)
    FROM PUBLIC;
