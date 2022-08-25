use tokio::io::{self, AsyncReadExt};
use tokio::fs::File;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut buffer = Vec::new();

    // ファイルをすべて読む
    let n = f.read_to_end(&mut buffer).await?;
    println!("The bytes: {:?}", &buffer[..n]);
    Ok(())
}
