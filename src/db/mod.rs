use diesel::pg::PgConnection;
use diesel_async::{
    AsyncPgConnection,
    AsyncConnection,
    pooled_connection::AsyncDieselConnectionManager,
};
use bb8::Pool;
use crate::config::Config;

pub mod models;
pub mod schema;

pub type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

// Should be:
pub async fn establish_connection(config: &Config) -> anyhow::Result<DbPool> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(20)
        .build(config)
        .await?;
    
    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> anyhow::Result<()> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
    
    let mut conn = pool.get().await?;
    conn.run_pending_migrations(MIGRATIONS)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection() {
        let test_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/nousflash_test".to_string());
            
        let pool = establish_connection(&test_url).await.unwrap();
        assert!(pool.get().await.is_ok());
    }
}