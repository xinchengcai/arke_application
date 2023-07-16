// Libs for ethereum contract 
use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;
use crate::key_value_store_frontend::KeyValueStore;

// Libs for arke
use arke_core::StoreKey;

// Libs for UI
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use serde::{Serialize, Deserialize};
use std::fs::{OpenOptions, File};
use std::io::Read;

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

pub async fn option3() {
    #![allow(non_snake_case)]
    
    /*  Setup the contract and an interface to access it's functionality */
    let transport = web3::transports::Http::new("HTTP://127.0.0.1:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let Store = KeyValueStore::new(
        &web3,
        // Update to match the deployed address
        "0xff9b37815B953374F1E6da8c0A22C9432fc2df8E".to_string(),
        )
        .await;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/contacts.json")
        .unwrap();
    let metadata = file.metadata().unwrap();
    if metadata.len() == 0 {
        println!("No contacts");
        return;
    }

    let contacts: Vec<Contact> = serde_json::from_reader(file).unwrap();
    // Convert each Contact to a string representation and collect them into a vector
    let mut ContactsMenu: Vec<String> = contacts.iter()
        .map(|contact| { format!("ID: {}     nickname: {}", contact.id, contact.nickname)}).collect();
    ContactsMenu.push("Go back".to_string());

    loop {
        let ContactsMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Which contact would you like to delete?")
        .default(0)
        .items(&ContactsMenu[..])
        .interact()
        .unwrap();
        match ContactsMenuSelection {
            index if index < contacts.len() => {
                // Here, use the index to get the corresponding contact and perform your operations
                let selected_contact = &contacts[index];
                // Your operations on selected_contact here
                let store_addr = selected_contact.store_addr.clone();
                // Assume Alice has the address 0xF0a16A9A70ddd46ab45ad029bFB749D5bA1a1E8a which has a memonic "abstract" in ganache
                let deleter_addr = Address::from_str("0xF0a16A9A70ddd46ab45ad029bFB749D5bA1a1E8a").unwrap();
                // Delete on the map
                Store.Delete(store_addr, deleter_addr).await;

                // Delete in the saved contacts
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
                // remove the contact
                contacts.remove(index);
                // Write contacts back to the file
                let file = File::create("src/contacts.json").unwrap();
                serde_json::to_writer(&file, &contacts).unwrap();

                break;
            }

            _ => {
                break;
            }
        }
    }
}