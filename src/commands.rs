use std::ptr::NonNull;

use serenity::{all::User, model::user};

use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn byte(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;

    let user_id = ctx.author().id.get();
    let guild_id = ctx.guild_id().expect("error: no guild in ctx").get();

    db.insert_user(user_id, guild_id)?;
    let user = db.get_user(user_id, guild_id)?.unwrap();

    let response = format!(
        "<@{}> grabbed a byte! They now have {} bytes.",
        user.id, user.score
    );

    ctx.say(response).await?;

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn info(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let db = &ctx.data().db;

    let user_id = user.id.get();
    let guild_id = ctx.guild_id().expect("error: no guild in ctx").get();

    let user = db.get_user(user_id, guild_id)?;

    let msg = match user {
        Some(u) => format!("user <@{}> has {} bytes!", u.id, u.score),
        None => format!("user not found"),
    };

    ctx.say(msg).await?;

    Ok(())
}
