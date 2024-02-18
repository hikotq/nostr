use std::fmt;

use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Serialize, Serializer,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Req {
    pub id: String,
    pub filter: Filter,
}

impl Serialize for Req {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element("REQ")?;
        seq.serialize_element(self.id.as_str())?;
        seq.serialize_element(&self.filter)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Req {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ReqVisitor;

        impl<'de> Visitor<'de> for ReqVisitor {
            type Value = Req;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of three elements")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Req, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let _ = seq
                    .next_element::<&str>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let id = seq
                    .next_element::<String>()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let filter = seq
                    .next_element::<Filter>()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                Ok(Req { id, filter })
            }
        }
        // deserializer に ReqVisitor を使用して、Req 構造体をデシリアライズします。
        deserializer.deserialize_seq(ReqVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Filter {
    // イベントのID、もしくは先頭部分（プレフィクス）のリスト
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<Vec<String>>,
    // 公開鍵、もしくは先頭部分のリスト
    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<String>>,
    // イベントの種類の数字のリスト
    #[serde(skip_serializing_if = "Option::is_none")]
    kinds: Option<Vec<u32>>,
    // "e"タグで参照されたイベントIDのリスト
    #[serde(rename = "#e", skip_serializing_if = "Option::is_none")]
    e_tags: Option<Vec<String>>,
    // "p"タグで参照された公開鍵のリスト
    #[serde(rename = "#p", skip_serializing_if = "Option::is_none")]
    p_tags: Option<Vec<String>>,
    // UNIXタイムスタンプ（秒単位の整数値）。パスするには、イベントはこれより新しくなければならない
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<i64>,
    // UNIXタイムスタンプ（秒単位の整数値）。パスするには、イベントはこれより古くなければならない
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<i64>,
    // 初回の問い合わせで返されるイベントの個数の上限
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
}

impl Filter {
    // 新しいクエリインスタンスを生成するためのコンストラクタ
    pub fn new() -> Self {
        Self {
            ids: None,
            authors: None,
            kinds: None,
            e_tags: None,
            p_tags: None,
            since: None,
            until: None,
            limit: None,
        }
    }

    // 各フィールドを設定するメソッド。selfの所有権を取り、更新された新しいインスタンスを返す。
    pub fn ids(mut self, ids: Vec<String>) -> Self {
        self.ids = Some(ids);
        self
    }

    pub fn authors(mut self, authors: Vec<String>) -> Self {
        self.authors = Some(authors);
        self
    }

    pub fn kinds(mut self, kinds: Vec<u32>) -> Self {
        self.kinds = Some(kinds);
        self
    }

    pub fn e_tags(mut self, e_tags: Vec<String>) -> Self {
        self.e_tags = Some(e_tags);
        self
    }

    pub fn p_tags(mut self, p_tags: Vec<String>) -> Self {
        self.p_tags = Some(p_tags);
        self
    }

    pub fn since(mut self, since: i64) -> Self {
        self.since = Some(since);
        self
    }

    pub fn until(mut self, until: i64) -> Self {
        self.until = Some(until);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Req;

    fn data_provider<'a>() -> (Req, &'a str) {
        let req = super::Req {
            id: "id".to_string(),
            filter: super::Filter::new()
                .ids(vec!["id".to_string()])
                .authors(vec!["pubkey".to_string()])
                .kinds(vec![1])
                .e_tags(vec!["e_tag".to_string()])
                .p_tags(vec!["p_tag".to_string()])
                .since(1708203194)
                .until(1708203194)
                .limit(10),
        };
        let serialized = r##"["REQ","id",{"ids":["id"],"authors":["pubkey"],"kinds":[1],"#e":["e_tag"],"#p":["p_tag"],"since":1708203194,"until":1708203194,"limit":10}]"##;
        (req, serialized)
    }

    #[test]
    fn serialize() {
        let (req, expected) = data_provider();
        assert_eq!(serde_json::to_string(&req).unwrap(), expected,);
    }

    #[test]
    fn deserialize() {
        let (expected, serialized) = data_provider();
        let req: Req = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, expected);
    }
}
