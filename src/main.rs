use dotenv::dotenv;
use lazy_static::lazy_static;
use std::{collections::HashMap, error::Error, path::Path, sync::Mutex};
use teloxide::{
    net::Download,
    prelude::*,
    types::{MediaKind, MessageKind},
};
use tokio::fs::File;

mod server;

lazy_static! {
    static ref MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

const BASE_URL: &str = "http://127.0.0.1:3000";

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    tokio::spawn(server::run());
    log::info!("Starting file share bot...");
    let bot = Bot::from_env().auto_send();
    teloxide::repl(bot, handler).await;
}

async fn handler(message: Message, bot: AutoSend<Bot>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let MessageKind::Common(message_common) = message.kind {
        let file_ids = match message_common.media_kind {
            MediaKind::Document(doc) => {
                let file_id = doc.document.file_id;
                // let file_path = bot.get_file(&file_id).send().await?.file_path;
                if let Some(file_name) = &doc.document.file_name {
                    MAP.lock()
                        .unwrap()
                        .insert(file_id.clone(), file_name.clone());
                }
                vec![file_id]
            }
            MediaKind::Video(video) => {
                let file_id = video.video.file_id;
                if let Some(file_name) = &video.video.file_name {
                    MAP.lock()
                        .unwrap()
                        .insert(file_id.clone(), file_name.clone());
                }
                vec![file_id]
            }

            MediaKind::Photo(photo) => photo.photo.into_iter().map(|photo| photo.file_id).collect(),
            _ => return Ok(()),
        };

        for file_id in file_ids {
            let file_path = bot.get_file(&file_id).send().await?.file_path;
            let path = Path::new("./tmp").join(&file_id);
            let mut file = File::create(path).await.unwrap();
            bot.download_file(&file_path, &mut file).await.unwrap();
            bot.send_message(
                message.chat.id,
                format!("Download Url: \n{}/{}", BASE_URL, file_id),
            )
            .await?;
        }
    }

    // respond(())
    Ok(())
}
