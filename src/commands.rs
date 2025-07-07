use serenity::all::User;

use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn byte(ctx: Context<'_>) -> Result<(), Error> {
    let response = format!("{} grabbed a byte!", ctx.author().name);
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn info(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let db = &ctx.data().db;

    let user_id = user.id.get();
    let guild_id = ctx.guild_id().expect("error").get();
    db.insert_new_guild(guild_id)?;
    db.insert_new_user(user_id, guild_id)?;

    let user = db.get_user(user_id)?.unwrap();

    ctx.say(format!("user {} has {} bytes!", user.id, user.score))
        .await?;
    Ok(())
}
