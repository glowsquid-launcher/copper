use std::error::Error;
use std::ops::{Deref, DerefMut, Div};
use std::path::PathBuf;
use std::time::Duration;

use futures::stream::FuturesUnordered;
use log::{debug, trace};
use reqwest::{Client, ClientBuilder};
use tokio::fs::create_dir_all;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch::Receiver;
use tokio::task::{self, JoinHandle};
use tokio_retry::{strategy::FixedInterval, Retry};

pub fn create_download_task(
    url: String,
    final_path: PathBuf,
    client: Option<Client>,
) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
    trace!("Creating download task for {}", url);
    tokio::spawn(async move {
        let client = client
            .clone()
            .unwrap_or_else(|| Client::builder().build().unwrap());

        let final_path_str = final_path.to_string_lossy().into_owned();
        let mut path_without_last_vec = final_path_str.split("/").collect::<Vec<&str>>();
        path_without_last_vec.pop();
        let path_without_last = path_without_last_vec.join("/");
        create_dir_all(&path_without_last).await?;

        // idk how to get rid of clone
        // hours wasted: 2
        let action = || {
            debug!("Attempting to download {}", url);
            client.get(url.clone()).send()
        };

        let retry_strategy = FixedInterval::from_millis(100).take(3);

        let mut response = Retry::spawn(retry_strategy, action).await?;

        trace!("Creating file at {}", &final_path_str);
        let mut file = tokio::fs::File::create(&final_path_str).await?;

        trace!("Writing response to file");
        while let Some(chunk) = response.chunk().await? {
            file.write(&chunk).await?;
        }
        trace!("Wrote response to file");

        debug!("Downloaded {}", url);
        Ok(())
    })
}

pub type ListOfResultHandles =
    FuturesUnordered<task::JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>>;

#[derive(Clone, Copy)]
pub struct DownloadProgress {
    pub total_size: u64,
    pub finished: u64,
}

pub struct DownloadWatcher {
    pub progress_watcher: Receiver<DownloadProgress>,
    pub download_task: JoinHandle<()>,
}

pub fn create_client() -> Client {
    ClientBuilder::new()
        .connection_verbose(true)
        .pool_idle_timeout(Some(Duration::from_secs(600)))
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .build()
        .unwrap()
}

pub struct DivPathBuf(pub PathBuf);

impl Deref for DivPathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DivPathBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Div<&str> for DivPathBuf {
    type Output = DivPathBuf;

    fn div(self, rhs: &str) -> Self::Output {
        DivPathBuf(self.join(rhs))
    }
}

impl Div<&str> for &DivPathBuf {
    type Output = DivPathBuf;

    fn div(self, rhs: &str) -> Self::Output {
        DivPathBuf(self.join(rhs))
    }
}
