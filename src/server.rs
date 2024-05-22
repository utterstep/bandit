use std::{net::SocketAddr, sync::Arc, time::Duration};

use eyre::{Result, WrapErr};
use rand::Rng;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, warn};

use crate::consts::{header, HANDSHAKE};

pub async fn server(bind_to: String, payload_size: usize, packet_size: usize) -> Result<()> {
    let mut buf = vec![0; payload_size];
    info!(
        payload_size,
        "populating payload buffer with {} bytes", payload_size
    );
    let mut rng = rand::thread_rng();
    rng.fill(&mut buf[..]);
    let buf = Arc::from(buf);
    info!("payload buffer populated");

    let listener = TcpListener::bind(&bind_to)
        .await
        .wrap_err("failed to bind to address")?;
    info!(binded_to = bind_to, packet_size, "server started listening");

    loop {
        let (socket, addr) = listener
            .accept()
            .await
            .wrap_err("failed to accept TCP connection")?;

        info!(%addr, "accepted TCP connection");

        tokio::spawn(serve_client(socket, addr, Arc::clone(&buf), packet_size));
    }
}

#[tracing::instrument(skip(socket, buf), err, ret)]
pub async fn serve_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    buf: Arc<[u8]>,
    packet_size: usize,
) -> Result<Duration> {
    let mut pos = 0;

    socket.set_nodelay(true).wrap_err("failed to set nodelay")?;
    socket
        .write_all(&HANDSHAKE)
        .await
        .wrap_err("failed to send handshake")?;

    let mut handshake_buf = [0; HANDSHAKE.len()];
    socket
        .read_exact(&mut handshake_buf)
        .await
        .wrap_err("failed to read handshake")?;

    if handshake_buf != HANDSHAKE {
        eyre::bail!("client sent incorrect handshake");
    }

    socket
        .write_all(&header(buf.len()))
        .await
        .wrap_err("failed to send header")?;

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
