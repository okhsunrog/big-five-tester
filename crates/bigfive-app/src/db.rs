//! Database module using Turso (embedded SQLite).
//!
//! Stores personality test results for shareable URLs.

use anyhow::{Context, Result};
use bigfive::PersonalityProfile;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::info;
use turso::{Builder, Connection, Database};

/// Global database instance
static DATABASE: OnceCell<Arc<Database>> = OnceCell::const_new();

/// A saved test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedResult {
    pub id: String,
    pub profile: PersonalityProfile,
    pub user_context: Option<String>,
    pub ai_analysis: Option<String>,
    pub lang: String,
    pub created_at: i64,
}

/// Initialize the database and create tables.
pub async fn init_database(path: &str) -> Result<()> {
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).context("Failed to create database directory")?;
    }

    let db = Builder::new_local(path)
        .build()
        .await
        .context("Failed to open database")?;

    let conn = db.connect().context("Failed to connect to database")?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS results (
            id TEXT PRIMARY KEY,
            profile_json TEXT NOT NULL,
            user_context TEXT,
            ai_analysis TEXT,
            lang TEXT NOT NULL DEFAULT 'en',
            created_at INTEGER NOT NULL
        )
        "#,
        (),
    )
    .await
    .context("Failed to create results table")?;

    DATABASE
        .set(Arc::new(db))
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    info!("Database initialized at {}", path);
    Ok(())
}

/// Get a database connection.
pub fn get_connection() -> Result<Connection> {
    let db = DATABASE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

    db.connect().context("Failed to get database connection")
}

/// Save a test result snapshot to the database.
pub async fn save_result(
    id: &str,
    profile: &PersonalityProfile,
    user_context: Option<&str>,
    ai_analysis: Option<&str>,
    lang: &str,
) -> Result<()> {
    let conn = get_connection()?;
    let profile_json = serde_json::to_string(profile).context("Failed to serialize profile")?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("System time error")?
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO results (id, profile_json, user_context, ai_analysis, lang, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        (id, profile_json.as_str(), user_context.unwrap_or(""), ai_analysis.unwrap_or(""), lang, now),
    )
    .await
    .context("Failed to insert result")?;

    Ok(())
}

/// Get a saved result by ID.
pub async fn get_result(id: &str) -> Result<Option<SavedResult>> {
    let conn = get_connection()?;

    let mut rows = conn
        .query(
            "SELECT id, profile_json, user_context, ai_analysis, lang, created_at FROM results WHERE id = ?",
            [id],
        )
        .await
        .context("Failed to query result")?;

    if let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let profile_json: String = row.get(1)?;
        let user_context: String = row.get(2)?;
        let ai_analysis: Option<String> = row.get::<String>(3).ok().filter(|s| !s.is_empty());
        let lang: String = row.get(4)?;
        let created_at: i64 = row.get(5)?;

        let profile: PersonalityProfile =
            serde_json::from_str(&profile_json).context("Failed to deserialize profile")?;

        let user_context = if user_context.is_empty() {
            None
        } else {
            Some(user_context)
        };

        Ok(Some(SavedResult {
            id,
            profile,
            user_context,
            ai_analysis,
            lang,
            created_at,
        }))
    } else {
        Ok(None)
    }
}
