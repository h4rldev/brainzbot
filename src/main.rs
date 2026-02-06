use brainzbot::{Brainz, BrainzContext, BrainzError};
use poise::{
    Framework, FrameworkOptions, PrefixFrameworkOptions, builtins::register_globally,
    serenity_prelude as serenity,
};
use redis::Client as RedisClient;
use reqwest::Client as HttpClient;
use serenity::{ClientBuilder, GatewayIntents};
use std::env;

mod brainzbot;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: BrainzContext<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), BrainzError> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let redis_url = env::var("REDIS_URL").expect("missing REDIS_URL");

    let intents = GatewayIntents::non_privileged();
    let http = HttpClient::new();
    let valkey = RedisClient::open(redis_url.as_str()).unwrap();
    let conn = valkey.get_multiplexed_async_connection().await.unwrap();

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![age()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("%".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                register_globally(ctx, &framework.options().commands).await?;
                Ok(Brainz::new(http, conn))
            })
        })
        .build();

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap_or_else(|_| panic!("Failed to create client"));

    client
        .start()
        .await
        .unwrap_or_else(|_| panic!("Failed to start client"));
}
