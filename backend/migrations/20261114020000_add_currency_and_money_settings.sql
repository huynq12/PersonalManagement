-- Currency, work settings, and the monthly budget. See SPEC.md, section 5 ("Account") and the
-- cross-cutting decision "Money". Amounts are integers in the minor unit of the currency.

ALTER TABLE users
    -- ISO 4217 code from the supported list. Changing it never converts existing amounts.
    ADD COLUMN currency text NOT NULL DEFAULT 'VND'
        CONSTRAINT users_currency_supported
        CHECK (currency IN ('VND', 'USD', 'EUR', 'GBP', 'JPY', 'KRW', 'CNY', 'SGD', 'THB', 'AUD'));

ALTER TABLE user_settings
    -- Pay per hour in minor units; null until the user sets it.
    ADD COLUMN hourly_wage bigint
        CONSTRAINT user_settings_hourly_wage_range
        CHECK (hourly_wage BETWEEN 1 AND 1000000000000),
    -- Usual hours per working day; only used to label a day above, equal to, or below.
    ADD COLUMN base_work_hours numeric(4, 2)
        CONSTRAINT user_settings_base_work_hours_range
        CHECK (base_work_hours > 0 AND base_work_hours <= 24),
    -- Monthly expense budget in minor units; null means no budget.
    ADD COLUMN monthly_budget bigint
        CONSTRAINT user_settings_monthly_budget_range
        CHECK (monthly_budget BETWEEN 1 AND 1000000000000),
    -- Upper bounds for the Pomodoro fields, which the original table left open.
    ADD CONSTRAINT user_settings_focus_minutes_max CHECK (focus_minutes <= 240),
    ADD CONSTRAINT user_settings_short_break_minutes_max CHECK (short_break_minutes <= 240),
    ADD CONSTRAINT user_settings_long_break_minutes_max CHECK (long_break_minutes <= 240),
    ADD CONSTRAINT user_settings_long_break_every_max CHECK (long_break_every <= 12);
