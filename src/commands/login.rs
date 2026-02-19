use std::time::{Duration, SystemTime, UNIX_EPOCH};

use poise::{
    CreateReply, execute_modal_on_component_interaction,
    serenity_prelude::{
        ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
        CreateEmbedFooter,
    },
};
use tokio::time::sleep;

use crate::{
    api::{ApiError, verify_token},
    brainzbot::{BrainzContext, BrainzError},
};

#[derive(Debug, poise::Modal)]
#[name = "ListenBrainz Login"]
struct TokenModal {
    #[name = "User Token"]
    #[min_length = 36]
    #[max_length = 36]
    #[placeholder = "00000000-0000-0000-0000-000000000000"]
    token: String,
}

#[poise::command(slash_command)]
pub async fn login(ctx: BrainzContext<'_>) -> Result<(), BrainzError> {
    let embed = CreateEmbed::new()
        .title("Welcome to Brainzbot")
        .description("Here's how to link your account to Brainzbot!\nMake sure you already logged into ListenBrainz, and visit the [User Settings](https://listenbrainz.org/settings/)\nFrom here, you can copy the user token and provide the token by clicking the button below");

    let button = CreateButton::new("open_token_modal").label("Login with Access Token");
    let components = CreateActionRow::Buttons(vec![button]);

    let msg = ctx
        .send(
            CreateReply::default()
                .embed(embed)
                .components(vec![components])
                .ephemeral(true),
        )
        .await?;

    let edit_embed = |embed: CreateEmbed| async {
        msg.edit(ctx, CreateReply::default().embed(embed).components(vec![]))
            .await
    };

    // TODO: can open the modal again by clicking the button even after the user has closed the
    // modal(click the x button/click outside the window) as long as it hasn't reached the timeout
    while let Some(mci) = ComponentInteractionCollector::new(ctx.serenity_context())
        .filter(|mci| mci.data.custom_id == "open_token_modal")
        .await
    {
        let modal_response = execute_modal_on_component_interaction::<TokenModal>(
            ctx,
            mci,
            None,
            Some(Duration::from_secs(10)),
        )
        .await?;
        let token = match modal_response {
            Some(m) => m.token,
            None => {
                edit_embed(
                    CreateEmbed::new()
                        .title("Failed")
                        .description("No token was provided. Please run `/login` again"),
                )
                .await?;
                return Ok(());
            }
        };

        edit_embed(
            CreateEmbed::new()
                .title("Verifying token")
                .description("Please wait while we verifying your token"),
        )
        .await?;

        match verify_token(token).await {
            Ok(username) => {
                edit_embed(
                    CreateEmbed::new()
                        .title("Success")
                        .description("You have successfully logged into ListenBrainz")
                        .footer(CreateEmbedFooter::new(format!("Username: {}", username))),
                )
                .await?
                // TODO: save the token (persumably hashed/encrypted) to db and use discord userid for the key
            }
            Err(ApiError::TokenInvalid) => {
                edit_embed(CreateEmbed::new().title("Failed").description(
                    "Please make sure you entered the correct token and run `/login` again",
                ))
                .await?;
            }
            Err(ApiError::RateLimited(dur)) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_secs();
                let epoch = now + dur.as_secs();
                edit_embed(
                    CreateEmbed::new()
                        .title("Failed")
                        .description(format!("Too fast! Please try again <t:{}:R>", epoch)),
                )
                .await?;

                sleep(dur).await;
                let _ = msg.delete(ctx).await;
            }
            Err(ApiError::ConnectionError(err)) => {
                edit_embed(
                    CreateEmbed::new()
                        .title("Failed")
                        .description(format!("Something is wrong with the connection: {}", err)),
                )
                .await?;
            }
            Err(ApiError::DatatypeMismatch) => {
                edit_embed(
                    CreateEmbed::new()
                        .title("Failed")
                        .description("Something unknown error occured"),
                )
                .await?;
            }
        };
    }
    Ok(())
}
