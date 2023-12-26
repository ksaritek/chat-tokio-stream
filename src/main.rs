use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("localhost:8080").await?;
    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (socket, addr) = listener.accept().await?;
        let tx = tx.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_connection(socket, addr, tx).await {
                eprintln!("Error handling connection: {}", err);
            }
        });
    }
}

async fn handle_connection(
    socket: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
    tx: broadcast::Sender<(String, std::net::SocketAddr)>,
) -> Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                let bytes_read = result?;
                if bytes_read == 0 {
                    break;
                }

                tx.send((line.clone(), addr))?;
                line.clear();
            }
            result = rx.recv() => {
                let (msg, other_addr) = result?;

                if addr != other_addr {
                    writer.write_all(msg.as_bytes()).await?;
                    writer.flush().await?;
                }
            }
        }
    }

    Ok(())
}
