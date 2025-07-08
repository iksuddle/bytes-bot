use bytes::{ClientData, Database, Error, commands};
use poise::serenity_prelude as serenity;
use serenity::all::GatewayIntents;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db = Database::new()?;

    dotenv::dotenv().ok();

    let token = dotenv::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");

    let intents = serenity::GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::byte(), commands::info()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // register commands in all guilds
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(ClientData { db })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();

    Ok(())
}
