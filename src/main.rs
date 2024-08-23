use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4545").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        process(stream).await
    }
}

async fn process(stream: TcpStream) {
    let mut connection = Connection::new(stream);

    if let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT {:?}", frame);

        let response = Frame::Error("unimplemented".to_string());

        connection.write_frame(&response).await.unwrap();
    }
}
