use tokio::runtime::Runtime;
mod key_issuing_authority;
use key_issuing_authority::keyIssuingAuthority; 

fn main() {
    println!("Key-issuing authority running...");
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let key_issuing_athority = keyIssuingAuthority::new().await;
        key_issuing_athority.start().await.unwrap();
    });
}