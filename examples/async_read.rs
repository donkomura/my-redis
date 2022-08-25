use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut buffer = [0; 10];

    // 最大10バイト読込む
    let n = f.read(&mut buffer[..]).await?;
    println!("The bytes: {:?}", &buffer[..n]);
    Ok(())
}
