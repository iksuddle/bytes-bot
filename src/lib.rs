use poise::CreateReply;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, params};
use serenity::all::{Colour, CreateEmbed};

pub mod commands;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("database error")]
    DbError(#[from] rusqlite::Error),
    #[error("discord error")]
    DiscordError(#[from] Box<serenity::Error>),
    #[error("{0}")]
    ByteError(String),
}

impl From<serenity::Error> for Error {
    fn from(value: serenity::Error) -> Self {
        Self::DiscordError(Box::new(value))
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::ByteError(value)
    }
}

pub struct User {
    id: DiscordId,
    _guild_id: DiscordId,
    score: u32,
}

pub struct Guild {
    _id: DiscordId,
    last_user_id: DiscordId,
    cooldown: u64,
}

pub struct ClientData {
    pub db: Database,
}

pub type Context<'a> = poise::Context<'a, ClientData, Error>;

type DiscordId = u64;

pub struct Database {
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new() -> Result<Self, Error> {
        let manager = SqliteConnectionManager::file("bytes.db3");
        let pool = r2d2::Pool::new(manager).expect("error creating conn pool");

        let db = Self { pool };

        let conn = db.get_pooled_connection();

        conn.pragma_update(None, "foreign_keys", "ON")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS guilds (
                id           INTEGER PRIMARY KEY,
                last_user_id INTEGER NOT NULL,
                cooldown     INTEGER DEFAULT 3600
            ) STRICT;

            CREATE TABLE IF NOT EXISTS users (
                id       INTEGER,
                guild_id INTEGER,
                score    INTEGER DEFAULT 1,
                PRIMARY KEY (id, guild_id),
                FOREIGN KEY (guild_id) REFERENCES guilds(id)
            ) STRICT;",
        )?;

        Ok(db)
    }

    fn get_pooled_connection(&self) -> r2d2::PooledConnection<SqliteConnectionManager> {
        self.pool.get().expect("error getting pool connection")
    }

    fn insert_guild(&self, id: DiscordId, user_id: DiscordId) -> Result<(), rusqlite::Error> {
        let conn = self.get_pooled_connection();

        conn.execute(
            "INSERT OR IGNORE INTO guilds (id, last_user_id) VALUES (?1, ?2)",
            params![id, user_id],
        )?;

        Ok(())
    }

    fn get_guild(&self, id: DiscordId) -> Result<Option<Guild>, rusqlite::Error> {
        let conn = self.get_pooled_connection();

        conn.query_one("SELECT * FROM guilds WHERE id = ?1", params![id], |row| {
            Ok(Guild {
                _id: row.get(0)?,
                last_user_id: row.get(1)?,
                cooldown: row.get(2)?,
            })
        })
        .optional()
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
                    _guild_id: row.get(1)?,
                    score: row.get(2)?,
                })
            },
        )
        .optional()
    }

    fn update_user_score(
        &self,
        user_id: DiscordId,
        guild_id: DiscordId,
        new_score: u32,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.get_pooled_connection();

        conn.execute(
            "UPDATE users
            SET score = ?1
            WHERE id = ?2 AND guild_id = ?3;",
            params![new_score, user_id, guild_id],
        )?;

        Ok(())
    }

    fn update_last_user(
        &self,
        guild_id: DiscordId,
        user_id: DiscordId,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.get_pooled_connection();

        conn.execute(
            "UPDATE guilds
            SET last_user_id = ?1
            WHERE id = ?2;",
            params![user_id, guild_id],
        )?;

        Ok(())
    }

    fn update_cooldown(&self, guild_id: DiscordId, cooldown: u64) -> Result<(), rusqlite::Error> {
        let conn = self.get_pooled_connection();

        conn.execute(
            "UPDATE guilds
            SET cooldown = ?1
            WHERE id = ?2;",
            params![cooldown, guild_id],
        )?;

        Ok(())
    }

    fn get_leaderboard(&self, n: u32) -> Result<Vec<User>, rusqlite::Error> {
        let conn = self.get_pooled_connection();

        let mut stmt = conn.prepare("SELECT * FROM users ORDER BY score DESC LIMIT ?1")?;
        let users: Vec<User> = stmt
            .query_map(params![n], |row| {
                Ok(User {
                    id: row.get(0)?,
                    _guild_id: row.get(1)?,
                    score: row.get(2)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();

        Ok(users)
    }
}

pub fn create_embed_success(msg: String) -> CreateReply {
    create_embed_reply("Success!".to_owned(), msg, Colour::DARK_GREEN)
}

pub fn create_embed_failure(msg: String) -> CreateReply {
    create_embed_reply("Uh oh!".to_owned(), msg, Colour::RED)
}

pub fn create_embed_reply(title: String, msg: String, colour: Colour) -> CreateReply {
    CreateReply::default().embed(
        CreateEmbed::new()
            .title(title)
            .description(msg)
            .colour(colour),
    )
}
