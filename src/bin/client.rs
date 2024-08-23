use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};

enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Command>(32);

    let manager = tokio::spawn(async move {
        use mini_redis;
        let mut client = mini_redis::client::connect("127.0.0.1:4000").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let value = client.get(key.as_str()).await;
                    let _ = resp.send(value);
                }
                Command::Set { key, value, resp } => {
                    let res = client.set(key.as_str(), value).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    let task_sender = tx.clone();
    let task1 = tokio::spawn(async move {
        let (sender, receiver) = oneshot::channel();

        let get = Command::Get {
            key: "foo".to_string(),
            resp: sender,
        };
        task_sender.send(get).await.unwrap();

        let response = receiver.await;
        println!("Get =>  {:?}", response);
    });

    let task2 = tokio::spawn(async move {
        let (sender, receiver) = oneshot::channel();
        let set = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            resp: sender,
        };
        tx.send(set).await.unwrap();

        let response = receiver.await;
        println!("Set =>  {:?}", response);
    });

    task2.await.unwrap();
    task1.await.unwrap();
    manager.await.unwrap();
}
