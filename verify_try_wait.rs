use std::process::Stdio;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("true")
        .stdout(Stdio::null())
        .spawn()?;

    // Wait for it to finish
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Check try_wait
    match child.try_wait()? {
        Some(status) => println!("try_wait: {:?}", status),
        None => println!("try_wait: None"),
    }

    // Check wait
    let status = child.wait().await?;
    println!("wait: {:?}", status);

    Ok(())
}
