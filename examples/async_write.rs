use tokio::io::{self, AsyncWriteExt};
use tokio::fs::File;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::create("foo.txt").await?;

    // 先頭からいくつかを書き込む
    let n = file.write(b"hoge fuga").await?;
    println!("Wrote the first {} bytes of 'hoge fuga'.", n);
    Ok(())
}
