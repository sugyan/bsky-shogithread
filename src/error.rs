use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Sdk(#[from] bsky_sdk::Error),
    #[error(transparent)]
    ShogiUsiParser(#[from] shogi_usi_parser::Error),
    #[error("not thread view post")]
    NotThreadViewPost,
    #[error("not feed post record")]
    NotFeedPostRecord,
}

pub type Result<T> = std::result::Result<T, Error>;
