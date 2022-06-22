use database::CONN;
use dotenv::dotenv;
use std::{error::Error, path::Path};
use teloxide::{
    net::Download,
    prelude::*,
    types::{MediaKind, MessageKind},
};
use tokio::fs::File;

mod database;
mod server;

const BASE_URL: &'static str = "http://127.0.0.1:3000";

// static CONN: Lazy<DB> = Lazy::new(|| DB::open(DATABASE_URL).unwrap());

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    tokio::spawn(server::run());
    tokio::spawn(database::gc::run());
    // let db = D
    // insert("sssss", "aaaaa").unwrap();
    log::info!("Starting file share bot...");
    let bot = Bot::from_env().auto_send();
    teloxide::repl(bot, handler).await;
}

async fn handler(message: Message, bot: AutoSend<Bot>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let MessageKind::Common(message_common) = message.kind {
        let file_ids = match message_common.media_kind {
            MediaKind::Document(doc) => {
                let file_id = doc.document.file_id;
                CONN.insert(&file_id, doc.document.file_name)?;
                vec![file_id]
            }
            MediaKind::Video(video) => {
                let file_id = video.video.file_id;
                CONN.insert(&file_id, video.video.file_name)?;
                vec![file_id]
            }

            MediaKind::Photo(photo) => photo
                .photo
                .into_iter()
                .map(|photo| {
                    CONN.insert(&photo.file_id, None).unwrap();
                    photo.file_id
                })
                .collect(),
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
    Ok(())
}
