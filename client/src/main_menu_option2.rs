#![allow(unused_assignments)]
#![allow(dead_code)]

use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;
use arke_core::{StoreKey, UserSecretKey,};
use ark_bls12_377:: Parameters;
use ark_ec::bls12::Bls12;
use crate::arke_frontend::Arke;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, Cursor};
use serde::{Serialize, Deserialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::fs::File;

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
struct MyInfo {
    nickname: String,
    id_string: String,
    eth_addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Contact {
    nickname: String,
    id_string: String,
    store_addr: H160,
    own_write_tag: StoreKey,
    own_read_tag: StoreKey,
    symmetric_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    nickname: String,
    id_string: String,
    eth_addr: String,
    finding: String,
    key_id: String,
    unread: bool,
}

pub async fn option2() -> Result<(), Box<dyn std::error::Error>> {
    let want_contact_discovery_nickname = dialoguer::Input::<String>::new()
        .with_prompt("Who do you want to make contact discovery?")
        .interact()
        .unwrap();
    let mut want_contact_discovery_id_string = String::new();

    // Read contacts.json
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/contacts.json")
        .await?;
    // Derialize contacts.json to read contact objects 
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    // Convert each contact to a string representation and collect them into a vector
    let contacts: Vec<Contact> = match serde_json::from_str(&contents) {
        Ok(contacts) => contacts,
        Err(_) => Vec::new(),
    };
    
    // Check whether the person I want to make contact discovery is in my contact book
    let contact = contacts.iter().find(|&c| c.nickname == want_contact_discovery_nickname);
    match contact {
        // If the person is in my contact book already
        Some(contact) => {
            println!("{:?} is already in your contacts", contact.nickname);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "User already in contacts")));
        },
        // If the person is not in my contact book
        None => {
            // Connect to the server
            println!("About to connect to the server...");
            let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
            println!("Successfully connected to the server.");
            // Create the request for find_user, i.e. check whether the person is a user or not
            let request = json!({
                "action": "find_user",
                "nickname": want_contact_discovery_nickname,
            });
            // Convert the request to a byte array
            let request_bytes = serde_json::to_vec(&request)?;
            // Write the request to the stream
            stream.write_all(&request_bytes).await?;
            // Create a buffer to read the response into
            let mut buf = vec![0; 1024];
            let n = stream.read(&mut buf).await?;
            // Parse the response
            let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;
            // Print the response
            println!("Response: {}", response);
            // Process the response based on the status field
            if let Some(status) = response.get("status") {
                match status.as_str() {
                    // If the person is a user
                    Some("success") => {
                        println!("User found");
                        if let Some(id_string) = response.get("id_string") {
                            // Get the id_string of the user
                            want_contact_discovery_id_string = id_string.as_str().unwrap().to_string();
                            // Close the stream
                            drop(stream);  
                        }
                    },
                    // If the person is not a user
                    Some("error") => {
                        if let Some(message) = response.get("message") {
                            println!("Error: {}", message.as_str().unwrap());
                        }   
                        // Close the stream
                        drop(stream);  
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "User not found")));
                    },
                    // If the server failed to respond
                    _ => {
                        println!("Invalid response from server");
                        // Close the stream
                        drop(stream);  
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid response from server")));
                    },
                }
            }
        },
    };


    // Read my_info.bin
    let mut my_info_file = File::open("src/my_info.bin")?;
    let mut deserialized: Vec<u8> = Vec::new();
    my_info_file.read_to_end(&mut deserialized)?;
    // Derialize my_info.bin to read my_info object
    let mut cursor = Cursor::new(&deserialized);
    let my_info = MyInfo::deserialize(&mut cursor)?;

    println!("About to connect to the server...");
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Successfully connected to the server.");
    // Create the request for compute_sks, 
    // i.e. compute the user secret keys for me and the user who I want to make contact discovery
    let request = json!({
        "action": "compute_sks",
        "alice_id_string": my_info.id_string,
        "bob_id_string": want_contact_discovery_id_string,
    });
    // Convert the request to a byte array
    let request_bytes = serde_json::to_vec(&request)?;
    // Write the request to the stream
    stream.write_all(&request_bytes).await?;
    // Create a buffer to read the response into
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    // Parse the response
    let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;
    // Process the response based on the status field
    let mut key_id: Option<String> = None;
    if let Some(status) = response.get("status") {
        match status.as_str() {
            // If secret keys are generated
            Some("success") => {
                // Get the key_id for retrieve my secret key from the hash map
                if let Some(key_id_value) = response.get("key_id") {
                    key_id = Some(key_id_value.as_str().unwrap().to_string());
                } 
            },
            // If secret keys are not generated
            Some("error") => {
                if let Some(message) = response.get("message") {
                    println!("Error: {}", message.as_str().unwrap());
                }
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Can not get key id")));
            },
            // If the server failed to respond
            _ => {
                println!("Invalid response from server");
            }
        }
    }
    // Close the stream
    drop(stream);


    // Connect to the server
    println!("About to connect to the server...");
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Successfully connected to the server.");
    // Create the request for retrieve_sks, 
    // i.e. Retrieve my secret key from the hash map
    let request = json!({
        "action": "retrieve_sks",
        "key_id": key_id,
        "id_string": my_info.id_string,
        "want_id_string": want_contact_discovery_id_string,
    });
    // Convert the request to a byte array
    let request_bytes = serde_json::to_vec(&request)?;
    // Write the request to the stream
    stream.write_all(&request_bytes).await?;
    // Create a buffer to read the response into
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    // Parse the response
    let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;
    // Initialize sk_base64 as None
    let mut sk_base64: Option<String> = None;
    // Process the response based on the status field
    if let Some(status) = response.get("status") {
        match status.as_str() {
            Some("success") => {
                if let Some(sk_value) = response.get("sk") {
                    sk_base64 = Some(sk_value.as_str().unwrap().to_string());
                } 
            },
            Some("error") => {
                if let Some(message) = response.get("message") {
                    println!("Error: {}", message.as_str().unwrap());
                }
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Can not get sk")));
            },
            _ => {
                println!("Invalid response from server");
            }
        }
    }
    // Print the my secret key which has been encoded in base64
    println!("sk_base64: {:?}", sk_base64);

    // Recover my secret key
    let mut sk: Option<UserSecretKey<Bls12<Parameters>>> = None;
    if let Some(sk_base64) = sk_base64 {
        // Decode my secret key from base64
        let sk_bytes = base64::decode(&sk_base64).unwrap();
        // CanonicalDeserialize my decoded secret key
        let mut sk_cursor = Cursor::new(&sk_bytes);
        sk = Some(UserSecretKey::<Bls12<Parameters>>::deserialize(&mut sk_cursor).unwrap());
    } 
    else {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "sk_base64 is None")));
    }
    let sk = match sk {
        Some(sk) => sk,
        None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "sk is None"))),
    };

    // Perform the rest part of id-nike (i.e. locally derive the shared seed) 
    // and the entire handshake (i.e. locally derive symmetric key from shared seed, locally derive write and read tag )
    let crypto = Arke::id_nike_and_handshake(my_info.id_string.clone(), 
                                    want_contact_discovery_id_string.clone(), 
                                                sk.clone());
    let symmetric_key = crypto.symmetric_key;
    let own_write_tag = crypto.alice_write_tag;
    let own_read_tag = crypto.alice_read_tag;

    // Derive store address from the write tags
    let mut store_addr_string = String::new();
    // Ensure I and my contact derive the same store address
    if my_info.id_string.clone() < want_contact_discovery_id_string.clone() {
        store_addr_string = hex::encode(Arke::to_address(&own_write_tag));
    }
    else {
        store_addr_string = hex::encode(Arke::to_address(&own_read_tag));
    }
    let store_addr = Address::from_str(&store_addr_string).unwrap();

    // Create new contact object
    let new_contact = Contact {
        nickname: want_contact_discovery_nickname.clone(), 
        id_string: want_contact_discovery_id_string.clone(),
        store_addr: store_addr.clone(),
        own_write_tag: own_write_tag.clone(),
        own_read_tag: own_read_tag.clone(),
        symmetric_key: symmetric_key.clone(),
    };

    // Read then write to my_contact.json
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/contacts.json")
        .await?;
    // Derialize contacts.json to read contact objects 
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    // Convert each contact to a string representation and collect them into a vector
    let mut contacts: Vec<Contact> = match serde_json::from_str(&contents) {
        Ok(contacts) => contacts,
        Err(_) => Vec::new(),
    };
    // Append the new contact to the vector
    contacts.push(new_contact);
    // Write contacts back to the file
    let contacts_json = serde_json::to_string(&contacts)?; 
    let mut file = File::create("src/contacts.json")?;
    file.write_all(contacts_json.as_bytes())?;

    Ok(())
} 