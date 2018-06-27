use error::*;
use std::sync::mpsc;
use std::thread;
use std::time;

#[derive(Debug)]
pub enum Message {
    FeatureUpdate(String),
    Kill,
}

pub fn send_message(feature: &str, id: &str, tx: &mpsc::Sender<Message>) {
    let message = Message::FeatureUpdate(String::from(id));

    tx.send(message)
        .wrap_error_kill(feature, "notify thread killed");
}

pub fn send_message_interval(
    feature: &'static str,
    id: String,
    tx: mpsc::Sender<Message>,
    interval: u64,
    delay: Option<u64>,
) {
    thread::spawn(move || {
        if let Some(delay_seconds) = delay {
            thread::sleep(time::Duration::from_secs(delay_seconds));
        }

        loop {
            send_message(feature, &id, &tx);

            thread::sleep(time::Duration::from_secs(interval));
        }
    });
}
