use vigem_rs::client::VigemClient;

fn main() -> anyhow::Result<()> {
    let mut client = VigemClient::new();
    client.connect()?;


    println!("Client created: {:?}", client);

    Ok(())
}