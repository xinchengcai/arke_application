// ---------------------------------------
// File: group_chat.rs
// Date: 11 Sept 2023
// Description: Group chat (client-side)
// ---------------------------------------
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(unused_assignments)]

use web3::types::{Address, H160, U256, FilterBuilder, Log};
use web3::futures::StreamExt;
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
use std::fs::{OpenOptions, File};
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
use tokio::task::JoinHandle;
use std::sync::atomic::{AtomicBool, Ordering};

const CONTRACT_ADDR: &str = "0xc23EDB04DebB123CDB1ac96a28eA18E8403a34d6";

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Group {
    id_string: String,
    store_addr: H160,
    own_write_tag: StoreKey,
    own_read_tag: StoreKey,
    symmetric_key: Vec<u8>,
    member_id_string: Vec<String>,
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

pub async fn groupChat() -> Result<(), Box<dyn std::error::Error>>{   
    // Setup the contract and an interface to access it's functionality 
    let transport = web3::transports::WebSocket::new("ws://127.0.0.1:9545").await?;
    let web3 = web3::Web3::new(transport);
    let web3 = Arc::new(web3);
    let Store = KeyValueStore::new(
        &web3,
        // Update to match the deployed contract address on ganache
        CONTRACT_ADDR.to_string(),
        ).await;     
    let Store = Arc::new(Store);

    // Read groups.json
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/groups.json")
        .unwrap();
    // Check whether groups.json is empty or not, i.e. whether there are groups or not
    let metadata = file.metadata().unwrap();
    // If empty, return to the main menu
    if metadata.len() == 0 {
        println!("No groups");
        return Ok(());
    }

    // Derialize groups.json to read group objects 
    let groups: Vec<Group> = serde_json::from_reader(file).unwrap();
    // Convert each group to a string representation and collect them into a vector
    let mut GroupsMenu: Vec<String> = groups.iter()
        .map(|group| { format!("Group ID string: {}", group.id_string)}).collect();
    // Add go back to the end of the vector
    GroupsMenu.push("Go back".to_string());

    // Display the vector as a menu
    loop {
        let mut handle1: Option<JoinHandle<()>> = None;
        let mut handle2: Option<JoinHandle<()>> = None;
        let should_terminate = Arc::new(AtomicBool::new(false));

        let GroupsMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Which group would you like to contact?")
            .default(0)
            .items(&GroupsMenu[..])
            .interact()
            .unwrap();
        match GroupsMenuSelection {
            // If selected a group
            index if index < groups.len() => {
                let selected_group = groups[index].clone();
                let id_string = selected_group.id_string.clone();
                let store_addr = selected_group.store_addr.clone();
                let own_write_tag = selected_group.own_write_tag.clone();
                let own_read_tag = selected_group.own_read_tag.clone();
                let symmetric_key = selected_group.symmetric_key.clone();
                let member_id_string = selected_group.member_id_string.clone();

                let GroupActionMenu = &[
                    "Chat",
                    "Exit",
                ];
                let GroupActionMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("What would you like to do with this group?")
                    .default(0)
                    .items(&GroupActionMenu[..])
                    .interact()
                    .unwrap();
                match GroupActionMenuSelection {
                    0 => {
                        let (tx, mut rx) = mpsc::channel(100);
                        let should_terminate_clone1 = Arc::clone(&should_terminate);
                        handle1 = Some(tokio::spawn(async move {
                            loop {
                                if should_terminate_clone1.load(Ordering::Relaxed) {
                                    //println!("terminated handle1");
                                    break;
                                }

                                let message = dialoguer::Input::<String>::new()
                                //.with_prompt("What message do you want to send? (type q to quit)")
                                .interact()
                                .unwrap();

                                if message == "q" {
                                    should_terminate_clone1.store(true, Ordering::Relaxed);
                                }

                                if tx.send(message).await.is_err() {
                                    break;
                                }
                            }
                        }));
 
                        let Store_clone1 = Arc::clone(&Store);
                        let web3_clone = Arc::clone(&web3);
                        let should_terminate_clone2 = Arc::clone(&should_terminate);
                        handle2 = Some(tokio::spawn(async move {
                            let filter = FilterBuilder::default()
                                .address(vec![CONTRACT_ADDR.parse().unwrap()])
                                .build();
                            loop {
                                if should_terminate_clone2.load(Ordering::Relaxed) {
                                    //println!("terminated handle2");
                                    break;
                                }
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
                                            if should_terminate_clone2.load(Ordering::Relaxed) {
                                                //println!("terminated handle2");
                                                break;
                                            }
                                            //println!("Event triggered!");
                                            let log_data = &_log.data.0;
                                            let log_str = String::from_utf8_lossy(log_data);
                                            let log_id: String = log_str.chars()
                                                .filter(|&c| (c.is_ascii_graphic() || c == ' ') && c != '0')
                                                .collect::<String>()
                                                .trim_start()
                                                .to_string();                       
                                            if log_id.contains(&my_info.id_string) {
                                                // Make Read transaction
                                                let reader_addr = Address::from_str(&my_info.eth_addr).unwrap();
                                                let symmetric_key = selected_group.symmetric_key.clone();
                                                Store_clone1.Read(store_addr, reader_addr, symmetric_key.clone(), own_read_tag.clone()).await;
                                                // Make Delete transaction
                                                let deleter_addr = Address::from_str(&my_info.eth_addr).unwrap();
                                                Store_clone1.Delete(store_addr, deleter_addr).await;
                                                thread::sleep(Duration::from_secs(1));  // sleep for 1 seconds before the next Read                                          
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        println!("Error subscribing to logs: {:?}", e);
                                        break;
                                    }
                                }
                            }
                        }));

                        let Store_clone2 = Arc::clone(&Store);
                        while let Some(message) = rx.recv().await {
                            if message == "q" {
                                should_terminate.store(true, Ordering::Relaxed);
                                if let Some(handle) = handle1.take() { 
                                    drop(handle);
                                }
                            
                                if let Some(handle) = handle2.take() {
                                    drop(handle);
                                }
                                //println!("dropped handles");
                            }
                            else {
                                let mut my_info_file = File::open("src/my_info.bin").unwrap();
                                let mut deserialized: Vec<u8> = Vec::new();
                                my_info_file.read_to_end(&mut deserialized).unwrap();
                                let mut cursor = Cursor::new(&deserialized);
                                let my_info = MyInfo::deserialize(&mut cursor).unwrap();
                                // Make Write transaction
                                let mut rng = thread_rng();
                                let (iv, cipher) =
                                UnlinkableHandshake::encrypt_message(&symmetric_key, &own_write_tag, message.as_bytes(), &mut rng).unwrap();
                                let writer_addr = Address::from_str(&my_info.eth_addr).unwrap();
                                Store_clone2.Write(cipher, iv, store_addr, writer_addr, member_id_string.clone()).await;
                                print_chatbox(&message);
                            }
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