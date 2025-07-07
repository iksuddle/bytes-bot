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
    score: u32,
    streak: u32,
    guild_id: DiscordId,
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

        let conn = pool.get().expect("error getting pool connection");

        conn.pragma_update(None, "foreign_keys", "ON")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id        INTEGER PRIMARY KEY,
                score     INTEGER NOT NULL DEFAULT 0,
                streak    INTEGER NOT NULL DEFAULT 0,
                guild_id  INTEGER NOT NULL,
                FOREIGN KEY (guild_id) REFERENCES guilds(id)
            ) STRICT;

            CREATE TABLE IF NOT EXISTS guilds (
                id           INTEGER PRIMARY KEY,
                last_user_id INTEGER,
                FOREIGN KEY (last_user_id) REFERENCES users(id)
            ) STRICT;",
        )?;

        Ok(Database { pool })
    }

    fn get_pooled_connection(&self) -> r2d2::PooledConnection<SqliteConnectionManager> {
        self.pool.get().expect("error getting pool connection")
    }

    pub fn get_user(&self, user_id: DiscordId) -> Result<Option<User>, rusqlite::Error> {
        let conn = self.get_pooled_connection();
        conn.query_row(
            "SELECT * FROM users WHERE id = ?1",
            params![user_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    score: row.get(1)?,
                    streak: row.get(2)?,
                    guild_id: row.get(3)?,
                })
            },
        )
        .optional()
    }

    pub fn get_guild(&self, guild_id: DiscordId) -> Result<Option<Guild>, rusqlite::Error> {
        let conn = self.get_pooled_connection();
        conn.query_row(
            "SELECT * FROM guilds WHERE id = ?1",
            params![guild_id],
            |row| {
                Ok(Guild {
                    id: row.get(0)?,
                    last_user_id: row.get(1)?,
                })
            },
        )
        .optional()
    }

    pub fn insert_new_user(
        &self,
        user_id: DiscordId,
        guild_id: DiscordId,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.get_pooled_connection();
        conn.execute(
            "INSERT INTO users (id, guild_id) VALUES (?1, ?2)",
            params![user_id, guild_id],
        )?;
        Ok(())
    }

    pub fn insert_new_guild(&self, guild_id: DiscordId) -> Result<(), rusqlite::Error> {
        let conn = self.get_pooled_connection();
        conn.execute("INSERT INTO guilds (id) VALUES (?1)", params![guild_id])?;
        Ok(())
    }
}
