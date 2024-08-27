use tokio_stream::StreamExt;

async fn publish() -> mini_redis::Result<()> {
    let mut client = mini_redis::client::connect("127.0.0.1:4000").await.unwrap();
    client.publish("numbers", "1".into()).await?;
    client.publish("numbers", "two".into()).await?;
    client.publish("numbers", "3".into()).await?;
    client.publish("numbers", "four".into()).await?;
    client.publish("numbers", "5".into()).await?;
    client.publish("numbers", "six".into()).await?;

    Ok(())
}
async fn subscribe() -> mini_redis::Result<()> {
    let client = mini_redis::client::connect("127.0.0.1:4000").await?;
    let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;

    let stream = subscriber
        .into_stream()
        .filter(|msg| match msg {
            Ok(msg) if msg.content.len() == 1 => true,
            _ => false,
        })
        .map(|msg| msg.unwrap().content)
        .take(3);

    tokio::pin!(stream);

    while let Some(msg) = stream.next().await {
        println!("GOT => {:?}", msg);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    tokio::spawn(async { publish().await });

    subscribe().await?;
    println!("DONE");
    Ok(())
}
