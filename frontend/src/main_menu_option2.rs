#![allow(unused_assignments)]
#![allow(dead_code)]

// Libs for ethereum contract 
use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;

// Libs for arke
use arke_core::StoreKey;
use crate::arke_frontend::Arke;

// Libs for UI
use serde::{Serialize, Deserialize};
use std::fs::{OpenOptions, File};
use std::io::Read;

#[derive(Deserialize, Debug)]
struct MyInfo {
    id: String,
    nickname: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Contact {
    nickname: String,
    id: String,
    store_addr: H160,
    own_write_tag: StoreKey,
    own_read_tag: StoreKey,
    contact_write_tag: StoreKey,
    contact_read_tag: StoreKey,
    symmetric_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    nickname: String,
    id: String,
}

pub fn option2() {
    let want_contact_discovery_nickname = dialoguer::Input::<String>::new()
        .with_prompt("Who do you want to make contact discovery?")
        .interact()
        .unwrap();

    let mut want_contact_discovery_id = String::new();

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/contacts.json").unwrap();
    // Read the existing contacts
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let contacts: Vec<Contact> = match serde_json::from_str(&contents) {
        Ok(contacts) => contacts,
        Err(_) => Vec::new(), // If error while parsing, treat as empty list
    };
    
    let contact = contacts.iter().find(|&c| c.nickname == want_contact_discovery_nickname);
    match contact {
        Some(contact) => {
            println!("{:?} is already in your contacts", contact.nickname);
            return;
        },
        None => {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open("../../arke_application/all_users.json").unwrap();
            // Read the existing users
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let users: Vec<User> = match serde_json::from_str(&contents) {
                Ok(users) => users,
                Err(_) => Vec::new(), // If error while parsing, treat as empty list
            };

            let user = users.iter().find(|&u| u.nickname == want_contact_discovery_nickname);
            match user {
                Some(user) => {
                    println!("Found the user {:?}", user.nickname);
                    want_contact_discovery_id = user.id.clone();
                },
                None => {
                    println!("Not a user");
                    return;
                }
            }
        },
    }
    
    let file = OpenOptions::new()
        .read(true)
        .open("src/my_info.json").unwrap();
    let my_info: MyInfo = serde_json::from_reader(file).unwrap();

    println!("Creating new contact object");
    let crypto = Arke::id_nike_and_handshake(my_info.id.clone(), want_contact_discovery_id.clone());
    let symmetric_key = crypto.symmetric_key;
    let own_write_tag = crypto.alice_write_tag;
    let own_read_tag = crypto.alice_read_tag;
    let contact_write_tag = crypto.bob_write_tag;
    let contact_read_tag = crypto.bob_read_tag;
    let store_addr_string = hex::encode(Arke::to_address(&own_write_tag));
    let store_addr = Address::from_str(&store_addr_string).unwrap();
    let new_contact = Contact {
        nickname: want_contact_discovery_nickname.clone(), 
        id: want_contact_discovery_id.clone(),
        store_addr: store_addr.clone(),
        own_write_tag: own_write_tag.clone(),
        own_read_tag: own_read_tag.clone(),
        contact_write_tag: contact_write_tag.clone(),
        contact_read_tag: contact_read_tag.clone(),
        symmetric_key: symmetric_key.clone(),
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


    let new_contact = Contact {
        nickname: my_info.nickname.clone(), 
        id: my_info.id.clone(),
        store_addr: store_addr.clone(),
        own_write_tag: contact_write_tag.clone(),
        own_read_tag: contact_read_tag.clone(),
        contact_write_tag: own_write_tag.clone(),
        contact_read_tag: own_read_tag.clone(),
        symmetric_key: symmetric_key.clone(),
    };
    // Write to the file
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("../frontend_copy/src/contacts.json").unwrap();
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
    let file = File::create("../frontend_copy/src/contacts.json").unwrap();
    serde_json::to_writer(&file, &contacts).unwrap();
}