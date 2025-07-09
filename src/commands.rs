use serenity::all::User;

use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn byte(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;

    let user_id = ctx.author().id.get();
    let guild_id = ctx.guild_id().ok_or("error: no guild in ctx")?.get();

    let user = match db.get_user(user_id, guild_id)? {
        Some(u) => u,
        None => {
            // new user and/or guild
            if db.get_guild(guild_id)?.is_none() {
                db.insert_guild(guild_id, user_id)?;
            } else {
                db.update_last_user(guild_id, user_id)?;
            }

            db.insert_user(user_id, guild_id)?;

            ctx.say(format!(
                "<@{}> grabbed a byte! They now have 1 byte.",
                user_id,
            ))
            .await?;

            return Ok(());
        }
    };

    let guild = db.get_guild(guild_id)?.unwrap();

    println!("user: {} last: {}", user.id, guild.last_user_id);
    let difference = if user.id == guild.last_user_id {
        user.score * 2
    } else {
        1
    };

    let new_score = user.score + difference;

    db.update_user_score(user_id, guild_id, new_score)?;

    db.update_last_user(guild_id, user_id)?;

    ctx.say(format!(
        "<@{}> grabbed {} bytes! They now have {} bytes.",
        user_id, difference, new_score
    ))
    .await?;

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
        None => "user not found".to_owned(),
    };

    ctx.say(msg).await?;

    Ok(())
}
