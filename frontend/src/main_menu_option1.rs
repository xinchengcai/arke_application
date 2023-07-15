// Libs for ethereum contract 
use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;
use crate::key_value_store_frontend::KeyValueStore;

// Libs for arke
use rand::thread_rng;
use arke_core::{UnlinkableHandshake, StoreKey};

// Libs for UI
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;


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


pub async fn option1() {
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
        .map(|contact| { format!("ID: {}     Nickname: {}", contact.id, contact.nickname)}).collect();
    ContactsMenu.push("Go back".to_string());

    loop {
        let ContactsMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Who would you like to contact?")
        .default(0)
        .items(&ContactsMenu[..])
        .interact()
        .unwrap();
        match ContactsMenuSelection {
            index if index < contacts.len() => {
                // Here, use the index to get the corresponding contact and perform your operations
                let selected_contact = &contacts[index];
                // Your operations on selected_contact here
                let id = selected_contact.id.clone();
                let store_addr = selected_contact.store_addr.clone();
                let own_write_tag = selected_contact.own_write_tag.clone();
                let contact_read_tag = selected_contact.contact_read_tag.clone();
                let symmetric_key = selected_contact.symmetric_key.clone();



                let message = dialoguer::Input::<String>::new()
                    .with_prompt("What message do you want to send?")
                    .interact()
                    .unwrap();
                let mut rng = thread_rng();
                let (iv, cipher) =
                    UnlinkableHandshake::encrypt_message(&symmetric_key, &own_write_tag, message.as_bytes(), &mut rng).unwrap();

                /* Write */
                // Assume Alice has the address 0xF0a16A9A70ddd46ab45ad029bFB749D5bA1a1E8a which has a memonic "abstract" in ganache
                let writer_addr = Address::from_str("0xF0a16A9A70ddd46ab45ad029bFB749D5bA1a1E8a").unwrap();
                println!("\nWriting");
                println!("Message: {:?}", message);
                Store.Write(cipher, store_addr, writer_addr, id).await;
                println!("At store address: {:?}", store_addr);

                /* Read */
                // Assume Bob has the address 0x5fDd59bBE37d408317161076EDE1F84c2a055c84 which has a memonic "bundle" in ganache
                let reader_addr = Address::from_str("0x5fDd59bBE37d408317161076EDE1F84c2a055c84").unwrap();
                println!("\nReading");
                Store.Read(store_addr, reader_addr, symmetric_key, contact_read_tag, iv).await;
                println!("At store address: {:?}", store_addr);
            }

            _ => {
                break;
            }
        }
    }
}