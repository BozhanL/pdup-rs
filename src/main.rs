use std::path::Path;

use futures_util::{stream, stream::StreamExt as _};
use indicatif::{MultiProgress, ProgressBar};
use reqwest::{Body, Version};
use tokio::{fs::File, io::BufReader};
use tokio_util::io::ReaderStream;

mod configs;
use configs::{API_URL, ARGS, CLIENT, PROGRESS_STYLE};

#[tokio::main]
async fn main() {
    let bars = MultiProgress::new();
    bars.set_move_cursor(true);

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

    stream::iter(ARGS.files.iter().map(Path::new))
        .for_each_concurrent(ARGS.workers.get(), |file| {
            let bars = bars.clone();
            let bar = bars.add(ProgressBar::no_length());
            bar.set_style(PROGRESS_STYLE.clone());

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
    let file_size = file.metadata().await.unwrap().len();
    bar.set_length(file_size);
    let reader_stream =
        ReaderStream::with_capacity(bar.wrap_async_read(BufReader::new(file)), 65536);

    let resp = CLIENT
        .put(url)
        .basic_auth("", Some(&ARGS.api_key))
        .body(Body::wrap_stream(reader_stream))
        .version(Version::HTTP_11)
        .send()
        .await
        .unwrap();

    if !resp.status().is_success() {
        bars.println(format!(
            "Failed to upload {}: {}",
            file_name,
            resp.text().await.unwrap()
        ))
        .unwrap();
        panic!("Failed to upload {}", file_name);
    }

    bar.finish_and_clear();
    bars.remove(&bar);
    bars.println(format!("Uploaded {}", file_name)).unwrap();
}
