#![allow(async_fn_in_trait)]
#![allow(clippy::needless_return)]

pub mod prelude;

pub mod emote;
pub mod stream;
pub mod stream_message;
pub mod twitch_user;
pub mod twitch_user_unknown_user_association;
pub mod unknown_user;

pub use anyhow::Error as ExtensionError;
