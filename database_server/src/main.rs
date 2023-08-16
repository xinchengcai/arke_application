use tokio::runtime::Runtime;
mod server;
use server::Server; 

fn main() {
    println!("Server running...");
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let server = Server::new("src/all_users.json").await;
        server.start().await.unwrap();
    });
}