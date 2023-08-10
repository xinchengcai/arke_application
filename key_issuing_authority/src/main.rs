use tokio::runtime::Runtime;
mod server;
use server::Server; 

fn main() {
    println!("Key-issuing authority running...");
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let server = Server::new().await;
        server.start().await.unwrap();
    });
}