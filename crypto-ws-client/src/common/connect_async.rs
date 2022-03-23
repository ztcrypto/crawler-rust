use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::tungstenite::{Error, Message};

use governor::{Quota, RateLimiter};
use nonzero_ext::*;
use std::num::NonZeroU32;

/// Wraps a websocket client inside an event loop, returns a message_rx to receive messages and
/// a command_tx to send commands to the websocket server.
///
/// To close the websocket connection, send a `Message::Close` message to the command_tx.
///
/// `limit`, max number of uplink messsages, for example, 100 per 10 seconds
pub async fn connect_async(
    url: &str,
    uplink_limit: Option<(NonZeroU32, std::time::Duration)>,
) -> Result<(Receiver<Message>, Sender<Message>), Error> {
    let (command_tx, mut command_rx) = tokio::sync::mpsc::channel::<Message>(1);
    let (message_tx, message_rx) = tokio::sync::mpsc::channel::<Message>(32);

    let ret = tokio_tungstenite::connect_async(url).await;
    if let Err(e) = ret {
        return Err(e);
    }
    let (ws_stream, _) = ret.unwrap();
    let (mut write, mut read) = ws_stream.split();

    let limiter = if let Some((max_burst, duration)) = uplink_limit {
        let quota = Quota::with_period(duration).unwrap().allow_burst(max_burst);
        RateLimiter::direct(quota)
    } else {
        RateLimiter::direct(Quota::per_second(nonzero!(u32::max_value())))
    };

    tokio::task::spawn(async move {
        loop {
            tokio::select! {
              command = command_rx.recv() => {
                match command {
                  Some(command) => {
                    match command {
                      Message::Close(_) => {
                        break; // close the connection and break the loop
                      }
                      _ => {
                        limiter.until_ready().await;
                        if let Err(err) =write.send(command).await {
                          error!("{}", err);
                        }
                      }
                    }
                  }
                  None => {
                    debug!("command_rx closed");
                    break;
                  }
                }
              }
              msg = read.next() => match msg {
                Some(Ok(msg)) => {
                  let _= message_tx.send(msg).await;
                }
                Some(Err(err)) => {
                  error!("{}", err);
                  break;
                }
                None => {
                  debug!("message_tx closed");
                  break;
                }
              }
            };
        }
        _ = write.send(Message::Close(None)).await;
    });

    Ok((message_rx, command_tx))
}