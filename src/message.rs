use std::fmt;

use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Serialize,
};

use crate::{
    event::Event,
    req::{Filter, Req},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ClientMessage {
    Req(Req),
    Event(Event),
    Close(String),
}

impl Serialize for ClientMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ClientMessage::Req(req) => serialize_req(req, serializer),
            ClientMessage::Event(event) => serialize_event(event, serializer),
            ClientMessage::Close(id) => serialize_close(id, serializer),
        }
    }
}

fn serialize_req<S>(req: &Req, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(3))?;
    seq.serialize_element("REQ")?;
    seq.serialize_element(req.id.as_str())?;
    seq.serialize_element(&req.filter)?;
    seq.end()
}

fn serialize_event<S>(event: &Event, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element("EVENT")?;
    seq.serialize_element(event)?;
    seq.end()
}

fn serialize_close<S>(id: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element("CLOSE")?;
    seq.serialize_element(id)?;
    seq.end()
}

impl<'de> Deserialize<'de> for ClientMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ClientMessageVisitor)
    }
}

struct ClientMessageVisitor;

impl<'de> Visitor<'de> for ClientMessageVisitor {
    type Value = ClientMessage;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of three elements")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<ClientMessage, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let kind = seq
            .next_element::<&str>()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        match kind {
            "REQ" => deserialize_req(&self, &mut seq),
            "EVENT" => deserialize_event(&self, &mut seq),
            "CLOSE" => deserialize_close(&self, &mut seq),
            _ => Err(de::Error::custom("unknown message kind")),
        }
    }
}

fn deserialize_req<'de, 'a, V>(
    visitor: &'a ClientMessageVisitor,
    seq: &mut V,
) -> Result<ClientMessage, <V as SeqAccess<'de>>::Error>
where
    V: SeqAccess<'de>,
{
    let id = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(1, visitor))?;
    let filter = seq
        .next_element::<Filter>()?
        .ok_or_else(|| de::Error::invalid_length(2, visitor))?;
    Ok(ClientMessage::Req(Req { id, filter }))
}

fn deserialize_event<'de, 'a, V>(
    visitor: &'a ClientMessageVisitor,
    seq: &mut V,
) -> Result<ClientMessage, <V as SeqAccess<'de>>::Error>
where
    V: SeqAccess<'de>,
{
    let event = seq
        .next_element::<Event>()?
        .ok_or_else(|| de::Error::invalid_length(1, visitor))?;
    Ok(ClientMessage::Event(event))
}

fn deserialize_close<'de, 'a, V>(
    visitor: &'a ClientMessageVisitor,
    seq: &mut V,
) -> Result<ClientMessage, <V as SeqAccess<'de>>::Error>
where
    V: SeqAccess<'de>,
{
    let id = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(1, visitor))?;
    Ok(ClientMessage::Close(id))
}

impl From<Req> for ClientMessage {
    fn from(req: Req) -> Self {
        ClientMessage::Req(req)
    }
}

impl From<Event> for ClientMessage {
    fn from(event: Event) -> Self {
        ClientMessage::Event(event)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use bech32::decode;

    use crate::event::{EventKind, UnsignedEvent};

    use super::ClientMessage;

    const TEST_PUBKEY: &str = "npub1test2s5u9l0z8dakmap5s6ddw8fvjsp6820h52nzjc35j8j8wv6qcnjx5q";
    const TEST_SECKEY: &str = "nsec1kj0mc49wzr2lqjka0m06ft0ku8n4zntgk6yh78vuvqdw7mnctk6q3uh0fr";

    fn data_provider_req<'a>() -> (ClientMessage, &'a str) {
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
        (req.into(), serialized)
    }

    #[test]
    fn serialize_req() {
        let (req, expected) = data_provider_req();
        assert_eq!(serde_json::to_string(&req).unwrap(), expected,);
    }

    #[test]
    fn deserialize_req() {
        let (expected, serialized) = data_provider_req();
        let message: ClientMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, expected);
    }

    fn data_provider_event<'a>() -> (ClientMessage, String) {
        let created_at = 1708838939;
        let (_, pubkey) = decode(TEST_PUBKEY).unwrap();
        let pubkey = hex::encode(pubkey);
        let (_, seckey) = decode(TEST_SECKEY).unwrap();
        let seckey = hex::encode(seckey);
        let event = UnsignedEvent::new(
            pubkey.clone(),
            EventKind::TextNote,
            vec![vec!["tag".to_string()]],
            "content".to_string(),
            created_at,
        )
        .sign(&seckey);
        let serialized = format!(
            r##"["EVENT",{{"id":"8b0a64c96cd09a3a86c0a225606f0b57a7fec7bf3773c68af13420c1d8d57f97","pubkey":"{pubkey}","created_at":{created_at},"kind":1,"tags":[["tag"]],"content":"content","sig":"80a143f5802118f295b9281b7192feb522ac9eb8cd6922694879cc36ca6f2d35077170f2953c5174b09049d6fa3463b6ed87cbe9e4ac627271ec0b5b73e0ee44"}}]"##,
        );
        (event.into(), serialized)
    }

    #[test]
    fn serialize_event() {
        let (req, expected) = data_provider_event();
        assert_eq!(serde_json::to_string(&req).unwrap(), expected,);
    }

    #[test]
    fn deserialize_event() {
        let (expected, serialized) = data_provider_event();
        let message: ClientMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, expected);
    }

    #[test]
    fn serialize_close() {
        let id = "id";
        let expected = r##"["CLOSE","id"]"##;
        let message = ClientMessage::Close(id.to_string());
        assert_eq!(serde_json::to_string(&message).unwrap(), expected,);
    }

    #[test]
    fn deserialize_close() {
        let id = "id";
        let serialized = r##"["CLOSE","id"]"##;
        let message: ClientMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, ClientMessage::Close(id.to_string()));
    }
}
