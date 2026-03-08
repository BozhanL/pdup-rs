use std::{path::Path, sync::Mutex};

use futures_util::stream::{self, StreamExt as _};
use indicatif::{MultiProgress, ProgressBar};
use reqwest::{Body, Version};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

mod configs;
use configs::{API_URL, ARGS, CLIENT, PROGRESS_STYLE};

// set_move_cursor plays badly with println
// https://github.com/console-rs/indicatif/issues/759
static PRINT_LOCK: Mutex<()> = Mutex::new(());

#[tokio::main]
async fn main() {
    let bars = MultiProgress::new();

    // https://github.com/console-rs/indicatif/issues/447
    let dummy_bar = bars.add(ProgressBar::no_length());
    bars.println(format!(
        "Uploading {} files with {} workers",
        ARGS.files.len(),
        ARGS.workers
    ))
    .unwrap();
    bars.println(format!(
        "Files to upload: {}",
        ARGS.files
            .iter()
            .map(Path::new)
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect::<Vec<_>>()
            .join(", ")
    ))
    .unwrap();

    dummy_bar.finish();
    bars.remove(&dummy_bar);
    drop(dummy_bar);

    stream::iter(ARGS.files.iter().map(Path::new))
        .for_each_concurrent(ARGS.workers.get(), |file| {
            let bars = bars.clone();

            let bar = {
                let len = file.metadata().unwrap().len();

                let _lock = PRINT_LOCK.lock().unwrap();
                bars.set_move_cursor(false);

                let bar = bars.add(ProgressBar::new(len));
                bar.set_style(PROGRESS_STYLE.clone());

                bars.set_move_cursor(true);
                drop(_lock);

                bar
            };

            let file_name = file.file_name().unwrap();
            let file_name = file_name.to_str().unwrap();
            bar.set_message(file_name);

            upload_one(file, bar, bars)
        })
        .await;
}

async fn upload_one(file: &Path, bar: ProgressBar, bars: MultiProgress) {
    let file_name = file.file_name().unwrap();
    let file_name = file_name.to_str().unwrap();
    let url = API_URL.join(file_name).unwrap();

    let file = File::open(&file).await.unwrap();
    let reader_stream = ReaderStream::with_capacity(bar.wrap_async_read(file), 64 * 1024);

    let resp = CLIENT
        .put(url)
        .basic_auth("", Some(&ARGS.api_key))
        .body(Body::wrap_stream(reader_stream))
        .version(Version::HTTP_11)
        .send()
        .await
        .unwrap();

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap();
        bars.suspend(|| {
            eprintln!("Failed to upload {}: {}", file_name, text);
            panic!("Failed to upload {}", file_name);
        });
    }

    {
        let _lock = PRINT_LOCK.lock().unwrap();
        bars.set_move_cursor(false);

        bar.finish();
        bars.remove(&bar);
        bars.println(format!("Uploaded {}", file_name)).unwrap();

        bars.set_move_cursor(true);
        drop(_lock);
    };
}
