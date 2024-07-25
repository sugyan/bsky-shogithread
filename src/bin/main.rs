use bsky_sdk::agent::config::{Config, FileStore};
use bsky_sdk::BskyAgent;
use bsky_shogithread::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let store = FileStore::new("config.json");
    let agent = BskyAgent::builder()
        .config(Config::load(&store).await?)
        .build()
        .await?;
    agent.to_config().await.save(&store).await?;

    Bot::new(agent).run().await
}
