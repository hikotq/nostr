use thiserror::Error;

#[derive(Error, Debug)]
pub enum NostrError {
    #[error("無効な形式のメッセージ: {0}")]
    InvalidMessage(String),
}
