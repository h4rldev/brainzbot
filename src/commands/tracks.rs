use poise::{
    CreateReply,
    serenity_prelude::{CreateEmbed, CreateEmbedAuthor, Member},
};

use crate::{
    api::{api_request, get_user},
    brainzbot::{BrainzContext, BrainzError},
};

// early testing
#[poise::command(slash_command)]
pub async fn nowplaying(ctx: BrainzContext<'_>, user: Option<Member>) -> Result<(), BrainzError> {
    let user = get_user(ctx, user).await.unwrap();
    let response = api_request(
        &user.token,
        format!("user/{}/playing-now", &user.username).as_str(),
    )
    .await;

    // it shouldve fetched
    // /metadata/recording/ for more complete info (release album art etc)
    match response {
        Ok(Some(data)) => {
            let data = &data["payload"]["listens"][0]["track_metadata"];
            println!("{data:?}");
            let _ = ctx
                .send(
                    CreateReply::default().embed(
                        CreateEmbed::new()
                            .author(CreateEmbedAuthor::new(format!(
                                "{} is listening to",
                                &user.username
                            )))
                            .description(format!(
                                "[{}]({})\n[{}]({})",
                                data["track_name"].as_str().unwrap(),
                                format!(
                                    "https://listenbrainz.org/track/{}",
                                    data["recording_mbid"]
                                ),
                                data["artist_name"].as_str().unwrap(),
                                format!(
                                    "https://listenbrainz.org/artist/{}",
                                    data["artist_mbids"][0]
                                ),
                            )),
                    ),
                )
                .await;
        }
        Ok(None) => {
            let _ = ctx
                .send(
                    CreateReply::default().embed(CreateEmbed::new().description(format!(
                        "{} isn't listening to any songs right now",
                        &user.username
                    ))),
                )
                .await;
        }
        Err(_) => {
            let _ = ctx
                .send(
                    CreateReply::default()
                        .embed(CreateEmbed::new().description("An error occured")),
                )
                .await;
        }
    };

    Ok(())
}
