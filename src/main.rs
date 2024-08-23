use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mini_redis::{Command, Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Vec<u8>>>>;

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:4000").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            process(stream, db).await;
        });
    }
}

async fn process(stream: TcpStream, db: Db) {
    let mut connection = Connection::new(stream);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                db.lock()
                    .unwrap()
                    .insert(cmd.key().into(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => Frame::Error(format!("unimplemented {:?}", cmd).to_string()),
        };

        connection.write_frame(&response).await.unwrap();
    }
}
