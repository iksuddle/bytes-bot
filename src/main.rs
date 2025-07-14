use bytes_bot::{ClientData, Database, commands, create_embed_failure};
use poise::{FrameworkError, serenity_prelude as serenity};
use serenity::all::GatewayIntents;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    let db = Database::new().expect("error creating db");

    let token = secret_store
        .get("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN not set in secrets file");

    let intents = serenity::GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::byte(),
                commands::info(),
                commands::cooldown(),
                commands::leaderboard(),
                commands::help(),
                // commands::role(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".to_owned()),
                ..Default::default()
            },
            manual_cooldowns: true,
            on_error: |err| {
                Box::pin(async move {
                    match err {
                        FrameworkError::Command { error, ctx, .. } => {
                            ctx.send(create_embed_failure(error.to_string())).await.ok();
                        }
                        FrameworkError::ArgumentParse { error, ctx, .. } => {
                            ctx.send(create_embed_failure(error.to_string())).await.ok();
                        }
                        _ => {
                            let _ = poise::builtins::on_error(err).await;
                        }
                    };
                })
            },
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                // register commands in all guilds
                // poise::builtins::register_globally(ctx, &[commands::byte()]).await?;
                Ok(ClientData { db })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
