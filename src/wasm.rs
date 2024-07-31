use crate::{Bot, Error, Result};
use bsky_sdk::{agent::config::Config, BskyAgent};
use log::Level;
use serde::Deserialize;
use std::sync::Once;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

static ONCE_INIT: Once = Once::new();

#[derive(Debug, Deserialize)]
struct Input {
    config: Option<Config>,
    identifier: String,
    password: String,
}

#[wasm_bindgen]
pub async fn shogi_thread(input: String) -> std::result::Result<String, JsValue> {
    ONCE_INIT.call_once(|| {
        console_log::init_with_level(Level::Debug).expect("failed to initialize logger");
    });
    match main(&input).await {
        Ok(s) => Ok(s),
        Err(e) => Err(JsValue::from_str(&format!("failed: {e}"))),
    }
}

async fn main(input: &str) -> Result<String> {
    let input = serde_json::from_str::<Input>(input)?;
    let agent = if let Some(config) = input.config {
        BskyAgent::builder().config(config).build().await?
    } else {
        let agent = BskyAgent::builder().build().await?;
        agent
            .login(input.identifier, input.password)
            .await
            .map_err(|e| Error::Sdk(e.into()))?;
        agent
    };
    let config = agent.to_config().await;
    Bot::new(agent).run().await?;
    Ok(serde_json::to_string(&config)?)
}
