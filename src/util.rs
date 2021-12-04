use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use indicatif::ProgressBar;
use reqwest::Client;
use tokio::task::JoinHandle;
use tokio_retry::{strategy::FixedInterval, Retry};

pub fn create_download_task(
    url: String,
    final_path: String,
    pb: Option<ProgressBar>,
    client: Option<Client>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut path_without_last_vec = final_path.split("/").collect::<Vec<&str>>();
        path_without_last_vec.pop();
        let path_without_last = path_without_last_vec.join("/");
        create_dir_all(&path_without_last).unwrap();

        // idk how to get rid of clone
        // hours wasted: 2
        let action = || {
            client
                .clone()
                .unwrap_or_else(|| Client::builder().build().unwrap())
                .get(url.clone())
                .send()
        };

        let retry_strategy = FixedInterval::from_millis(100).take(3);

        let response = Retry::spawn(retry_strategy, action).await.unwrap();

        let mut bytes = response.bytes().await.unwrap();
        let mut file = File::create(final_path).unwrap();
        file.write(&mut bytes).unwrap();

        if let Some(pb) = pb {
            pb.inc(1);
        }
    })
}
