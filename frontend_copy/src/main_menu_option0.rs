// Libs for arke
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use arke_core::random_id;
const IDENTIFIER_STRING_LENGTH: usize = 8;

// Libs for UI
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read};

#[derive(Deserialize, Debug)]
struct MyInfo {
    nickname: String,
    id: String,
    eth_addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    nickname: String,
    id: String,
    eth_addr: String,
}


pub fn option0 () {
    let mut my_info_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/my_info.json")
        .unwrap();
    let mut all_users_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("../../arke_application/all_users.json")
        .unwrap();

    let metadata = my_info_file.metadata().unwrap();
    if metadata.len() != 0 {
        let my_info: MyInfo = serde_json::from_reader(my_info_file).unwrap();
        println!("ID: {}    Nickname: {}    Eth address: {}", my_info.id, my_info.nickname, my_info.eth_addr);
    }
    else {
        let id = random_id!(IDENTIFIER_STRING_LENGTH);
        let eth_addr = dialoguer::Input::<String>::new()
            .with_prompt("What is your eth address")
            .interact()
            .unwrap();
        let nickname = dialoguer::Input::<String>::new()
            .with_prompt("What nickname would you like")
            .interact()
            .unwrap();
        let my_info = json!({
            "nickname": nickname,
            "id": id,
            "eth_addr": eth_addr,
        });
        // Convert to a JSON string
        let data_string = my_info.to_string();
        // Write to files
        my_info_file.write_all(data_string.as_bytes()).unwrap();
        println!("ID: {}    Nickname: {}     Eth address: {}",  id, nickname, eth_addr);

        // Write to files
        // Read the existing users
        let new_user = User {
            nickname: nickname,
            id: id,
            eth_addr: eth_addr,
        };
        let mut contents = String::new();
        all_users_file.read_to_string(&mut contents).unwrap();
        let mut users: Vec<User> = match serde_json::from_str(&contents) {
            Ok(users) => users,
            Err(_) => Vec::new(), // If error while parsing, treat as empty list
        };

        // Append the new contact
        users.push(new_user);
        // Write contacts back to the file
        let all_users_file = File::create("../../arke_application/all_users.json").unwrap();
        serde_json::to_writer(&all_users_file, &users).unwrap();
    }
}