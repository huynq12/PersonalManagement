//! Business rules for finance categories, transactions, and the monthly summary.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{
        Budget, Category, CreateCategoryRequest, CreateTransactionRequest, Kind,
        ListCategoriesQuery, Summary, Transaction, UpdateCategoryRequest, UpdateTransactionRequest,
    },
    repository::{self, CategoryChanges, TransactionChanges},
};
use crate::{
    error::{AppError, AppResult, is_unique_violation},
    validation::{field_error, parse_month},
};

/// Maps the unique violation on the name index to [`AppError::FinanceCategoryNameTaken`].
fn map_name_conflict(error: sqlx::Error) -> AppError {
    if is_unique_violation(&error) {
        AppError::FinanceCategoryNameTaken
    } else {
        AppError::Database(error)
    }
}

/// Parses the `month` query value.
fn month_start(month: &str) -> AppResult<chrono::NaiveDate> {
    parse_month(month).ok_or_else(|| field_error("month", "format"))
}

/// A blank note is the same as no note.
fn clean_note(note: &str) -> Option<&str> {
    Some(note.trim()).filter(|note| !note.is_empty())
}

/// Lists the user's categories by kind, then name.
pub(super) async fn list_categories(
    db: &PgPool,
    user_id: Uuid,
    query: ListCategoriesQuery,
) -> AppResult<Vec<Category>> {
    let kind = query.kind.map(Kind::as_str);
    Ok(repository::list_categories(db, user_id, kind, query.include_archived).await?)
}

/// Creates a category.
///
/// # Errors
///
/// Returns [`AppError::FinanceCategoryNameTaken`] if the user has a category of this kind with
/// this name, archived or not, ignoring case.
pub(super) async fn create_category(
    db: &PgPool,
    user_id: Uuid,
    request: CreateCategoryRequest,
) -> AppResult<Category> {
    repository::insert_category(
        db,
        user_id,
        request.kind.as_str(),
        request.name.trim(),
        &request.color,
    )
    .await
    .map_err(map_name_conflict)
}

/// Updates the given fields of a category.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such category, and
/// [`AppError::FinanceCategoryNameTaken`] if the new name belongs to another category of the
/// same kind.
pub(super) async fn update_category(
    db: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
    request: UpdateCategoryRequest,
) -> AppResult<Category> {
    let changes = CategoryChanges {
        name: request.name.as_deref().map(str::trim),
        color: request.color.as_deref(),
        archived: request.archived,
    };

    repository::update_category(db, user_id, category_id, changes, Utc::now())
        .await
        .map_err(map_name_conflict)?
        .ok_or(AppError::NotFound)
}

/// Deletes a category.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such category, and
/// [`AppError::FinanceCategoryInUse`] if a transaction uses it.
pub(super) async fn delete_category(
    db: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
) -> AppResult<()> {
    let deleted = repository::delete_category(db, user_id, category_id)
        .await
        .map_err(|error| {
            if error
                .as_database_error()
                .is_some_and(sqlx::error::DatabaseError::is_foreign_key_violation)
            {
                AppError::FinanceCategoryInUse
            } else {
                AppError::Database(error)
            }
        })?;
    if deleted {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

/// The month's transactions, newest first.
///
/// # Errors
///
/// Returns [`AppError::Validation`] on the `month` field with code `format` if `month` is not
/// `YYYY-MM`.
pub(super) async fn list_transactions(
    db: &PgPool,
    user_id: Uuid,
    month: &str,
) -> AppResult<Vec<Transaction>> {
    let rows = repository::list_transactions(db, user_id, month_start(month)?).await?;
    Ok(rows.into_iter().map(Transaction::from).collect())
}

/// Records a transaction.
///
/// # Errors
///
/// Returns [`AppError::Validation`] on the `category_id` field with code `not_found` if the
/// user has no such category, or `archived` if it is archived.
pub(super) async fn create_transaction(
    db: &PgPool,
    user_id: Uuid,
    request: CreateTransactionRequest,
) -> AppResult<Transaction> {
    let category = repository::find_category(db, user_id, request.category_id)
        .await?
        .ok_or_else(|| field_error("category_id", "not_found"))?;
    if category.archived_at.is_some() {
        return Err(field_error("category_id", "archived"));
    }

    let id = repository::insert_transaction(
        db,
        user_id,
        category.id,
        request.amount,
        request.date,
        request.note.as_deref().and_then(clean_note),
    )
    .await?;
    get_transaction(db, user_id, id).await
}

/// Updates the given fields of a transaction.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such transaction. A new category fails
/// with [`AppError::Validation`] on `category_id`: `not_found` if the user has no such
/// category, `kind_mismatch` if its kind differs from the transaction's, `archived` if it is
/// archived. Keeping the current category never fails on its archive state.
pub(super) async fn update_transaction(
    db: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
    request: UpdateTransactionRequest,
) -> AppResult<Transaction> {
    let current = repository::find_transaction(db, user_id, transaction_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let new_category_id = request
        .category_id
        .filter(|category_id| *category_id != current.category_id);
    if let Some(category_id) = new_category_id {
        let category = repository::find_category(db, user_id, category_id)
            .await?
            .ok_or_else(|| field_error("category_id", "not_found"))?;
        if category.kind != current.kind {
            return Err(field_error("category_id", "kind_mismatch"));
        }
        if category.archived_at.is_some() {
            return Err(field_error("category_id", "archived"));
        }
    }

    let changes = TransactionChanges {
        category_id: new_category_id,
        amount: request.amount,
        date: request.date,
        note: request
            .note
            .as_ref()
            .map(|note| note.as_deref().and_then(clean_note)),
    };
    if !repository::update_transaction(db, user_id, transaction_id, changes).await? {
        // Deleted by a concurrent request since it was read above.
        return Err(AppError::NotFound);
    }
    get_transaction(db, user_id, transaction_id).await
}

/// Reads one transaction back after a write.
async fn get_transaction(
    db: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
) -> AppResult<Transaction> {
    repository::find_transaction(db, user_id, transaction_id)
        .await?
        .map(Transaction::from)
        .ok_or(AppError::NotFound)
}

/// Deletes a transaction.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such transaction.
pub(super) async fn delete_transaction(
    db: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
) -> AppResult<()> {
    if repository::delete_transaction(db, user_id, transaction_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

/// The month's totals, balance, per-category breakdown, and budget use.
///
/// # Errors
///
/// Returns [`AppError::Validation`] on the `month` field with code `format` if `month` is not
/// `YYYY-MM`.
pub(super) async fn summary(db: &PgPool, user_id: Uuid, month: &str) -> AppResult<Summary> {
    let start = month_start(month)?;
    let by_category = repository::category_totals(db, user_id, start).await?;
    let budget = repository::monthly_budget(db, user_id).await?;

    let total_of = |kind: Kind| -> i64 {
        by_category
            .iter()
            .filter(|row| row.kind == kind.as_str())
            .map(|row| row.total)
            .sum()
    };
    let income_total = total_of(Kind::Income);
    let expense_total = total_of(Kind::Expense);

    Ok(Summary {
        income_total,
        expense_total,
        balance: income_total - expense_total,
        by_category,
        budget: budget.map(|amount| Budget {
            amount,
            spent: expense_total,
        }),
    })
}
