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
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use serde_json::json;
use std::thread;
use std::time::Duration;
use tokio::sync::Mutex;

//use crate::tui;

#[derive(Clone, Serialize, Deserialize, Debug)]
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
    let len = message.len();
    println!("\n");
    // Print top border
    println!(" {}", "━".repeat(len + 4));
    // Print message
    println!("/  {}  \\", message);
    // Print bottom border with tail
    println!(" {}", "━".repeat(len + 4));
    println!("▼");
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
                //tui::start_tui();
                let selected_contact = contacts[index].clone();
                let id_string = selected_contact.id_string.clone();
                let store_addr = selected_contact.store_addr.clone();
                let own_write_tag = selected_contact.own_write_tag.clone();
                let own_read_tag = selected_contact.own_read_tag.clone();
                let symmetric_key = selected_contact.symmetric_key.clone();

                // Start of your program
                let (tx, mut rx) = mpsc::channel(100);
                // At the start of the loop where you handle a selected contact...
                //let (mut read_stream, mut write_stream) = tokio::try_join!(
                 //   TcpStream::connect("127.0.0.1:8080"),
                //    TcpStream::connect("127.0.0.1:8080"),
                //)
                //.expect("Could not connect to server");
                let read_stream1 = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").await.expect("Could not connect to server")));
                let read_stream2 = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").await.expect("Could not connect to server")));
                let write_stream = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").await.expect("Could not connect to server")));
                

                tokio::spawn(async move {
                    loop {
                        let message = dialoguer::Input::<String>::new()
                            //.with_prompt("What message do you want to send? (type Esc to quit)")
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
                tokio::spawn(async move {
                    loop {
                        let mut my_info_file = File::open("src/my_info.bin").unwrap();
                        let mut deserialized: Vec<u8> = Vec::new();
                        my_info_file.read_to_end(&mut deserialized).unwrap();
                        let mut cursor = Cursor::new(&deserialized);
                        let my_info = MyInfo::deserialize(&mut cursor).unwrap();

                        // Create the request for read unread_flag
                        let request = json!({
                            "action": "unread_flag",
                            "id_string": my_info.id_string,
                            "rw": "r",
                        });
                        // Convert the request to a byte array
                        let request_bytes = serde_json::to_vec(&request).expect("Could not convert request");
                        // Lock the stream before using it
                        let mut locked_stream1 = read_stream1.lock().await;
                        // Write the request to the stream
                        locked_stream1.write_all(&request_bytes).await.expect("Could not write the stream");
                        // Create a buffer to read the response into
                        let mut buf = vec![0; 1024];
                        let n = locked_stream1.read(&mut buf).await.expect("Could not read the response");
                        let s = std::str::from_utf8(&buf[..n]).expect("Could not convert to string");
                        for line in s.split('\n') {
                            if !line.is_empty() {
                                // Parse the response
                                //println!("Server response: {}", s);
                                let response: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("Could not parse response");
                                if let Some(flag) = response.get("flag") {
                                /* Read */
                                    match flag.as_bool() {
                                        Some(true) => {
                                            let reader_addr = Address::from_str(&my_info.eth_addr).unwrap();
                                            let symmetric_key = selected_contact.symmetric_key.clone();
                                            Store_clone1.Read(store_addr, reader_addr, symmetric_key.clone(), own_read_tag.clone()).await;
                                            // Lock the stream before using it
                                            let mut locked_stream2 = read_stream2.lock().await;
                                            // Create the request for write unread_flag to false
                                            let request = json!({
                                                "action": "unread_flag",
                                                "id_string": my_info.id_string,
                                                "rw": "wf",
                                            });
                                            // Convert the request to a byte array
                                            let request_bytes = serde_json::to_vec(&request).expect("Could not convert request");
                                            // Write the request to the stream
                                            locked_stream2.write_all(&request_bytes).await.expect("Could not write the stream");
                                        },
                                        Some(false) => {},
                                        None => {}
                                    } 
                                }
                            }
                        }
                        thread::sleep(Duration::from_secs(1));  // sleep for 3 seconds before the next read
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
                    
                    // Create the request for write unread_flag to true
                    let request = json!({
                        "action": "unread_flag",
                        "id_string": selected_contact.id_string,
                        "rw": "wt",
                    });
                    // Convert the request to a byte array
                    let request_bytes = serde_json::to_vec(&request).expect("Could not convert request");
                    // Lock the stream before using it
                    let mut locked_stream = write_stream.lock().await;
                    // Write the request to the stream
                    locked_stream.write_all(&request_bytes).await.expect("Could not write the stream");
                    //println!("{}", my_info.nickname);
                    print_chatbox(&message);
                }
                    
            }

            _ => {
                // If selected go back, return to the main menu
                break;
            }
        }
    }
}