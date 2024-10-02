use std::sync::Arc;

use anyhow::Result;
use essentials::{error, info};
use slack_morphism::prelude::*;

use crate::{env::Env, exporter};

pub async fn run(app_token: String) -> Result<()> {
    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()?));
    let socket_mode_callbacks =
        SlackSocketModeListenerCallbacks::new().with_command_events(handle_command);
    let listener_environment = Arc::new(SlackClientEventsListenerEnvironment::new(client.clone()));
    let listener = SlackClientSocketModeListener::new(
        &SlackClientSocketModeConfig::new(),
        listener_environment.clone(),
        socket_mode_callbacks,
    );
    info!("Starting listener");
    listener
        .listen_for(&SlackApiToken::new(app_token.into()))
        .await?;
    info!("Listener started");
    listener.serve().await;
    info!("Listener stopped");
    Ok(())
}

async fn handle_command(
    event: SlackCommandEvent,
    client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> UserCallbackResult<SlackCommandEventResponse> {
    if event.command.0 != "/download" {
        return Ok(SlackCommandEventResponse::new(
            SlackMessageContent::new().with_text("Unknown command".into()),
        ));
    }
    if event.text.is_none() {
        return Ok(SlackCommandEventResponse::new(
            SlackMessageContent::new().with_text("Invalid command usage. You must select a project to be exported. (e.g. `/download bushfires`".into()),
        ));
    }
    tokio::spawn(async move {
        if let Err(err) = process_message(client, event).await {
            error!(?err, "Error processing message");
        }
    });
    Ok(SlackCommandEventResponse::new(
        SlackMessageContent::new()
            .with_text("Working on it... You will be notified once export is finished!".into()),
    ))
}

async fn process_message(client: Arc<SlackHyperClient>, event: SlackCommandEvent) -> Result<()> {
    let env = Env::new()?;
    let token = SlackApiToken::new(env.slack_bot_token.into());
    let mapping = &env.dirs_mapping.0;
    let session = client.open_session(&token);

    let project = event.text.unwrap_or_default();
    let path = match mapping.get(&project) {
        Some(path) => path,
        None => {
            session
                .chat_post_ephemeral(&SlackApiChatPostEphemeralRequest::new(
                    event.channel_id.clone(),
                    event.user_id,
                    SlackMessageContent::new()
                        .with_text(format!("Invalid project name: `{project}`")),
                ))
                .await?;
            return Ok(());
        }
    };

    let url = match exporter::export(path, env.google_dir_id).await {
        Ok(url) => url,
        Err(err) => {
            error!(?err, "Error exporting");
            session
                .chat_post_ephemeral(&SlackApiChatPostEphemeralRequest::new(
                    event.channel_id.clone(),
                    event.user_id,
                    SlackMessageContent::new().with_text("Error exporting".into()),
                ))
                .await?;
            return Ok(());
        }
    };
    session
        .chat_post_message(&SlackApiChatPostMessageRequest::new(
            event.channel_id,
            SlackMessageContent::new().with_blocks(vec![
                SlackBlock::Header(SlackHeaderBlock::new(format!("Exported {project}").into())),
                SlackBlock::Divider(SlackDividerBlock::new()),
                SlackBlock::Section(
                    SlackSectionBlock::new().with_text(SlackBlockText::MarkDown(
                        format!(
                            "Hey <@{}>,\n{project} has just been successfully exported!\n\nYou can download it <{url}|here>.",
                            event.user_id.0
                        )
                        .into(),
                    )),
                ),
            ]),
        ))
        .await?;
    Ok(())
}
