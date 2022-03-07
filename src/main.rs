mod downloader;

use std::fs;
use std::fs::{remove_dir_all, remove_file};
use std::io::{stdin};
use std::process::{Command, exit};
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use log::{debug, info, LevelFilter};
use crate::downloader::Downloader;

/// 替换flutter下的dart sdk为arm版本
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub commands: PatcherCommands,
    /// flutter所在目录
    #[clap(short, long)]
    path: Option<String>,
    /// debug 信息
    #[clap(short, long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum PatcherCommands {
    /// 执行替换dart sdk
    Patch {
        /// 下载线程数
        #[clap(short, long)]
        task_num: Option<u64>,
    },
    /// 重置为默认dart sdk
    Revert {},
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new().filter_level(LevelFilter::Debug).init();
    let args = Args::parse();
    log::set_max_level(if args.debug { LevelFilter::Debug } else { LevelFilter::Info });
    match args.commands {
        PatcherCommands::Patch {
            task_num,
        } => {
            info!("Start Patching");
            let path = match args.path {
                Some(p) => p,
                None => get_flutter_bin_path()?
            };
            let task_num = match task_num {
                Some(t) => t,
                None => 4
            };
            download_dart_sdk(&path, task_num).await?;
        }
        PatcherCommands::Revert {} => {
            info!("Start Reverting");
            let path = match args.path {
                Some(p) => p,
                None => get_flutter_bin_path()?
            };
            revert_dart_sdk(&path)?;
        }
    }
    Ok(())
}

/// 获取flutter路径
fn get_flutter_bin_path() -> Result<String> {
    let output = Command::new("which").arg("flutter").output()?;
    debug!("Execute Command 'which flutter'：{:?}", output);
    if output.status.success() {
        let mut path = String::from_utf8(output.stdout)?;
        path.replace_range(path.len() - 9.., "");
        info!("flutter 路径：{}", &path);
        Ok(path)
    } else {
        Err(wrap_stderr(output.stderr, String::from("获取flutter路径失败!")))
    }
}

/// 下载dart sdk 并替换
async fn download_dart_sdk(path: &str, task_num: u64) -> Result<()> {
    let dart_sdk_version = fs::read_to_string(format!("{path}/cache/dart-sdk/version"))?;
    let output = Command::new(format!("{path}/dart")).arg("--version").output()?;
    debug!("Execute Command 'dart --version'：{:?}", output);
    if !output.status.success() {
        return Err(wrap_stderr(output.stderr, String::from("dart 执行错误！")));
    }
    let dart_version_output = String::from_utf8(output.stdout)?;
    info!("Old bundled Dart SDK version:{dart_version_output}");
    info!("是否继续? (y/n) ");
    let mut ok = String::new();
    let _ = stdin().read_line(&mut ok)?;
    if ok.trim().eq("y") || ok.trim().eq("Y") {
        let channel = if dart_sdk_version.contains("dev") {
            "dev"
        } else if dart_sdk_version.contains("beta") {
            "beta"
        } else {
            "stable"
        };
        info!("Downloading Dart SDK {} for macos_arm64 on {} channel...",dart_sdk_version.trim(),channel);
        let download_url = format!("https://storage.googleapis.com/dart-archive/channels/{}/release/{}/sdk/dartsdk-macos-arm64-release.zip", channel, dart_sdk_version.trim());
        info!("fetching {download_url}");
        let download_zip = format!("{path}/cache/dart-sdk.zip");
        let downloader = Downloader::new(download_url, &download_zip, task_num);
        info!("downloading with {} process",task_num);
        downloader.run().await?;
        info!("Deleting bundled Dart SDK...");
        remove_dir_all(format!("{path}/cache/dart-sdk"))?;
        info!("Unzipping Dart SDK...");
        let out_dir = format!("{path}/cache");
        let output = Command::new("unzip").args(vec!["-o", &download_zip, "-d", &out_dir]).output()?;
        debug!("Execute Command 'unzip -o {} -d {}'：{:?}",&download_zip,&out_dir, output);
        if !output.status.success() {
            return Err(wrap_stderr(output.stderr, format!("解压文件失败")));
        }
        info!("Deleting zip file...");
        remove_file(&download_zip)?;
        info!("Deleting engine frontend_server.dart.snapshot file...");
        remove_file(format!("{path}/cache/artifacts/engine/darwin-x64/frontend_server.dart.snapshot"))?;
        info!("Copying Dart SDK frontend_server.dart.snapshot file to engine...");
        let _ = fs::copy(
            format!("{path}/cache/dart-sdk/bin/snapshots/frontend_server.dart.snapshot"),
            format!("{path}/cache/artifacts/engine/darwin-x64/frontend_server.dart.snapshot"),
        )?;
        let output = Command::new(format!("{path}/dart")).arg("--version").output()?;
        debug!("Execute Command 'dart --version'：{:?}", output);
        if !output.status.success() {
            return Err(wrap_stderr(output.stderr, String::from("dart 执行错误！")));
        }
        let dart_version_output = String::from_utf8(output.stdout)?;
        info!("New bundled Dart SDK version:{dart_version_output}");
        info!("All Done!");
        exit(0);
    } else {
        exit(1);
    }
}

/// 重置为flutter默认sdk
fn revert_dart_sdk(path: &str) -> Result<()> {
    info!("Deleting Dart SDK...");
    remove_dir_all(format!("{path}/cache"))?;
    info!("Executing flutter doctor...");
    Command::new("flutter").arg("doctor").status()?;
    Ok(())
}

fn wrap_stderr(stderr: Vec<u8>, err_else: String) -> anyhow::Error {
    if stderr.is_empty() {
        anyhow!(err_else)
    } else {
        let msg = String::from_utf8(stderr).unwrap_or(err_else);
        anyhow!(msg)
    }
}