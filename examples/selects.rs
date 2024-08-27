use std::time::Duration;

use rand::Rng;
use tokio::sync::mpsc;

async fn yield_nums(tx: mpsc::Sender<i32>) {
    for num in 1..128 {
        let success = rand::thread_rng().gen_bool(1.0 / 3.0);
        if success {
            if tx.send(num).await.is_err() {
                break;
            }
        }
    }
}

async fn some_operation() {
    tokio::time::sleep(Duration::from_millis(1)).await;
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(128);

    tokio::spawn(async move {
        yield_nums(tx).await;
    });

    let op = some_operation();
    tokio::pin!(op);
    loop {
        tokio::select! {
             _ = &mut op =>{
                println!("op finish");
                break;
             },
             Some(v)= rx.recv() =>{
                if v%4==0{
                    println!("got: {v}");
                    break;
                }
             }

        }
    }
}
