use brainzbot::{Brainz, BrainzContext, BrainzError};
use dotenvy::dotenv;
use poise::{
    Framework, FrameworkOptions, PrefixFrameworkOptions, builtins::register_globally,
    serenity_prelude as serenity,
};
use redis::Client as RedisClient;
use reqwest::Client as HttpClient;
use serenity::{ClientBuilder, GatewayIntents};
use std::env;

use crate::commands::login::login;

mod api;
mod brainzbot;
mod commands;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let valkey_url = env::var("VALKEY_URL").expect("missing VALKEY_URL");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let http = HttpClient::new();
    let valkey = RedisClient::open(valkey_url.as_str()).unwrap();
    let conn = valkey.get_multiplexed_async_connection().await.unwrap();

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![login()],
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
