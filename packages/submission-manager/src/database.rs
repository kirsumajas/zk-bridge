use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DepositRecord {
    pub deposit_id: String,
    pub ton_tx_hash: String,
    pub sender_address: String,
    pub recipient_solana: String,
    pub amount: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone)] 
pub struct DatabaseService {
    pool: SqlitePool,
}

impl DatabaseService {
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        // Create table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deposits (
                deposit_id TEXT PRIMARY KEY,
                ton_tx_hash TEXT NOT NULL,
                sender_address TEXT NOT NULL,
                recipient_solana TEXT NOT NULL,
                amount TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                error_message TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn store_deposit(&self, mut deposit: DepositRecord) -> Result<(), sqlx::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        deposit.created_at = now;
        deposit.updated_at = now;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO deposits 
            (deposit_id, ton_tx_hash, sender_address, recipient_solana, amount, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&deposit.deposit_id)
        .bind(&deposit.ton_tx_hash)
        .bind(&deposit.sender_address)
        .bind(&deposit.recipient_solana)
        .bind(&deposit.amount)
        .bind(&deposit.status)
        .bind(deposit.created_at)
        .bind(deposit.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pending_deposits(&self) -> Result<Vec<DepositRecord>, sqlx::Error> {
        let deposits = sqlx::query_as::<_, DepositRecord>(
            "SELECT * FROM deposits WHERE status = 'pending' ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(deposits)
    }

    pub async fn update_deposit_status(
        &self,
        deposit_id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(
            "UPDATE deposits SET status = ?, error_message = ?, updated_at = ? WHERE deposit_id = ?",
        )
        .bind(status)
        .bind(error_message)
        .bind(now)
        .bind(deposit_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_queue_stats(&self) -> Result<(usize, usize), sqlx::Error> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM deposits")
            .fetch_one(&self.pool)
            .await?;

        let completed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM deposits WHERE status = 'completed'")
            .fetch_one(&self.pool)
            .await?;

        Ok((total.0 as usize, completed.0 as usize))
    }
}