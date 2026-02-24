use std::num::NonZeroUsize;

use clap::Parser;
use glob::glob;
use indicatif::ProgressStyle;
use once_cell::sync::Lazy;
use reqwest::{Client, Url};

pub static ARGS: Lazy<Args> = Lazy::new(Args::parse_with_glob_expand);
pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::builder().http1_only().build().unwrap());

pub static PROGRESS_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template(
        "{spinner} {msg} {wide_bar} {bytes}/{total_bytes} {bytes_per_sec} ({eta})",
    )
    .unwrap()
});

pub static API_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://pixeldrain.com/api/file/").unwrap());

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub api_key: String,

    #[arg(required = true)]
    pub files: Vec<String>,

    #[arg(short, long, default_value_t = NonZeroUsize::new(4).unwrap())]
    pub workers: NonZeroUsize,
}

impl Args {
    pub fn parse_with_glob_expand() -> Self {
        let mut args = Self::parse();
        args.files = expand_globs(&args.files);
        args
    }
}

fn expand_globs(inputs: &[String]) -> Vec<String> {
    let mut v = inputs
        .iter()
        .flat_map(|i| glob(i).unwrap())
        .map(|entry| entry.unwrap())
        .filter(|p| p.is_file())
        .map(|path| path.to_str().unwrap().to_owned())
        .collect::<Vec<_>>();

    v.sort();
    v.dedup();

    v
}
