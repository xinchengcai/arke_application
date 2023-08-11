#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use web3::types::Address;
use web3::types::{H160, U256};
use web3::futures::StreamExt;
use web3::types::{FilterBuilder, Log};
use std::str::FromStr;
use crate::key_value_store_frontend::KeyValueStore;
use rand::{distributions::Alphanumeric, Rng, thread_rng};
use arke_core::{UnlinkableHandshake, UserSecretKey, StoreKey};
use ark_ec::bls12::Bls12;
use ark_bls12_377::Parameters;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, Cursor};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::fs::File;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use serde_json::json;
use std::thread;
use std::time::Duration;
use tokio::sync::Mutex;
use crossterm::terminal;
use chrono::{Local, Timelike};
use textwrap::wrap;
use std::io::{Seek, SeekFrom};


#[derive(Clone, Serialize, Deserialize, Debug)]
struct Contact {
    id_string: String,
    store_addr: H160,
    own_write_tag: StoreKey,
    own_read_tag: StoreKey,
    symmetric_key: Vec<u8>,
    eth_addr: String,
}

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
struct MyInfo {
    id_string: String,
    eth_addr: String,
    sk: UserSecretKey<Bls12<Parameters>>,
}

fn print_chatbox(message: &str) {
    let term_width = terminal::size().unwrap().0 as usize;
    // Deducting space for the border and padding
    let wrap_width = term_width/2;
    // Wrapping the text based on the terminal width
    let wrapped_text = wrap(message, wrap_width);
    // Finding the maximum line length after wrapping
    let max_length = wrapped_text.iter().map(|line| line.len()).max().unwrap_or(0);
    println!("\n┌{}┐", "─".repeat(max_length + 2));
    for line in wrapped_text {
        println!("│ {}{} │", line, " ".repeat(max_length - line.len()));
    }
    println!("└{}┘", "─".repeat(max_length + 2));
    let local_time = Local::now();
    let time_str = format!("▼  {:02}:{:02}:{:02}", local_time.hour(), local_time.minute(), local_time.second());
    println!("{}", time_str);
}

pub async fn option1() -> Result<(), Box<dyn std::error::Error>>{   
    // Setup the contract and an interface to access it's functionality 
    let transport = web3::transports::WebSocket::new("ws://127.0.0.1:9545").await?;
    let web3 = web3::Web3::new(transport);
    let web3 = Arc::new(web3);
    let Store = KeyValueStore::new(
        &web3,
        // Update to match the deployed contract address on ganache
        "0x061d97dBFf19a8540090142781891CaC6B5Eb982".to_string(),
        ).await;     
    let Store = Arc::new(Store);

    
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
        return Ok(());
    }

    // Derialize contacts.json to read contact objects 
    let contacts: Vec<Contact> = serde_json::from_reader(file).unwrap();
    // Convert each contact to a string representation and collect them into a vector
    let mut ContactsMenu: Vec<String> = contacts.iter()
        .map(|contact| { format!("ID string: {}", contact.id_string)}).collect();
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
                let selected_contact = contacts[index].clone();
                let id_string = selected_contact.id_string.clone();
                let store_addr = selected_contact.store_addr.clone();
                let own_write_tag = selected_contact.own_write_tag.clone();
                let own_read_tag = selected_contact.own_read_tag.clone();
                let symmetric_key = selected_contact.symmetric_key.clone();
                let eth_addr = selected_contact.eth_addr.clone();

                let ContactActionMenu = &[
                    "Chat",
                    "Pay",
                    "Exit",
                ];
                let ContactActionMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("What would you like to do with this contact?")
                    .default(0)
                    .items(&ContactActionMenu[..])
                    .interact()
                    .unwrap();
                match ContactActionMenuSelection {
                    0 => {
                        let (tx, mut rx) = mpsc::channel(100);

                        tokio::spawn(async move {
                            loop {
                                let message = dialoguer::Input::<String>::new()
                                //.with_prompt("What message do you want to send? (type q to quit)")
                                .interact()
                                .unwrap();
                                if message == "q" {       
                                    break;
                                }
                                if tx.send(message).await.is_err() {
                                    break;
                                }
                            }
                        });
 
                        let Store_clone1 = Arc::clone(&Store);
                        let web3_clone = Arc::clone(&web3);
                        tokio::spawn(async move {
                            let filter = FilterBuilder::default()
                                .address(vec!["0x061d97dBFf19a8540090142781891CaC6B5Eb982".parse().unwrap()])
                                .build();
                            loop {
                                let filter_clone = filter.clone();
                                let mut my_info_file = File::open("src/my_info.bin").unwrap();
                                let mut deserialized: Vec<u8> = Vec::new();
                                my_info_file.read_to_end(&mut deserialized).unwrap();
                                let mut cursor = Cursor::new(&deserialized);
                                let my_info = MyInfo::deserialize(&mut cursor).unwrap();

                                match web3_clone.eth_subscribe().subscribe_logs(filter_clone).await {
                                    Ok(mut sub) => {
                                        // Process incoming events
                                        while let Some(Ok(_log)) = sub.next().await {
                                            //println!("Event triggered!");
                                            let log_data = &_log.data.0;
                                            let log_str = String::from_utf8_lossy(log_data);
                                            let log_id: String = log_str.chars()
                                                .filter(|&c| (c.is_ascii_graphic() || c == ' ') && c != '0')
                                                .collect::<String>()
                                                .trim_start()
                                                .to_string();                          
                                            if log_id == my_info.id_string {
                                                /* Read */
                                                let reader_addr = Address::from_str(&my_info.eth_addr).unwrap();
                                                let symmetric_key = selected_contact.symmetric_key.clone();
                                                Store_clone1.Read(store_addr, reader_addr, symmetric_key.clone(), own_read_tag.clone()).await;
                                                thread::sleep(Duration::from_secs(1));  // sleep for 1 seconds before the next Read                                          
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        // Handle the error, possibly by logging it and/or breaking out of the loop
                                        println!("Error subscribing to logs: {:?}", e);
                                        break;
                                    }
                                }
                            }
                        });

                        let Store_clone2 = Arc::clone(&Store);
                        while let Some(message) = rx.recv().await {
                            let mut my_info_file = File::open("src/my_info.bin").unwrap();
                            let mut deserialized: Vec<u8> = Vec::new();
                            my_info_file.read_to_end(&mut deserialized).unwrap();
                            let mut cursor = Cursor::new(&deserialized);
                            let my_info = MyInfo::deserialize(&mut cursor).unwrap();
                            /* Write */
                            let mut rng = thread_rng();
                            let (iv, cipher) =
                            UnlinkableHandshake::encrypt_message(&symmetric_key, &own_write_tag, message.as_bytes(), &mut rng).unwrap();
                            let writer_addr = Address::from_str(&my_info.eth_addr).unwrap();
                            Store_clone2.Write(cipher, iv, store_addr, writer_addr, id_string.clone()).await;
                            print_chatbox(&message);
                        }
                    }

                    1 => {
                        if selected_contact.eth_addr.to_string().len() == 0 {
                            let recepient_eth_addr = dialoguer::Input::<String>::new()
                                .with_prompt("What is the recipient ethereum address?")
                                .interact()
                                .unwrap();
                            let mut file = OpenOptions::new()
                                .read(true)
                                .write(true)
                                .open("src/contacts.json")
                                .unwrap();
                            // Derialize contacts.json to read contact objects 
                            let mut contacts: Vec<Contact> = serde_json::from_reader(&file).unwrap();
                            for contact in &mut contacts {
                                if contact.id_string == selected_contact.id_string {
                                    contact.eth_addr = recepient_eth_addr.clone();
                                    // Truncate the file and rewind to the beginning
                                    file.set_len(0).unwrap();
                                    file.seek(SeekFrom::Start(0)).unwrap();
                                    // Write the updated contacts back to the file
                                    serde_json::to_writer(&file, &contacts).unwrap();
                                    break;
                                }
                            }
                            let amount = dialoguer::Input::<String>::new()
                                .with_prompt("How much Ether do you want to send?")
                                .interact()
                                .unwrap();
                            let amount_in_ether: f64 = amount.parse().expect("Failed to parse user input");
                            // Convert the amount from Ether to wei
                            let amount_in_wei = U256::from_dec_str(&(amount_in_ether * 1e18).to_string()).expect("Failed to convert to wei");
                            let Store_clone = Arc::clone(&Store);

                            let mut my_info_file = File::open("src/my_info.bin").unwrap();
                            let mut deserialized: Vec<u8> = Vec::new();
                            my_info_file.read_to_end(&mut deserialized).unwrap();
                            let mut cursor = Cursor::new(&deserialized);
                            let my_info = MyInfo::deserialize(&mut cursor).unwrap();
                            let sender_addr = Address::from_str(&my_info.eth_addr).unwrap();
                            Store_clone.sendEther(Address::from_str(&recepient_eth_addr).unwrap(), amount_in_wei, sender_addr).await;
                            return Ok(());
                        }
                        else {
                            let amount = dialoguer::Input::<String>::new()
                                .with_prompt("How much Ether do you want to send?")
                                .interact()
                                .unwrap();
                            let amount_in_ether: f64 = amount.parse().expect("Failed to parse user input");
                            // Convert the amount from Ether to wei
                            let amount_in_wei = U256::from_dec_str(&(amount_in_ether * 1e18).to_string()).expect("Failed to convert to wei");
                            let Store_clone = Arc::clone(&Store);

                            let mut my_info_file = File::open("src/my_info.bin").unwrap();
                            let mut deserialized: Vec<u8> = Vec::new();
                            my_info_file.read_to_end(&mut deserialized).unwrap();
                            let mut cursor = Cursor::new(&deserialized);
                            let my_info = MyInfo::deserialize(&mut cursor).unwrap();
                            let sender_addr = Address::from_str(&my_info.eth_addr).unwrap();
                            Store_clone.sendEther(Address::from_str(&eth_addr).unwrap(), amount_in_wei, sender_addr).await;
                            return Ok(());
                        }
                    }

                    _ => {
                        return Ok(());
                    }
                }    
            }

            _ => {
                // If selected go back, return to the main menu
                return Ok(());
            }
        }
    }
}