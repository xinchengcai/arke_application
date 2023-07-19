#![allow(dead_code)]
#![allow(non_snake_case)]

use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;
use crate::key_value_store_frontend::KeyValueStore;
use rand::thread_rng;
use arke_core::{UnlinkableHandshake, StoreKey};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, Cursor};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::fs::File;

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
    eth_addr: String
}

fn print_chatbox(message: &str) {
    let border = "-".repeat(message.len() + 4);  // "+4" to account for extra padding

    println!("{}", border);
    println!("| {} |", message);
    println!("{}", border);
}

pub async fn option1() {   
    // Setup the contract and an interface to access it's functionality 
    let transport = web3::transports::Http::new("HTTP://127.0.0.1:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let Store = KeyValueStore::new(
        &web3,
        // Update to match the deployed contract address on ganache
        "0xDD7FE36d9340b502F143a4B43663613b0b29cc1f".to_string(),
        ).await;     

    // Read contacts.json
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
        .map(|contact| { format!("ID string: {}     Nickname: {}", contact.id_string, contact.nickname)}).collect();
    // Add go back to the end of the vector
    ContactsMenu.push("Go back".to_string());

    // Display the vector as a menu
    loop {
        let ContactsMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Who would you like to contact?")
        .default(0)
        .items(&ContactsMenu[..])
        .interact()
        .unwrap();
        match ContactsMenuSelection {
            // If selected a contact
            index if index < contacts.len() => {
                let selected_contact = &contacts[index];
                let id_string = selected_contact.id_string.clone();
                let store_addr = selected_contact.store_addr.clone();
                let own_write_tag = selected_contact.own_write_tag.clone();
                let own_read_tag = selected_contact.own_read_tag.clone();
                let symmetric_key = selected_contact.symmetric_key.clone();

                /* Read */
                // After selecting a contact, first read the store to get the message sent by this contact to me
                // Read my_info.bin
                let mut my_info_file = File::open("src/my_info.bin").unwrap();
                let mut deserialized: Vec<u8> = Vec::new();
                my_info_file.read_to_end(&mut deserialized).unwrap();
                // Derialize my_info.bin to read my_info object
                let mut cursor = Cursor::new(&deserialized);
                let my_info = MyInfo::deserialize(&mut cursor).unwrap();
                // Read the store
                let reader_addr = Address::from_str(&my_info.eth_addr).unwrap();
                println!("{}", selected_contact.nickname);
                Store.Read(store_addr, reader_addr, symmetric_key.clone(), own_read_tag).await;     

                /* Write */
                // After reading the store, write the store to send the message to the selected contact
                let message = dialoguer::Input::<String>::new()
                    .with_prompt("What message do you want to send?")
                    .interact()
                    .unwrap();
                let mut rng = thread_rng();
                let (iv, cipher) =
                    UnlinkableHandshake::encrypt_message(&symmetric_key, &own_write_tag, message.as_bytes(), &mut rng).unwrap();
                // Write the store
                let writer_addr = Address::from_str(&my_info.eth_addr).unwrap();
                Store.Write(cipher, iv, store_addr, writer_addr, id_string).await;
                println!("{}", my_info.nickname);
                print_chatbox(&message);
            }

            _ => {
                // If selected go back, return to the main menu
                break;
            }
        }
    }
}