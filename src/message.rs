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

#[derive(Debug, PartialEq, Eq)]
pub enum ServerMessage {
    Event(ServerMessageEvent),
    Ok(ServerOk),
    EOSE(String),
    Closed(Closed),
    Notice(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ServerMessageEvent {
    pub subscribe_id: String,
    pub event: Event,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ServerOk {
    pub event_id: String,
    pub accepted: bool,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Closed {
    pub subscribe_id: String,
    pub message: String,
}

impl Serialize for ServerMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ServerMessage::Event(event) => serialize_server_event(event, serializer),
            ServerMessage::Ok(ok) => serialize_ok(ok, serializer),
            ServerMessage::EOSE(id) => serialize_eose(id, serializer),
            ServerMessage::Closed(closed) => serialize_closed(closed, serializer),
            ServerMessage::Notice(message) => serialize_notice(message, serializer),
        }
    }
}

fn serialize_server_event<S>(event: &ServerMessageEvent, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(3))?;
    seq.serialize_element("EVENT")?;
    seq.serialize_element(&event.subscribe_id)?;
    seq.serialize_element(&event.event)?;
    seq.end()
}

fn serialize_ok<S>(ok: &ServerOk, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(4))?;
    seq.serialize_element("OK")?;
    seq.serialize_element(&ok.event_id)?;
    seq.serialize_element(&ok.accepted)?;
    seq.serialize_element(&ok.message)?;
    seq.end()
}

fn serialize_eose<S>(id: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element("EOSE")?;
    seq.serialize_element(id)?;
    seq.end()
}

fn serialize_closed<S>(closed: &Closed, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(3))?;
    seq.serialize_element("CLOSED")?;
    seq.serialize_element(&closed.subscribe_id)?;
    seq.serialize_element(&closed.message)?;
    seq.end()
}

fn serialize_notice<S>(message: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element("NOTICE")?;
    seq.serialize_element(message)?;
    seq.end()
}

impl<'de> Deserialize<'de> for ServerMessage {
    fn deserialize<D>(deserializer: D) -> Result<ServerMessage, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ServerMessageVisitor)
    }
}

struct ServerMessageVisitor;

impl<'de> Visitor<'de> for ServerMessageVisitor {
    type Value = ServerMessage;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of three elements")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<ServerMessage, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let kind = seq
            .next_element::<&str>()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        match kind {
            "EVENT" => deserialize_server_event(&self, &mut seq),
            "OK" => deserialize_ok(&self, &mut seq),
            "EOSE" => deserialize_eose(&self, &mut seq),
            "CLOSED" => deserialize_closed(&self, &mut seq),
            "NOTICE" => deserialize_notice(&self, &mut seq),
            _ => Err(de::Error::custom("unknown message kind")),
        }
    }
}

fn deserialize_server_event<'de, 'a, V>(
    visitor: &'a ServerMessageVisitor,
    seq: &mut V,
) -> Result<ServerMessage, <V as SeqAccess<'de>>::Error>
where
    V: SeqAccess<'de>,
{
    let subscribe_id = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(1, visitor))?;
    let event = seq
        .next_element::<Event>()?
        .ok_or_else(|| de::Error::invalid_length(2, visitor))?;
    Ok(ServerMessage::Event(ServerMessageEvent {
        subscribe_id,
        event,
    }))
}

fn deserialize_ok<'de, 'a, V>(
    visitor: &'a ServerMessageVisitor,
    seq: &mut V,
) -> Result<ServerMessage, <V as SeqAccess<'de>>::Error>
where
    V: SeqAccess<'de>,
{
    let event_id = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(1, visitor))?;
    let accepted = seq
        .next_element::<bool>()?
        .ok_or_else(|| de::Error::invalid_length(2, visitor))?;
    let message = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(3, visitor))?;
    Ok(ServerMessage::Ok(ServerOk {
        event_id,
        accepted,
        message,
    }))
}

fn deserialize_eose<'de, 'a, V>(
    visitor: &'a ServerMessageVisitor,
    seq: &mut V,
) -> Result<ServerMessage, <V as SeqAccess<'de>>::Error>
where
    V: SeqAccess<'de>,
{
    let id = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(1, visitor))?;
    Ok(ServerMessage::EOSE(id))
}

fn deserialize_closed<'de, 'a, V>(
    visitor: &'a ServerMessageVisitor,
    seq: &mut V,
) -> Result<ServerMessage, <V as SeqAccess<'de>>::Error>
where
    V: SeqAccess<'de>,
{
    let subscribe_id = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(1, visitor))?;
    let message = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(2, visitor))?;
    Ok(ServerMessage::Closed(Closed {
        subscribe_id,
        message,
    }))
}

fn deserialize_notice<'de, 'a, V>(
    visitor: &'a ServerMessageVisitor,
    seq: &mut V,
) -> Result<ServerMessage, <V as SeqAccess<'de>>::Error>
where
    V: SeqAccess<'de>,
{
    let message = seq
        .next_element::<String>()?
        .ok_or_else(|| de::Error::invalid_length(1, visitor))?;
    Ok(ServerMessage::Notice(message))
}

#[cfg(test)]
mod tests {

    use crate::message::Event;
    use bech32::decode;

    use crate::event::{EventKind, UnsignedEvent};

    use super::{ClientMessage, ServerMessage, ServerMessageEvent};

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

    fn data_provider_event<'a>() -> (Event, String) {
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
            r##"{{"id":"8b0a64c96cd09a3a86c0a225606f0b57a7fec7bf3773c68af13420c1d8d57f97","pubkey":"{pubkey}","created_at":{created_at},"kind":1,"tags":[["tag"]],"content":"content","sig":"80a143f5802118f295b9281b7192feb522ac9eb8cd6922694879cc36ca6f2d35077170f2953c5174b09049d6fa3463b6ed87cbe9e4ac627271ec0b5b73e0ee44"}}"##,
        );
        (event, serialized)
    }

    #[test]
    fn serialize_event() {
        let (event, raw_event) = data_provider_event();
        let req = ClientMessage::Event(event);
        let expected = format!(r##"["EVENT",{}]"##, raw_event,);
        assert_eq!(serde_json::to_string(&req).unwrap(), expected,);
    }

    #[test]
    fn deserialize_event() {
        let (expected, raw_event) = data_provider_event();
        let serialized = format!(r##"["EVENT",{}]"##, raw_event,);
        let message: ClientMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, expected.into());
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

    #[test]
    fn serialize_server_event() {
        let (event, serialized) = data_provider_event();
        let event = ServerMessageEvent {
            subscribe_id: "id".to_string(),
            event,
        };
        let expected = format!(r##"["EVENT","id",{}]"##, serialized,);
        let message = ServerMessage::Event(event);
        assert_eq!(serde_json::to_string(&message).unwrap(), expected,);
    }

    #[test]
    fn deserialize_server_event() {
        let (event, serialized) = data_provider_event();
        let expected = ServerMessage::Event(ServerMessageEvent {
            subscribe_id: "id".to_string(),
            event,
        });
        let serialized = format!(r##"["EVENT","id",{}]"##, serialized,);
        let message: ServerMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, expected);
    }

    #[test]
    fn serizlie_ok() {
        let ok = super::ServerOk {
            event_id: "id".to_string(),
            accepted: true,
            message: "message".to_string(),
        };
        let expected = r##"["OK","id",true,"message"]"##;
        let message = ServerMessage::Ok(ok);
        assert_eq!(serde_json::to_string(&message).unwrap(), expected,);
    }

    #[test]
    fn deserialize_ok() {
        let ok = super::ServerOk {
            event_id: "id".to_string(),
            accepted: true,
            message: "message".to_string(),
        };
        let expected = ServerMessage::Ok(ok);
        let serialized = r##"["OK","id",true,"message"]"##;
        let message: ServerMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, expected);
    }

    #[test]
    fn serialize_eose() {
        let id = "id";
        let expected = r##"["EOSE","id"]"##;
        let message = ServerMessage::EOSE(id.to_string());
        assert_eq!(serde_json::to_string(&message).unwrap(), expected,);
    }

    #[test]
    fn deserialize_eose() {
        let id = "id";
        let expected = ServerMessage::EOSE(id.to_string());
        let serialized = r##"["EOSE","id"]"##;
        let message: ServerMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, expected);
    }

    #[test]
    fn serialize_closed() {
        let closed = super::Closed {
            subscribe_id: "id".to_string(),
            message: "message".to_string(),
        };
        let expected = r##"["CLOSED","id","message"]"##;
        let message = ServerMessage::Closed(closed);
        assert_eq!(serde_json::to_string(&message).unwrap(), expected,);
    }

    #[test]
    fn deserialize_closed() {
        let closed = super::Closed {
            subscribe_id: "id".to_string(),
            message: "message".to_string(),
        };
        let expected = ServerMessage::Closed(closed);
        let serialized = r##"["CLOSED","id","message"]"##;
        let message: ServerMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, expected);
    }

    #[test]
    fn serialize_notice() {
        let message = "message";
        let expected = r##"["NOTICE","message"]"##;
        let message = ServerMessage::Notice(message.to_string());
        assert_eq!(serde_json::to_string(&message).unwrap(), expected,);
    }

    #[test]
    fn deserialize_notice() {
        let message = "message";
        let expected = ServerMessage::Notice(message.to_string());
        let serialized = r##"["NOTICE","message"]"##;
        let message: ServerMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, expected);
    }
}
