use anyhow::Result;
use env::Env;

pub mod env;
mod exporter;
mod slack;

pub use exporter::authorize;

pub async fn start(env: Env) -> Result<()> {
    slack::run(env.slack_app_token).await?;
    Ok(())
}
