use tokio::{
    io::{self},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let (mut rd, mut wr) = stream.split();
            if io::copy(&mut rd, &mut wr).await.is_err() {
                eprintln!("Error Echoing");
            }
        });
    }
}
