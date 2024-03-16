use std::{path::Path, str::FromStr};

use buttplug::{
    client::{ButtplugClient, ScalarValueCommand},
    core::connector::new_json_ws_client_connector,
};
use clap::Parser;
use miette::{IntoDiagnostic, Result};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
};

#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use nix::{sys::stat::Mode, unistd::mkfifo};
#[cfg(unix)]
use tokio::net::unix::pipe;

#[cfg(windows)]
use tokio::net::windows::named_pipe::{self, PipeMode};

/// Native connector for the CuButt Factorio mod
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Websocket address of the Buttplug server to connect to
    ws_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    #[cfg(unix)]
    let pipe = {
        let home = dirs_sys::home_dir().unwrap();
        let path = home.join(".factorio/script-output/buttplug.commands");

        match fs::metadata(&path).await {
            Ok(metadata) if metadata.file_type().is_fifo() => (),
            _ => mkfifo(&path, Mode::S_IRWXU).into_diagnostic()?,
        }

        pipe::OpenOptions::new()
            .read_write(true) // makes it resilient to the other end closing
            .open_receiver(path)
            .into_diagnostic()?
    };

    #[cfg(windows)]
    let pipe = {
        let appdata = dirs_sys::known_folder_roaming_app_data().unwrap();
        let path = appdata.join("Factorio/script-output/buttplug.commands");

        const PIPE_NAME: &str = r"\\.\pipe\cubutt";

        match fs::metadata(&path).await {
            Ok(metadata) if metadata.is_symlink() => (),
            _ => fs::symlink_file(&path, Path::new(PIPE_NAME)).await.into_diagnostic()?,
        }

        named_pipe::ServerOptions::new()
            .pipe_mode(PipeMode::Message)
            .create(PIPE_NAME).into_diagnostic()?
    };
    
    let reader = BufReader::new(pipe);
    let mut lines = reader.lines();

    let client = ButtplugClient::new("CuButt");
    let connector = new_json_ws_client_connector(&args.ws_addr);
    client.connect(connector).await.into_diagnostic()?;

    client.start_scanning().await.into_diagnostic()?;

    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(value) = f64::from_str(&line) {
            for device in client.devices() {
                device
                    .vibrate(&ScalarValueCommand::ScalarValue(value))
                    .await
                    .into_diagnostic()?;
            }
        }
    }

    Ok(())
}
