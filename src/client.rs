use std::fmt::Write;

use eyre::{Result, WrapErr};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;

use crate::consts::{HANDSHAKE, HEADER_SIZE};

#[tracing::instrument(err)]
pub async fn client_tcp(server: &str, packet_size: usize) -> Result<()> {
    let mut socket = TcpStream::connect(server)
        .await
        .wrap_err("failed to connect to server")?;

    // check if server sends the correct handshake
    let mut handshake = [0; HANDSHAKE.len()];
    socket
        .read_exact(&mut handshake)
        .await
        .wrap_err("failed to read handshake")?;

    if handshake != HANDSHAKE {
        eyre::bail!("server sent incorrect handshake");
    }

    socket
        .write_all(&HANDSHAKE)
        .await
        .wrap_err("failed to send handshake")?;

    let mut header_buf = [0; HEADER_SIZE];
    socket
        .read_exact(&mut header_buf)
        .await
        .wrap_err("failed to read header")?;

    let len = match header_buf {
        [b'B', b'H', b'E', b'A', b'D', len @ ..] => u64::from_le_bytes(len) as usize,
        _ => {
            eyre::bail!("received invalid header");
        }
    };

    info!(len, "received header, {} bytes to follow", len);

    let pb = ProgressBar::new(len as u64);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}, {speed_mbit})")
        .wrap_err("failed to set progress bar style")?
        .with_key("speed_mbit", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.2} Mbit/s", state.per_sec() * 8.0 / 1024.0 / 1024.0).unwrap())
        .progress_chars("#>-"));

    let start = std::time::Instant::now();

    let mut buf = vec![0; packet_size];
    let mut total_read = 0;
    loop {
        let n = socket.read(&mut buf).await?;
        total_read += n;
        pb.inc(n as u64);

        if n == 0 || total_read == len {
            break;
        }
    }

    let elapsed = start.elapsed();
    pb.finish_with_message("Download complete");

    info!(
        "client received {} bytes in {} ms",
        total_read,
        elapsed.as_millis()
    );
    info!(
        "speed: {:.2} MiB/s",
        (total_read as f64 / 1024.0 / 1024.0) / (elapsed.as_secs_f64())
    );

    Ok(())
}
