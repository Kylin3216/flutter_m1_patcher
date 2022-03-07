use anyhow::{Error, Result};
use futures::future::join_all;
use futures::StreamExt;
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE};
use reqwest::{IntoUrl, Url};
use std::io::SeekFrom;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{File, remove_file};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;

pub struct Downloader<U: IntoUrl, P: AsRef<Path>> {
    url: U,
    path: P,
    task_num: u64,
}

impl<U: IntoUrl, P: AsRef<Path>> Downloader<U, P> {
    pub fn new(url: U, path: P, task_num: u64) -> Downloader<U, P> {
        Downloader {
            url,
            path,
            task_num,
        }
    }

    pub async fn run(self) -> Result<()> {
        let url = self.url.into_url()?;
        let mut handles = vec![];
        let (range, length) = check_request_range(url.clone()).await?;
        let file = Arc::new(Mutex::new(File::create(&self.path).await?));
        let is_error = if range {
            let task_length = length / self.task_num;
            for i in 0..(self.task_num - 1) {        // 线程数必须大于等于1
                handles.push(tokio::spawn(download(
                    url.clone(),
                    (task_length * i, task_length * (i + 1) - 1),
                    true,
                    Arc::clone(&file),
                )));
            }
            handles.push(tokio::spawn(
                download(url.clone(), (task_length * (self.task_num - 1), u64::MAX), true, Arc::clone(&file))
            ));
            let ret = join_all(handles).await;
            drop(file);
            ret.into_iter().flatten().any(|n| n.is_err())
        } else {
            download(url.clone(), (0, length - 1), false, file)
                .await
                .is_err()
        };
        if is_error {
            remove_file(&self.path).await?;
            Err(Error::msg("download file error"))
        } else {
            Ok(())
        }
    }
}

async fn check_request_range(url: Url) -> Result<(bool, u64)> {
    let mut range = false;
    let req = reqwest::Client::new().head(url);
    let rep = req.send().await?;
    if !rep.status().is_success() {
        return Err(Error::msg("request fail"));
    }
    let headers = rep.headers();
    if headers
        .get(ACCEPT_RANGES)
        .map(|val| (val.to_str().ok()?.eq("bytes")).then(|| ()))
        .flatten()
        .is_some()
    {
        range = true;
    }
    let length = headers
        .get(CONTENT_LENGTH)
        .map(|val| val.to_str().ok())
        .flatten()
        .map(|val| val.parse().ok())
        .flatten()
        .ok_or(Error::msg("get length fail"))?;
    Ok((range, length))
}

async fn download(url: Url, (mut start, end): (u64, u64), is_partial: bool,
                  file: Arc<Mutex<File>>) -> Result<()> {
    let req = reqwest::Client::new().get(url);

    let req = if is_partial {
        if end == u64::MAX {
            req.header(RANGE, format!("bytes={}-{}", start, ""))
        } else {
            req.header(RANGE, format!("bytes={}-{}", start, end))
        }
    } else {
        req
    };
    let rep = req.send().await?;
    if !rep.status().is_success() {
        return Err(Error::msg("request fail"));
    }
    let mut stream = rep.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let mut chunk = chunk?;
        let mut file = file.lock().await;
        file.seek(SeekFrom::Start(start)).await?;
        start += chunk.len() as u64;
        file.write_all_buf(&mut chunk).await?;
    }
    Ok(())
}