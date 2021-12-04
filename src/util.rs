use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use futures_retry::{FutureRetry, RetryPolicy};
use indicatif::ProgressBar;
use reqwest::Client;
use tokio::task::JoinHandle;

fn handle_connection_error(_e: reqwest::Error) -> RetryPolicy<reqwest::Error> {
    RetryPolicy::Repeat
}

pub fn create_download_task(
    url: String,
    final_path: String,
    pb: ProgressBar,
    client: Client,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut path_without_last_vec = final_path.split("/").collect::<Vec<&str>>();
        path_without_last_vec.pop();
        let path_without_last = path_without_last_vec.join("/");
        create_dir_all(&path_without_last).unwrap();

        // idk how to get rid of clone
        // hours wasted: 0.5
        let action = || client.get(url.clone()).send();

        let (response, _err) = FutureRetry::new(move || action(), handle_connection_error)
            .await
            .map_err(|(e, _attempt)| e)
            .unwrap();
        let mut bytes = response.bytes().await.unwrap();
        let mut file = File::create(final_path).unwrap();
        file.write(&mut bytes).unwrap();
        pb.clone().inc(1);
    })
}
