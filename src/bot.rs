use crate::shogi::{last_move_ki2, pos2img, MoveChecker};
use crate::{Error, Result};
use bsky_sdk::api::app::bsky;
use bsky_sdk::api::app::bsky::feed::defs::{PostView, ThreadViewPost, ThreadViewPostRepliesItem};
use bsky_sdk::api::app::bsky::feed::get_post_thread::OutputThreadRefs;
use bsky_sdk::api::app::bsky::feed::post::{RecordEmbedRefs, ReplyRef, ReplyRefData};
use bsky_sdk::api::com::atproto::repo::create_record::Output;
use bsky_sdk::api::com::atproto::repo::strong_ref::{Main, MainData};
use bsky_sdk::api::records::{KnownRecord, Record};
use bsky_sdk::api::types::string::Datetime;
use bsky_sdk::api::types::Union;
use bsky_sdk::BskyAgent;
use shogi_core::{Position, ToUsi};
use shogi_usi_parser::FromUsi;

pub struct Bot {
    agent: BskyAgent,
}

impl Bot {
    pub fn new(agent: BskyAgent) -> Self {
        Self { agent }
    }
    pub async fn run(&self) -> Result<()> {
        let latest = self.get_latest().await?;
        let Record::Known(KnownRecord::AppBskyFeedPost(record)) = &latest.post.record else {
            return Err(Error::NotFeedPostRecord);
        };
        // retrieve latest position
        let pos = if let Some(Union::Refs(RecordEmbedRefs::AppBskyEmbedImagesMain(images))) =
            &record.embed
        {
            Position::from_usi(&images.images[0].alt)?
        } else {
            Default::default()
        };
        log::debug!("{}", pos.to_sfen_owned());
        let mut checker = MoveChecker::new(pos)?;
        // find valid reply
        for reply in latest.data.replies.unwrap_or_default().iter().rev() {
            if let Union::Refs(ThreadViewPostRepliesItem::ThreadViewPost(post)) = reply {
                if let Record::Known(KnownRecord::AppBskyFeedPost(record)) = &post.post.record {
                    log::debug!("{}", record.text);
                    match checker.try_move(&record.text) {
                        Ok(pos) => {
                            let output = self.reply_position(&post.post, pos).await?;
                            log::info!("{:?}", output.data);
                            break;
                        }
                        Err(e) => {
                            log::warn!("failed to move: {e:?}");
                        }
                    }
                }
            }
        }
        Ok(())
    }
    async fn get_latest(&self) -> Result<ThreadViewPost> {
        let session = self
            .agent
            .get_session()
            .await
            .ok_or(Error::Sdk(bsky_sdk::Error::NotLoggedIn))?;
        let feed = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_author_feed(
                bsky::feed::get_author_feed::ParametersData {
                    actor: session.data.did.into(),
                    cursor: None,
                    filter: None,
                    limit: 1.try_into().ok(),
                }
                .into(),
            )
            .await
            .map_err(bsky_sdk::Error::from)?
            .data
            .feed;
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_post_thread(
                bsky::feed::get_post_thread::ParametersData {
                    depth: 1.try_into().ok(),
                    parent_height: 500.try_into().ok(),
                    uri: feed[0].post.uri.clone(),
                }
                .into(),
            )
            .await
            .map_err(bsky_sdk::Error::from)?;
        match output.data.thread {
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsThreadViewPost(post)) => Ok(*post),
            _ => Err(Error::NotThreadViewPost),
        }
    }
    async fn reply_position(&self, post: &PostView, pos: &Position) -> Result<Output> {
        let embed = self.embed(pos).await?;
        let reply = Some(Self::reply_ref(post));
        let mut text = format!("{}手目: ", pos.ply() - 1);
        if let Some(mv) = pos.last_move() {
            if let Ok(Some(ki2)) = last_move_ki2(pos) {
                text.push_str(&format!("{ki2} ({})", mv.to_usi_owned()));
            } else {
                text.push_str(&mv.to_usi_owned());
            }
        }
        Ok(self
            .agent
            .create_record(bsky_sdk::api::app::bsky::feed::post::RecordData {
                created_at: Datetime::now(),
                embed,
                entities: None,
                facets: None,
                labels: None,
                langs: None,
                reply,
                tags: None,
                text,
            })
            .await?)
    }
    async fn post_init(&self) -> Result<Output> {
        let pos = Position::default();
        let embed = self.embed(&pos).await?;
        Ok(self
            .agent
            .create_record(bsky_sdk::api::app::bsky::feed::post::RecordData {
                created_at: Datetime::now(),
                embed,
                entities: None,
                facets: None,
                labels: None,
                langs: None,
                reply: None,
                tags: None,
                text: String::from("start"),
            })
            .await?)
    }
    fn reply_ref(post: &PostView) -> ReplyRef {
        let parent = Main::from(MainData {
            cid: post.cid.clone(),
            uri: post.uri.clone(),
        });
        if let Record::Known(KnownRecord::AppBskyFeedPost(record)) = &post.record {
            if let Some(reply) = &record.reply {
                return ReplyRefData {
                    parent,
                    root: reply.root.clone(),
                }
                .into();
            }
        };
        let root = parent.clone();
        ReplyRefData { parent, root }.into()
    }
    async fn embed(&self, pos: &Position) -> Result<Option<Union<RecordEmbedRefs>>> {
        let (input, (width, height)) = pos2img(pos);
        let image = self
            .agent
            .api
            .com
            .atproto
            .repo
            .upload_blob(input)
            .await
            .map_err(bsky_sdk::Error::from)?
            .data
            .blob;
        let alt = format!(
            "startpos moves {}",
            pos.moves()
                .iter()
                .map(|mv| mv.to_usi_owned())
                .collect::<Vec<_>>()
                .join(" ")
        );
        Ok(Some(Union::Refs(
            bsky::feed::post::RecordEmbedRefs::AppBskyEmbedImagesMain(Box::new(
                bsky::embed::images::MainData {
                    images: vec![bsky::embed::images::ImageData {
                        alt,
                        aspect_ratio: Some(
                            bsky::embed::images::AspectRatioData {
                                height: u64::from(height)
                                    .try_into()
                                    .expect("failed to convert height"),
                                width: u64::from(width)
                                    .try_into()
                                    .expect("failed to convert width"),
                            }
                            .into(),
                        ),
                        image,
                    }
                    .into()],
                }
                .into(),
            )),
        )))
    }
}
