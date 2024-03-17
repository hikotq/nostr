use axum::extract::ws::Message;
use tokio::sync::mpsc::UnboundedSender;

use crate::req::Filter;

pub struct Subscriber {
    pub client: String,
    pub sender: UnboundedSender<Message>,
    pub id: String,
    pub filter: Filter,
}
