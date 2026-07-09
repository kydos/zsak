use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use iced::Task;

use zsak::types::{QueryableParams, SubscribeParams, ZenohEvent};
use crate::app::Message;

pub fn subscribe_stream(
    session: Arc<zenoh::Session>,
    params: SubscribeParams,
) -> (Task<Message>, oneshot::Sender<()>) {
    let (event_tx, event_rx) = mpsc::channel::<ZenohEvent>(64);
    let (cancel_tx, cancel_rx) = oneshot::channel();
    tokio::spawn(async move {
        zsak::action::do_subscribe_streaming(&session, params, event_tx, cancel_rx).await;
    });
    let stream = ReceiverStream::new(event_rx).map(Message::StreamEvent);
    (Task::stream(stream), cancel_tx)
}

pub fn queryable_stream(
    session: Arc<zenoh::Session>,
    params: QueryableParams,
) -> (Task<Message>, oneshot::Sender<()>) {
    let (event_tx, event_rx) = mpsc::channel::<ZenohEvent>(64);
    let (cancel_tx, cancel_rx) = oneshot::channel();
    tokio::spawn(async move {
        zsak::action::do_queryable_streaming(&session, params, event_tx, cancel_rx).await;
    });
    let stream = ReceiverStream::new(event_rx).map(Message::StreamEvent);
    (Task::stream(stream), cancel_tx)
}

pub fn liveliness_subscribe_stream(
    session: Arc<zenoh::Session>,
    key_expr: String,
) -> (Task<Message>, oneshot::Sender<()>) {
    let (event_tx, event_rx) = mpsc::channel::<ZenohEvent>(64);
    let (cancel_tx, cancel_rx) = oneshot::channel();
    tokio::spawn(async move {
        zsak::action::do_subscribe_liveliness_streaming(&session, key_expr, event_tx, cancel_rx).await;
    });
    let stream = ReceiverStream::new(event_rx).map(Message::StreamEvent);
    (Task::stream(stream), cancel_tx)
}
