#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use web3::types::Address;
use web3::types::{H160, U256};
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

pub async fn option1() {   
    // Setup the contract and an interface to access it's functionality 
    let transport = web3::transports::Http::new("HTTP://127.0.0.1:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let Store = KeyValueStore::new(
        &web3,
        // Update to match the deployed contract address on ganache
        "0xaa9DA43992664c44A2d46ccEd7c14a1CBf805177".to_string(),
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
                        // update session
                        let mut my_info_file = File::open("src/my_info.bin").unwrap();
                        let mut deserialized: Vec<u8> = Vec::new();
                        my_info_file.read_to_end(&mut deserialized).unwrap();
                        let mut cursor = Cursor::new(&deserialized);
                        let my_info = MyInfo::deserialize(&mut cursor).unwrap();

                        let session_token: String = rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(30) 
                            .map(char::from)
                            .collect();

                        println!("About to connect to the server for estabilishing the chatting session ...");
                        let mut stream = TcpStream::connect("127.0.0.1:8080").await.expect("Could not connect to server");
                        println!("Successfully connected to the server for estabilishing the chatting session.");
                        // Create the request for update_session
                        let request = json!({
                            "action": "update_session",
                            "id_string": my_info.id_string.clone(),
                            "session":session_token.clone(),
                        });
                        // Convert the request to a byte array
                        let request_bytes = serde_json::to_vec(&request).expect("Could not convert request");
                        // Write the request to the stream
                        stream.write_all(&request_bytes).await.expect("Could not write the stream");

                        let (tx, mut rx) = mpsc::channel(100);
                        let read_stream1 = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").await.expect("Could not connect to server")));
                        let read_stream2 = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").await.expect("Could not connect to server")));
                        let write_stream = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").await.expect("Could not connect to server")));
                
                        tokio::spawn(async move {
                            loop {
                                let message = dialoguer::Input::<String>::new()
                                //.with_prompt("What message do you want to send? (type q to quit)")
                                .interact()
                                .unwrap();
                                if message == "q" {
                                    println!("About to connect to the server for removing chatting session...");
                                    let mut stream = TcpStream::connect("127.0.0.1:8080").await.expect("Could not connect to server");
                                    println!("Successfully connected to the server for removing chatting session.");
                                    // Create the request for update_session
                                    let request = json!({
                                        "action": "update_session",
                                        "id_string": my_info.id_string.clone(),
                                        "session": String::new(),
                                    });
                                    // Convert the request to a byte array
                                    let request_bytes = serde_json::to_vec(&request).expect("Could not convert request");
                                    // Write the request to the stream
                                    stream.write_all(&request_bytes).await.expect("Could not write the stream");            
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
                                    "session": session_token.clone(),
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
                                                        "session": String::new(),
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
                                thread::sleep(Duration::from_secs(1));  // sleep for 1 seconds before the next Read
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
                                "session": String::new(),
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
                            break;
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
                            break;
                        }
                    }

                    _ => {
                        break;
                    }
                }    
            }

            _ => {
                // If selected go back, return to the main menu
                break;
            }
        }
    }
}