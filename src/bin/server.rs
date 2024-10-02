use anyhow::Result;
use essentials::info;
use folder_exporter::env::Env;

#[tokio::main]
async fn main() -> Result<()> {
    essentials::install();
    let env = Env::new()?;
    info!("Application running");
    folder_exporter::start(env).await
}
