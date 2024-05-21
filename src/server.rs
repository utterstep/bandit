use std::{sync::Arc, time::Duration};

use eyre::{Result, WrapErr};
use rand::Rng;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, warn};

use crate::consts::HANDSHAKE;

#[tracing::instrument]
pub async fn server(bind_to: String, payload_size: usize, packet_size: usize) -> Result<()> {
    let mut buf = vec![0; payload_size];
    info!("populating payload buffer with {} bytes", payload_size);
    let mut rng = rand::thread_rng();
    rng.fill(&mut buf[..]);
    let buf = Arc::from(buf);

    let listener = TcpListener::bind(bind_to)
        .await
        .wrap_err("failed to bind to address")?;

    loop {
        let (socket, addr) = listener
            .accept()
            .await
            .wrap_err("failed to accept connection")?;

        tokio::spawn(serve_client(socket, addr, Arc::clone(&buf), packet_size));
    }
}

#[tracing::instrument(skip(socket, buf), err, ret)]
pub async fn serve_client(
    mut socket: TcpStream,
    addr: std::net::SocketAddr,
    buf: Arc<[u8]>,
    packet_size: usize,
) -> Result<Duration> {
    let mut pos = 0;

    socket.set_nodelay(true).wrap_err("failed to set nodelay")?;
    socket
        .write_all(&HANDSHAKE)
        .await
        .wrap_err("failed to send handshake")?;

    let start = std::time::Instant::now();
    loop {
        match socket
            .write(&buf[pos..buf.len().min(pos + packet_size)])
            .await
        {
            Ok(0) => {
                warn!("client {} disconnected", addr);
                break;
            }
            Ok(n) => {
                pos += n;
            }
            Err(e) => {
                error!("error sending data to {}: {}", addr, e);

                return Err(e).wrap_err("failed to send data");
            }
        }

        if pos == buf.len() {
            break;
        }
    }

    let elapsed = start.elapsed();
    info!(
        "sent {} bytes in {} ms to {}",
        pos,
        elapsed.as_millis(),
        addr
    );
    info!(
        "speed: {:.2} MiB/s",
        (pos as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64()
    );

    socket
        .shutdown()
        .await
        .wrap_err("failed to shutdown socket")?;

    Ok(elapsed)
}
