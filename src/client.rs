use eyre::{Result, WrapErr};
use tokio::{io::AsyncReadExt, net::TcpStream};

use tracing::info;

use crate::consts::HANDSHAKE;

#[tracing::instrument(err)]
pub async fn client_test(server: &str, packet_size: usize) -> Result<()> {
    let mut socket = TcpStream::connect(server)
        .await
        .wrap_err("failed to connect to server")?;

    // check if server sends the correct handshake
    let mut handshake = [0; 12];
    socket
        .read_exact(&mut handshake)
        .await
        .wrap_err("failed to read handshake")?;

    if handshake != HANDSHAKE {
        eyre::bail!("server sent incorrect handshake");
    }

    let start = std::time::Instant::now();

    let mut buf = vec![0; packet_size];
    let mut total_read = 0;
    loop {
        let n = socket.read(&mut buf).await?;
        total_read += n;
        if n == 0 {
            break;
        }
    }

    let elapsed = start.elapsed();

    info!("client received {} bytes in {:?}", total_read, elapsed);
    info!(
        "speed: {:.2} MiB/s",
        (total_read as f64 / 1024.0 / 1024.0) / (elapsed.as_secs_f64())
    );

    Ok(())
}
