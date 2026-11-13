//! SQL for finance categories, transactions, and the monthly summary.
//!
//! Every query filters by `user_id`, so one user can never read or change another user's
//! data. Name uniqueness is enforced by the `finance_categories_user_id_kind_lower_name_idx`
//! index and the "category in use" rule by the foreign key of `transactions.category_id`; the
//! service turns their violations into domain errors.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::{Category, CategoryRef, CategoryTotal, TransactionRow};

/// Lists a user's categories by kind, then name ignoring case.
pub(super) async fn list_categories(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    kind: Option<&str>,
    include_archived: bool,
) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as!(
        Category,
        r#"
        SELECT id, kind, name, color, archived_at, created_at
        FROM finance_categories
        WHERE user_id = $1
          AND ($2::text IS NULL OR kind = $2)
          AND ($3 OR archived_at IS NULL)
        ORDER BY kind, lower(name), id
        "#,
        user_id,
        kind,
        include_archived,
    )
    .fetch_all(executor)
    .await
}

/// Finds one of the user's categories, archived or not.
pub(super) async fn find_category(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    category_id: Uuid,
) -> sqlx::Result<Option<Category>> {
    sqlx::query_as!(
        Category,
        r#"
        SELECT id, kind, name, color, archived_at, created_at
        FROM finance_categories
        WHERE id = $2 AND user_id = $1
        "#,
        user_id,
        category_id,
    )
    .fetch_optional(executor)
    .await
}

/// Inserts a category. Fails with a unique violation if the user already has the name within
/// the kind.
pub(super) async fn insert_category(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    kind: &str,
    name: &str,
    color: &str,
) -> sqlx::Result<Category> {
    sqlx::query_as!(
        Category,
        r#"
        INSERT INTO finance_categories (user_id, kind, name, color)
        VALUES ($1, $2, $3, $4)
        RETURNING id, kind, name, color, archived_at, created_at
        "#,
        user_id,
        kind,
        name,
        color,
    )
    .fetch_one(executor)
    .await
}

/// Changes of `PATCH /finance-categories/{id}`; `None` keeps the current value.
pub(super) struct CategoryChanges<'a> {
    pub name: Option<&'a str>,
    pub color: Option<&'a str>,
    pub archived: Option<bool>,
}

/// Updates a category. Returns `None` if the user has no such category. Fails with a unique
/// violation if the new name belongs to another category of the same kind.
pub(super) async fn update_category(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    category_id: Uuid,
    changes: CategoryChanges<'_>,
    now: DateTime<Utc>,
) -> sqlx::Result<Option<Category>> {
    sqlx::query_as!(
        Category,
        r#"
        UPDATE finance_categories
        SET name = COALESCE($3, name),
            color = COALESCE($4, color),
            archived_at = CASE
                WHEN $5::boolean IS NULL THEN archived_at
                -- Archiving twice keeps the first archive time.
                WHEN $5 THEN COALESCE(archived_at, $6)
                ELSE NULL
            END
        WHERE id = $2 AND user_id = $1
        RETURNING id, kind, name, color, archived_at, created_at
        "#,
        user_id,
        category_id,
        changes.name,
        changes.color,
        changes.archived,
        now,
    )
    .fetch_optional(executor)
    .await
}

/// Deletes a category. Returns whether it existed. Fails with a foreign key violation if a
/// transaction uses it.
pub(super) async fn delete_category(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    category_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM finance_categories WHERE id = $2 AND user_id = $1",
        user_id,
        category_id,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// The transactions of the month starting at `month_start`, newest date first, then newest
/// created first.
pub(super) async fn list_transactions(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    month_start: NaiveDate,
) -> sqlx::Result<Vec<TransactionRow>> {
    sqlx::query_as!(
        TransactionRow,
        r#"
        SELECT t.id, c.kind, t.amount, t.date, t.note, t.created_at,
               c.id AS category_id, c.name AS category_name, c.color AS category_color
        FROM transactions t
        JOIN finance_categories c ON c.id = t.category_id
        WHERE t.user_id = $1
          AND t.date >= $2 AND t.date < ($2::date + interval '1 month')::date
        ORDER BY t.date DESC, t.created_at DESC, t.id DESC
        "#,
        user_id,
        month_start,
    )
    .fetch_all(executor)
    .await
}

/// Finds one of the user's transactions with its category.
pub(super) async fn find_transaction(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    transaction_id: Uuid,
) -> sqlx::Result<Option<TransactionRow>> {
    sqlx::query_as!(
        TransactionRow,
        r#"
        SELECT t.id, c.kind, t.amount, t.date, t.note, t.created_at,
               c.id AS category_id, c.name AS category_name, c.color AS category_color
        FROM transactions t
        JOIN finance_categories c ON c.id = t.category_id
        WHERE t.id = $2 AND t.user_id = $1
        "#,
        user_id,
        transaction_id,
    )
    .fetch_optional(executor)
    .await
}

/// Inserts a transaction and returns its ID.
pub(super) async fn insert_transaction(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    category_id: Uuid,
    amount: i64,
    date: NaiveDate,
    note: Option<&str>,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        r#"
        INSERT INTO transactions (user_id, category_id, amount, date, note)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        user_id,
        category_id,
        amount,
        date,
        note,
    )
    .fetch_one(executor)
    .await
}

/// Changes of `PATCH /transactions/{id}`; `None` keeps the current value.
pub(super) struct TransactionChanges<'a> {
    pub category_id: Option<Uuid>,
    pub amount: Option<i64>,
    pub date: Option<NaiveDate>,
    /// Outer `None` keeps the note; `Some(None)` clears it.
    pub note: Option<Option<&'a str>>,
}

/// Updates a transaction. Returns whether it existed.
pub(super) async fn update_transaction(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    transaction_id: Uuid,
    changes: TransactionChanges<'_>,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE transactions
        SET category_id = COALESCE($3, category_id),
            amount = COALESCE($4, amount),
            date = COALESCE($5, date),
            note = CASE WHEN $6 THEN $7 ELSE note END
        WHERE id = $2 AND user_id = $1
        "#,
        user_id,
        transaction_id,
        changes.category_id,
        changes.amount,
        changes.date,
        changes.note.is_some(),
        changes.note.flatten(),
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Deletes a transaction. Returns whether it existed.
pub(super) async fn delete_transaction(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    transaction_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM transactions WHERE id = $2 AND user_id = $1",
        user_id,
        transaction_id,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// The month's totals per category, only for categories with transactions in the month and
/// archived ones included. Expenses come first, then the largest total first.
pub(super) async fn category_totals(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    month_start: NaiveDate,
) -> sqlx::Result<Vec<CategoryTotal>> {
    let rows = sqlx::query!(
        r#"
        SELECT c.kind, c.id, c.name, c.color, sum(t.amount)::int8 AS "total!"
        FROM transactions t
        JOIN finance_categories c ON c.id = t.category_id
        WHERE t.user_id = $1
          AND t.date >= $2 AND t.date < ($2::date + interval '1 month')::date
        GROUP BY c.id
        ORDER BY c.kind, 5 DESC, lower(c.name), c.id
        "#,
        user_id,
        month_start,
    )
    .fetch_all(executor)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| CategoryTotal {
            kind: row.kind,
            category: CategoryRef {
                id: row.id,
                name: row.name,
                color: row.color,
            },
            total: row.total,
        })
        .collect())
}

/// The user's monthly expense budget, or `None` when none is set.
pub(super) async fn monthly_budget(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
) -> sqlx::Result<Option<i64>> {
    let budget = sqlx::query_scalar!(
        "SELECT monthly_budget FROM user_settings WHERE user_id = $1",
        user_id,
    )
    .fetch_optional(executor)
    .await?;
    Ok(budget.flatten())
}
