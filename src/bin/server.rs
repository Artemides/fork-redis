use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type SharedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4000").await.unwrap();
    println!("Listennin on: 127.0.0.1:4000");

    let db = new_shared_db(5);
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let db = db.clone();

        println!(
            "Incomming Conection: {} Accepted ",
            stream.local_addr().unwrap()
        );
        tokio::spawn(async move {
            process(stream, db).await;
        });
    }
}

async fn process(stream: TcpStream, db: SharedDb) {
    let mut connection = Connection::new(stream);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                let mut shard = db[hash(cmd.key()) % db.len()].lock().unwrap();

                shard.insert(cmd.key().into(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                let shard = db[hash(cmd.key()) % db.len()].lock().unwrap();

                if let Some(value) = shard.get(cmd.key()) {
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

fn hash(key: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish() as usize
}

fn new_shared_db(num_shards: usize) -> SharedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        let shard = Mutex::new(HashMap::new());
        db.push(shard);
    }

    Arc::new(db)
}
