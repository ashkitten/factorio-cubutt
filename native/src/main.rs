use std::{env, os::unix::fs::FileTypeExt, path::PathBuf, str::FromStr};

use buttplug::{
    client::{ButtplugClient, ScalarValueCommand},
    core::connector::new_json_ws_client_connector,
};
use clap::Parser;
use miette::{IntoDiagnostic, Result};
use nix::{sys::stat::Mode, unistd::mkfifo};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::unix::pipe,
};

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
    let path = {
        let home = env::var("HOME").into_diagnostic()?;
        let mut path = PathBuf::from(&home);
        path.push(".factorio/script-output/buttplug.commands");
        path
    };

    match path.metadata() {
        Ok(metadata) if metadata.file_type().is_fifo() => (),
        res => {
            if res.is_ok() {
                fs::remove_file(&path).await.into_diagnostic()?;
            }
            mkfifo(&path, Mode::S_IRWXU).into_diagnostic()?;
        },
    }

    let pipe = pipe::OpenOptions::new()
        .read_write(true) // makes it resilient to the other end closing
        .open_receiver(path)
        .into_diagnostic()?;
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
