use mini_redis::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = mini_redis::client::connect("127.0.0.1:4000").await?;
    client.set("rust", "0.1.0".into()).await?;
    let value = client.get("rust").await?;

    println!("got value from the server; result={:?}", value);

    Ok(())
}
