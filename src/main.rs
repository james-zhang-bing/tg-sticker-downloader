use std::{env, process::Command};

use ffmpeg_next::format::context::Input;
use teloxide::{net::Download, prelude::*, types::InputFile};
use tokio::fs;
#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            if text.starts_with("https://t.me/addstickers/") {
                let pack_name = &text["https://t.me/addstickers/".len()..];

                let sticker_set = bot.get_sticker_set(pack_name).send().await.unwrap();
                let set_name = sticker_set.name.clone();
                fs::create_dir(&set_name).await?;
                for (key, sticker) in sticker_set.stickers.iter().enumerate() {
                    let file_id = sticker.file.id.clone();
                    let file = bot.get_file(file_id.clone()).await?;
                    let file_name = if sticker.is_animated() || sticker.is_video() {
                        format!("{}_{}.mp4", &set_name, key)
                    } else {
                        format!("{}_{}.png", &set_name, key)
                    };
                    let path = format!("{}/{}", &set_name, file_name);
                    println!("downloading {}", &file_name);
                    let mut dst = fs::File::create(&path).await?;
                    bot.download_file(&file.path, &mut dst).await?;
                    if sticker.is_animated() || sticker.is_video() {
                        //转换成gif
                        // ffmpeg -i animation.gif.mp4 -vf "scale=320:-1,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" output.gif
                        let gif_name = &format!("{}.gif", &path);
                        let ffmpeg_args = vec![
                            "-i",
                            &path,
                            "-vf",
                            r"scale=320:-1,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                            gif_name,
                        ];
                        let mut cmd = Command::new("ffmpeg");
                        cmd.args(ffmpeg_args);
                        let output = cmd.output().expect("Failed to execute command");
                        println!("Output: {}", String::from_utf8_lossy(&output.stdout));
                        fs::remove_file(&path).await.unwrap_or_else(|why| {
                            println!("! {:?}", why.kind());
                        });
                    }
                }
                println!("download done");
            }
        }
        ResponseResult::<()>::Ok(())
    })
    .await;
}
