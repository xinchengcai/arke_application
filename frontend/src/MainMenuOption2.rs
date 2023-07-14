// Libs for ethereum contract 
use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;

// Libs for arke
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use arke_core::{random_id, StoreKey};
const IDENTIFIER_STRING_LENGTH: usize = 8;
use crate::arke_frontend::Arke;

// Libs for UI
use serde::{Serialize, Deserialize};
use std::fs::{OpenOptions, File};
use std::io::Read;

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

pub fn option2() {
    let want_contact_discovery_id = dialoguer::Input::<String>::new()
    .with_prompt("Who do you want to make contact discovery?")
    .interact()
    .unwrap();

    let want_contact_discovery_id = random_id!(IDENTIFIER_STRING_LENGTH);

    let file = OpenOptions::new()
        .read(true)
        .open("src/my_info.json").unwrap();
    let my_info: MyInfo = serde_json::from_reader(file).unwrap();

    println!("Creating new contact object");
    let crypto = Arke::id_nike_and_handshake(my_info.id.clone(), want_contact_discovery_id.clone());
    let symmetric_key = crypto._symmetric_key;
    let write_tag = crypto._alice_write_tag;
    let read_tag = crypto._bob_read_tag;
    let store_addr_string = hex::encode(Arke::to_address(&write_tag));
    let store_addr = Address::from_str(&store_addr_string).unwrap();
    let new_contact = Contact {
        id: want_contact_discovery_id.clone(),
        store_addr: store_addr,
        write_tag: write_tag,
        read_tag: read_tag,
        symmetric_key: symmetric_key,
    };
    // Write to the file
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/contacts.json").unwrap();
    // Read the existing contacts
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mut contacts: Vec<Contact> = match serde_json::from_str(&contents) {
        Ok(contacts) => contacts,
        Err(_) => Vec::new(), // If error while parsing, treat as empty list
    };
    // Append the new contact
    contacts.push(new_contact);
    // Write contacts back to the file
    let file = File::create("src/contacts.json").unwrap();
    serde_json::to_writer(&file, &contacts).unwrap();
}