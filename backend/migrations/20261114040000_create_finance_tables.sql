-- Finance categories and transactions. See SPEC.md section 5 ("Finances").
--
-- Row level security is enabled with no policy, as for the other tables: the app connects as
-- the table owner and bypasses it, while Supabase's Data API roles get no access.

CREATE TABLE finance_categories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- Never changes after creation: a transaction takes its kind from its category.
    kind text NOT NULL CHECK (kind IN ('expense', 'income')),
    name text NOT NULL,
    -- Hex color in the form #RRGGBB; the palette to choose from is a UI concern.
    color text NOT NULL CHECK (color ~ '^#[0-9A-Fa-f]{6}$'),
    -- Set when the user hides the category; it keeps its name, so the name stays taken.
    archived_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
-- Names are unique per user and kind ignoring case, and archived categories count too.
CREATE UNIQUE INDEX finance_categories_user_id_kind_lower_name_idx
    ON finance_categories (user_id, kind, lower(name));
ALTER TABLE finance_categories ENABLE ROW LEVEL SECURITY;

CREATE TABLE transactions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- A category with transactions cannot be deleted; it can be archived instead.
    category_id uuid NOT NULL REFERENCES finance_categories (id) ON DELETE RESTRICT,
    -- Minor units of the account currency.
    amount bigint NOT NULL CHECK (amount > 0 AND amount <= 1000000000000),
    date date NOT NULL,
    note text CHECK (char_length(note) <= 500),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX transactions_user_id_date_idx ON transactions (user_id, date);
ALTER TABLE transactions ENABLE ROW LEVEL SECURITY;
