use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, params};

// todo: Database Connection is not Send or Sync use pool
pub struct ClientData {
    pub db: Database,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, ClientData, Error>;

pub mod commands;

type DiscordId = u64;

pub struct User {
    id: DiscordId,
    guild_id: DiscordId,
    score: u32,
    streak: u32,
}

pub struct Guild {
    id: DiscordId,
    last_user_id: Option<DiscordId>,
}

pub struct Database {
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new() -> Result<Self, Error> {
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(manager).expect("error creating conn pool");

        let db = Self { pool };

        let conn = db.get_pooled_connection();

        conn.pragma_update(None, "foreign_keys", "ON")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS guilds (
                id           INTEGER PRIMARY KEY,
                last_user_id INTEGER
            ) STRICT;

            CREATE TABLE IF NOT EXISTS users (
                id       INTEGER,
                guild_id INTEGER,
                score    INTEGER DEFAULT 1,
                streak   INTEGER DEFAULT 1,
                PRIMARY KEY (id, guild_id),
                FOREIGN KEY (guild_id) REFERENCES guilds(id)
            ) STRICT;",
        )?;

        Ok(db)
    }

    fn get_pooled_connection(&self) -> r2d2::PooledConnection<SqliteConnectionManager> {
        self.pool.get().expect("error getting pool connection")
    }

    fn insert_user(&self, user_id: DiscordId, guild_id: DiscordId) -> Result<(), rusqlite::Error> {
        let conn = self.get_pooled_connection();

        // ensure guild exists
        conn.execute(
            "INSERT OR IGNORE INTO guilds (id) VALUES (?1)",
            params![guild_id],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO users (id, guild_id) VALUES (?1, ?2)",
            params![user_id, guild_id],
        )?;

        Ok(())
    }

    fn get_user(
        &self,
        id: DiscordId,
        guild_id: DiscordId,
    ) -> Result<Option<User>, rusqlite::Error> {
        let conn = self.get_pooled_connection();

        conn.query_one(
            "SELECT * FROM users WHERE id = ?1 AND guild_id = ?2",
            params![id, guild_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    guild_id: row.get(1)?,
                    score: row.get(2)?,
                    streak: row.get(3)?,
                })
            },
        )
        .optional()
    }
}
