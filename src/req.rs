use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq)]
pub struct Req {
    pub id: String,
    pub filter: Filter,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Filter {
    // イベントのID、もしくは先頭部分（プレフィクス）のリスト
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    // 公開鍵、もしくは先頭部分のリスト
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    // イベントの種類の数字のリスト
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<u16>>,
    // "e"タグで参照されたイベントIDのリスト
    #[serde(rename = "#e", skip_serializing_if = "Option::is_none")]
    pub e_tags: Option<Vec<String>>,
    // "p"タグで参照された公開鍵のリスト
    #[serde(rename = "#p", skip_serializing_if = "Option::is_none")]
    pub p_tags: Option<Vec<String>>,
    // UNIXタイムスタンプ（秒単位の整数値）。パスするには、イベントはこれより新しくなければならない
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    // UNIXタイムスタンプ（秒単位の整数値）。パスするには、イベントはこれより古くなければならない
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<i64>,
    // 初回の問い合わせで返されるイベントの個数の上限
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
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

    pub fn kinds(mut self, kinds: Vec<u16>) -> Self {
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
