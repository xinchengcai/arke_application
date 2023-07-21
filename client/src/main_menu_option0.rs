use rand::{distributions::Alphanumeric, thread_rng, Rng};
use arke_core::random_id;
const IDENTIFIER_STRING_LENGTH: usize = 8;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, BufWriter, Cursor};
use serde::{Serialize, Deserialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::fs::File;

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
struct MyInfo {
    nickname: String,
    id_string: String,
    eth_addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    nickname: String,
    id_string: String,
    eth_addr: String,
    finding: String,
    key_id: String,
    unread: bool,
}



pub async fn option0 () -> Result<(), Box<dyn std::error::Error>> {
    // Read my_info.bin
    let mut my_info_file = File::open("src/my_info.bin").unwrap();
    let mut deserialized: Vec<u8> = Vec::new();
    my_info_file.read_to_end(&mut deserialized).unwrap();

    // Check whether my_info.bin is empty or not, i.e. whether a user or not
    let metadata = my_info_file.metadata().unwrap();

    // If not empty, I am a user, read my_info
    if metadata.len() != 0 {
        // Derialize my_info.bin to read my_info object
        let mut cursor = Cursor::new(&deserialized);
        let my_info = MyInfo::deserialize(&mut cursor).unwrap();
        // Print my_info
        println!("ID string: {}    Nickname: {}    Eth address: {}", my_info.id_string, my_info.nickname, my_info.eth_addr);
    }

    // If empty, I am not user, create new user info
    else { 
        // Generate new id_string
        let id_string = random_id!(IDENTIFIER_STRING_LENGTH);
        // Ask my eth_addr
        let eth_addr = dialoguer::Input::<String>::new()
            .with_prompt("What is your eth address")
            .interact()
            .unwrap();
        // Ask my nickname
        let nickname = dialoguer::Input::<String>::new()
            .with_prompt("What nickname would you like")
            .interact()
            .unwrap();
        // Create new my_info object
        let my_info = MyInfo {
            nickname: nickname,
            id_string: id_string,
            eth_addr: eth_addr
        };
        // Serialize the new my_info object
        let mut serialized: Vec<u8> = Vec::new();
        my_info.serialize(&mut serialized).unwrap();
        // Write to my_info.bin
        let mut my_info_file = BufWriter::new(File::create("src/my_info.bin").unwrap());
        my_info_file.write_all(&serialized).unwrap();
        // Print my_info
        println!("ID string: {}    Nickname: {}    Eth address: {}",
                my_info.id_string, my_info.nickname, my_info.eth_addr);

        // Create new user object
        let new_user = User {
            nickname: my_info.nickname,
            id_string: my_info.id_string,
            eth_addr: my_info.eth_addr,
            finding: String::new(),
            key_id: String::new(),
            unread: false,
        };
        // Connect to the server
        let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
        // Create the request for add_user, i.e. write the new user object to all_users.json in server
        let request = json!({
            "action": "add_user",
            "id_string": new_user.id_string,
            "nickname": new_user.nickname,
            "eth_addr": new_user.eth_addr,
            "finding": new_user.finding,
            "key_id": new_user.key_id,
            "unread": new_user.unread,
        });
        // Convert the request to a byte array
        let request_bytes = serde_json::to_vec(&request)?;
        // Write the request to the stream
        stream.write_all(&request_bytes).await?;
        // Create a buffer to read the response into
        let mut buf = vec![0; 1024];
        let n = stream.read(&mut buf).await?;
        // Parse the response
        let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;
        // Print the response
        println!("Response: {}", response);
    }

    Ok(())
}