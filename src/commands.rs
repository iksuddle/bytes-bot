use poise::CreateReply;
use serenity::all::{Colour, CreateEmbed, User};

use crate::{Context, Error, create_embed_reply, create_embed_success};

/// Grab a byte!
#[poise::command(prefix_command)]
pub async fn byte(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;

    let user_id = ctx.author().id.get();
    let guild_id = ctx
        .guild_id()
        .ok_or("error: no guild in ctx".to_owned())?
        .get();

    let guild = match db.get_guild(guild_id)? {
        Some(g) => g,
        None => {
            db.insert_guild(guild_id, ctx.author().id.get())?;
            db.get_guild(guild_id)?.unwrap()
        }
    };

    // check for cooldown
    {
        let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();

        let cooldown_durations = poise::CooldownConfig {
            guild: Some(std::time::Duration::from_secs(guild.cooldown)),
            ..Default::default()
        };

        match cooldown_tracker.remaining_cooldown(ctx.cooldown_context(), &cooldown_durations) {
            Some(remaining) => {
                let total_seconds = remaining.as_secs();
                let minutes = total_seconds / 60;
                let seconds = total_seconds % 60;

                let msg = match minutes {
                    0 => format!("Please wait **{} seconds**", seconds),
                    1 => format!("Please wait **1 minute and {} seconds**", seconds),
                    _ => format!(
                        "Please wait **{} minutes and {} seconds**",
                        minutes, seconds
                    ),
                };

                return Err(msg.into());
            }
            None => cooldown_tracker.start_cooldown(ctx.cooldown_context()),
        }
    };

    let user = match db.get_user(user_id, guild_id)? {
        Some(u) => u,
        None => {
            db.insert_user(user_id, guild_id)?;
            db.update_last_user(guild_id, user_id)?;

            let msg = format!("<@{user_id}> grabbed a byte! They now have 1 byte.");

            ctx.send(create_embed_success(msg)).await?;

            return Ok(());
        }
    };

    let difference = if user.id == guild.last_user_id {
        user.score * 2
    } else {
        1
    };
    let new_score = user.score + difference;

    db.update_user_score(user_id, guild_id, new_score)?;
    db.update_last_user(guild_id, user_id)?;

    let msg = format!("<@{user_id}> grabbed {difference} bytes! They now have {new_score} bytes.");

    ctx.send(create_embed_success(msg)).await?;

    Ok(())
}

/// Check how many bytes other members have.
#[poise::command(prefix_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "the member in question"] member: Option<User>,
) -> Result<(), Error> {
    let db = &ctx.data().db;

    let member = member.unwrap_or(ctx.author().to_owned());

    let user_id = member.id.get();
    let guild_id = ctx.guild_id().expect("error: no guild in ctx").get();

    let user = db.get_user(user_id, guild_id)?;

    let msg = match user {
        Some(u) => format!("user <@{}> has {} bytes!", u.id, u.score),
        None => format!("user <@{}> has no bytes...", user_id),
    };

    ctx.send(create_embed_reply("Info".to_owned(), msg, Colour::BLUE))
        .await?;

    Ok(())
}

/// Change the byte cooldown for the server.
#[poise::command(prefix_command, required_permissions = "ADMINISTRATOR")]
pub async fn cooldown(
    ctx: Context<'_>,
    #[description = "the new cooldown time"] cooldown: Vec<String>,
) -> Result<(), Error> {
    let d = duration_str::parse(cooldown.join(" "))?.as_secs();
    let guild_id = ctx
        .guild_id()
        .ok_or("no guild in context".to_owned())?
        .get();

    let db = &ctx.data().db;
    if db.get_guild(guild_id)?.is_none() {
        db.insert_guild(guild_id, 0)?;
    }

    db.update_cooldown(guild_id, d)?;

    ctx.send(create_embed_success(format!(
        "cooldown updated to {d} seconds!"
    )))
    .await?;

    Ok(())
}

/// Display's the guild leaderboard
#[poise::command(prefix_command, aliases("lb"))]
pub async fn leaderboard(
    ctx: Context<'_>,
    #[description = "number of entries to show"] n: Option<u32>,
) -> Result<(), Error> {
    let db = &ctx.data().db;

    let users = db.get_leaderboard(n.unwrap_or(10))?;

    let mut content = String::new();

    for (i, u) in users.iter().enumerate() {
        content.push_str(format!("{}. <@{}> - {} bytes\n", i, u.id, u.score).as_str());
    }

    let lb_embed = CreateEmbed::new()
        .title("Leaderboard")
        .field(format!("Top {} members:", n.unwrap_or(10)), content, false)
        .colour(Colour::DARK_GREEN);

    ctx.send(CreateReply::default().embed(lb_embed)).await?;

    Ok(())
}

/// Show this help menu.
#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "Provide a command name to view more info.
You can edit your message to the bot and the bot will edit its response.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;

    Ok(())
}

// /// Set the byte master role.
// #[poise::command(prefix_command)]
// pub async fn role(
//     ctx: Context<'_>,
//     #[description = "The role to set for the byte master"] role: Option<Role>,
// ) -> Result<(), Error> {
//     let db = &ctx.data().db;
//
//     let guild_id = ctx.guild_id().expect("error: no guild in ctx").get();
//
//     let guild = match db.get_guild(guild_id)? {
//         Some(g) => g,
//         None => {
//             db.insert_guild(guild_id, 0)?;
//             db.get_guild(guild_id)?.unwrap()
//         }
//     };
//
//     let role_id = match role {
//         Some(r) => r.id.get(),
//         None => {
//             if let Some(id) = guild.master_role {
//                 let msg = format!("Byte master role is currently <@&{}>", id);
//                 let embed = create_embed_success(msg);
//                 ctx.send(embed).await?;
//             } else {
//                 let msg =
//                     format!("Byte master role not set for this server. Provide a role to set it.");
//                 let embed = create_embed_failure(msg);
//                 ctx.send(embed).await?;
//             }
//             return Ok(());
//         }
//     };
//
//     db.update_role(guild_id, role_id)?;
//
//     let msg = format!("Byte master role updated to <@&{}>", role_id);
//     let embed = create_embed_success(msg);
//
//     ctx.send(embed).await?;
//
//     Ok(())
// }
