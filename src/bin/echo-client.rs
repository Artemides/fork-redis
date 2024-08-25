use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:4000").await.unwrap();
    let (rd, mut wr) = stream.into_split();

    tokio::spawn(async move {
        let stdin = io::stdin();
        let reader = io::BufReader::new(stdin);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            wr.write_all(format!("{line}\n").as_bytes()).await.unwrap();
        }
    });

    let reader = io::BufReader::new(rd);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.unwrap() {
        println!("\t{}", line.to_uppercase());
    }

    Ok(())
}
