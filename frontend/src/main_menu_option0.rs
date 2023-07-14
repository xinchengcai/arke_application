// Libs for ethereum contract 
use web3::types::H160;

// Libs for arke
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use arke_core::{random_id, StoreKey};
const IDENTIFIER_STRING_LENGTH: usize = 8;

// Libs for UI
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct MyInfo {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Contact {
    id: String,
    store_addr: H160,
    write_tag: StoreKey,
    read_tag: StoreKey,
    symmetric_key: Vec<u8>,
}


pub fn option0 () {
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open("src/my_info.json")
    .unwrap();

    let metadata = file.metadata().unwrap();
    if metadata.len() != 0 {
        let my_info: MyInfo = serde_json::from_reader(file).unwrap();
        println!("ID: {}", my_info.id);
    }
    else {
        let id = random_id!(IDENTIFIER_STRING_LENGTH);
        let my_info = json!({
            "id": id,
        });
        // Convert to a JSON string
        let data_string = my_info.to_string();
        // Write to the file
        file.write_all(data_string.as_bytes()).unwrap();
        println!("ID: {}", id);
    }
}