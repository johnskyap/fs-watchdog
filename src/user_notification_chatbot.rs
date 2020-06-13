use anyhow::{Context, Result};
use bbb_api::BbbApi;
use std::sync::mpsc::{channel, Receiver, Sender};
use tokio::runtime::Runtime;

fn chatbot(recv: Receiver<()>) -> Result<()> {
    let api = bbb_api::BbbApi::from_bbbconf()?;
    let message: String = std::env::var("FREESWITCH_WATCHDOG_CHAT_MESSAGE").unwrap_or("A technical problem has occured. Please wait two minutes and then rejoin audio (telephone icon). We are sorry for the inconvenience.".to_owned());
    let mut runtime = Runtime::new()?;
    loop {
        recv.recv()?;

        runtime.block_on(async {
            let res = api.broadcast_message(&message).await;
            if res.is_err() {
                eprintln!("error sending broadcast message:\n{:?}", res.unwrap_err());
            } else {
                let res = res.unwrap();
                for single_res in res {
                    if single_res.is_err() {
                        eprintln!(
                            "error sending broadcast message to meeting:\n{:?}",
                            single_res.unwrap_err()
                        );
                    }
                }
            }
        });
    }
}

pub fn spawn_chatbot() -> Sender<()> {
    let (sender, receiver) = channel();
    std::thread::spawn(move || {
        chatbot(receiver).unwrap();
    });
    sender
}
