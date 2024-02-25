use hex;
use libsecp256k1::{sign, Message, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct UnsignedEvent {
    // SHA-256 (32バイト) を小文字の16進数で表記
    id: String,
    // 公開鍵 (32バイト) を小文字の16進数で表記
    pubkey: String,
    // UNIXタイムスタンプ（秒単位）
    created_at: u64,
    // イベントの種類
    kind: EventKind,
    // タグ
    tags: Vec<Vec<String>>,
    // 任意の文字列
    content: String,
}

impl UnsignedEvent {
    pub fn new(
        pubkey: String,
        kind: EventKind,
        tags: Vec<Vec<String>>,
        content: String,
        created_at: u64,
    ) -> Self {
        // シリアライズしたイベントからハッシュ値(id)を計算
        let serialized_event = format!(
            r#"[0,"{}",{},{},{},"{}"]"#,
            pubkey,
            created_at,
            u64::from(kind),
            serde_json::to_string(&tags).unwrap(),
            content
        );

        let mut hasher = Sha256::new();
        hasher.update(serialized_event);
        let hash = hasher.finalize();
        let id = hex::encode(&hash);

        Self {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content,
        }
    }

    pub fn sign(self, seckey: &str) -> Event {
        // 計算したidと秘密鍵を使って署名を作成
        let key = SecretKey::parse_slice(&hex::decode(seckey).unwrap()).unwrap();
        let (signature, _) = sign(
            &Message::parse_slice(&hex::decode(&self.id).unwrap()).unwrap(),
            &key,
        );
        let sig = hex::encode(signature.serialize());
        Event {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Event {
    // SHA-256 (32バイト) を小文字の16進数で表記
    id: String,
    // 公開鍵 (32バイト) を小文字の16進数で表記
    pubkey: String,
    // UNIXタイムスタンプ（秒単位）
    created_at: u64,
    // イベントの種類
    kind: EventKind,
    // タグ
    tags: Vec<Vec<String>>,
    // 任意の文字列
    content: String,
    // 署名 (64バイトの16進数)
    sig: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EventKind {
    MetaData,
    TextNote,
}

impl From<EventKind> for u64 {
    fn from(kind: EventKind) -> u64 {
        match kind {
            EventKind::MetaData => 0,
            EventKind::TextNote => 1,
        }
    }
}

impl From<u64> for EventKind {
    fn from(kind: u64) -> EventKind {
        match kind {
            0 => EventKind::MetaData,
            1 => EventKind::TextNote,
            _ => panic!("unknown event kind"),
        }
    }
}

impl Serialize for EventKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64((*self).into())
    }
}

impl<'de> Deserialize<'de> for EventKind {
    fn deserialize<D>(deserializer: D) -> Result<EventKind, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kind = u64::deserialize(deserializer)?;
        Ok(kind.into())
    }
}
