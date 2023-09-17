use tokio::runtime::Runtime;
mod registration_authority;
use registration_authority::registrationAuthority; 

fn main() {
    println!("Registration authority running...");
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let registration_authority = registrationAuthority::new().await;
        registration_authority.start().await.unwrap();
    });
}