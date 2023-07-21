#![allow(non_snake_case)]

use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;
use crate::key_value_store_frontend::KeyValueStore;
use arke_core::StoreKey;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use serde::{Serialize, Deserialize};
use std::fs::{OpenOptions, File};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, Cursor};

#[derive(Serialize, Deserialize, Debug)]
struct Contact {
    nickname: String,
    id_string: String,
    store_addr: H160,
    own_write_tag: StoreKey,
    own_read_tag: StoreKey,
    symmetric_key: Vec<u8>,
}

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
struct MyInfo {
    nickname: String,
    id_string: String,
    eth_addr: String,
}

pub async fn option3() {
    // Setup the contract and an interface to access it's functionality
    let transport = web3::transports::Http::new("HTTP://127.0.0.1:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let Store = KeyValueStore::new(
        &web3,
        // Update to match the deployed address
        "0xDD7FE36d9340b502F143a4B43663613b0b29cc1f".to_string(),
    ).await;

    // Read to my_contact.json
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/contacts.json")
        .unwrap();
    // Check whether contacts.json is empty or not, i.e. whether there are contacts or not
    let metadata = file.metadata().unwrap();
    // If empty, return to the main menu
    if metadata.len() == 0 {
        println!("No contacts");
        return;
    }

    // Derialize contacts.json to read contact objects 
    let contacts: Vec<Contact> = serde_json::from_reader(file).unwrap();
    // Convert each contact to a string representation and collect them into a vector
    let mut ContactsMenu: Vec<String> = contacts.iter()
        .map(|contact| { format!("ID: {}     nickname: {}", contact.id_string, contact.nickname)}).collect();
    // Add go back to the end of the vector
    ContactsMenu.push("Go back".to_string());

    // Display the vector as a menu
    loop {
        let ContactsMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Which contact would you like to delete?")
        .default(0)
        .items(&ContactsMenu[..])
        .interact()
        .unwrap();
        match ContactsMenuSelection {
            index if index < contacts.len() => {
                // If selected a contact
                let selected_contact = &contacts[index];

                // Delete in the saved contacts
                // Read then write to my_contact.json
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("src/contacts.json").unwrap();
                // Derialize contacts.json to read contact objects 
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                // Convert each contact to a string representation and collect them into a vector
                let mut contacts: Vec<Contact> = match serde_json::from_str(&contents) {
                    Ok(contacts) => contacts,
                    Err(_) => Vec::new(),
                };
                // remove the contact from the vector
                contacts.remove(index);
                // Write contacts back to the file
                let file = File::create("src/contacts.json").unwrap();
                serde_json::to_writer(&file, &contacts).unwrap();

                /* Delete */
                let store_addr = selected_contact.store_addr.clone(); 
                let mut my_info_file = File::open("src/my_info.bin").unwrap();
                let mut deserialized: Vec<u8> = Vec::new();
                my_info_file.read_to_end(&mut deserialized).unwrap();
                let mut cursor = Cursor::new(&deserialized);
                let my_info = MyInfo::deserialize(&mut cursor).unwrap();
                let deleter_addr = Address::from_str(&my_info.eth_addr).unwrap();
                // Delete on the map of the store
                Store.Delete(store_addr, deleter_addr).await;

                // Return to the main menu
                break;
            }

            // If selected go back
            _ => {
                // Return to the main menu
                break;
            }
        }
    }
}