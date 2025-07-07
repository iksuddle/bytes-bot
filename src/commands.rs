use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn byte(ctx: Context<'_>) -> Result<(), Error> {
    let response = format!("{} grabbed a byte!", ctx.author().name);
    ctx.say(response).await?;
    Ok(())
}
