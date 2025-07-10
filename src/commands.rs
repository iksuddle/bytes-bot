use serenity::all::User;

use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn byte(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;

    let user_id = ctx.author().id.get();
    let guild_id = ctx.guild_id().ok_or("error: no guild in ctx")?.get();

    let guild = match db.get_guild(guild_id)? {
        Some(g) => g,
        None => {
            db.insert_guild(guild_id, ctx.author().id.get())?;
            db.get_guild(guild_id)?.unwrap()
        }
    };

    // check cooldown
    {
        let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();

        let mut cooldown_durations = poise::CooldownConfig::default();
        cooldown_durations.guild = Some(std::time::Duration::from_secs(guild.cooldown));

        match cooldown_tracker.remaining_cooldown(ctx.cooldown_context(), &cooldown_durations) {
            Some(remaining) => {
                return Err(format!("Please wait {} seconds", remaining.as_secs()).into());
            }
            // todo: call start_cooldown after user grabs byte
            None => cooldown_tracker.start_cooldown(ctx.cooldown_context()),
        };
    }

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

#[poise::command(prefix_command)]
pub async fn cooldown(ctx: Context<'_>, cooldown: Vec<String>) -> Result<(), Error> {
    let d = duration_str::parse(cooldown.join(" "))?.as_secs();

    let guild_id = ctx.guild_id().ok_or("error: no guild in ctx")?.get();

    // set server cooldown
    let db = &ctx.data().db;
    db.update_cooldown(guild_id, d)?;

    ctx.say(format!("cooldown updated to {} seconds.", d))
        .await?;

    Ok(())
}
