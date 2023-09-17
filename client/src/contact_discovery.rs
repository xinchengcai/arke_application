// ---------------------------------------
// File: contact_discovery.rs
// Date: 11 Sept 2023
// Description: Contact discovery (client-side)
// ---------------------------------------
#![allow(unused_assignments)]
#![allow(dead_code)]

use web3::types::{Address, H160};
use std::str::FromStr;
use arke_core::{StoreKey, UserSecretKey, ThresholdObliviousIdNIKE};
use ark_bw6_761::BW6_761;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_ec::bls12::Bls12;
use crate::discovery_info::DiscoveryInfo;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, Cursor};
use serde::{Serialize, Deserialize};
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use std::fs::File;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
// Maximum number of dishonest pariticipants that the system can tolerate
const THRESHOLD: usize = 3;
// Domain identifier for the registration authority of this example
const REGISTRAR_DOMAIN: &'static [u8] = b"registration";

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
struct MyInfo {
    id_string: String,
    eth_addr: String,
    sk: UserSecretKey<Bls12<Parameters>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Friend {
    id_string: String,
    store_addr: H160,
    own_write_tag: StoreKey,
    own_read_tag: StoreKey,
    symmetric_key: Vec<u8>,
    eth_addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id_string: String,
}

pub async fn contactDiscovery() -> Result<(), Box<dyn std::error::Error>> {
    let want_contact_discovery_id_string = dialoguer::Input::<String>::new()
        .with_prompt("Which contact do you want to discover?")
        .interact()
        .unwrap();

    // Read friends.json
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/friends.json")
        .await?;
    // Derialize friends.json to read friend objects 
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    // Convert each friend to a string representation and collect them into a vector
    let friends: Vec<Friend> = match serde_json::from_str(&contents) {
        Ok(friends) => friends,
        Err(_) => Vec::new(),
    };
    
    // Check whether the target user is in user's friend list
    let friend = friends.iter().find(|&c| c.id_string == want_contact_discovery_id_string);
    match friend {
        // If the target user is in user's friend list already
        Some(friend) => {
            println!("You have already discovered {:?}", friend.id_string);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "User already in frined list")));
        },
        // If the target user is not in user's friend list
        None => {},
    };


    // Read my_info.bin
    let mut my_info_file = File::open("src/my_info.bin")?;
    let mut deserialized: Vec<u8> = Vec::new();
    my_info_file.read_to_end(&mut deserialized)?;
    // Derialize my_info.bin to read my_info object
    let mut cursor = Cursor::new(&deserialized);
    let my_info = MyInfo::deserialize(&mut cursor)?;

    // Perform the rest part of ID-NIKE (i.e. ID-NIKE.SharedKey) 
    // and the entire Handshake (i.e. Handshake.DeriveWrite and Handshake.DriveRead)
    let discovery = DiscoveryInfo::id_nike_and_handshake(my_info.id_string.clone(), 
                                    want_contact_discovery_id_string.clone(), 
                                                my_info.sk.clone());
    let symmetric_key = discovery.symmetric_key;
    let own_write_tag = discovery.alice_write_tag;
    let own_read_tag = discovery.alice_read_tag;

    // Derive store address from the write tags
    let mut store_addr_string = String::new();
    // Ensure the user and target user derive the same store address
    if my_info.id_string.clone() < want_contact_discovery_id_string.clone() {
        store_addr_string = hex::encode(DiscoveryInfo::to_address(&own_write_tag));
    }
    else {
        store_addr_string = hex::encode(DiscoveryInfo::to_address(&own_read_tag));
    }
    let store_addr = Address::from_str(&store_addr_string).unwrap();

    // Create new friend object
    let new_friend = Friend {
        id_string: want_contact_discovery_id_string.clone(),
        store_addr: store_addr.clone(),
        own_write_tag: own_write_tag.clone(),
        own_read_tag: own_read_tag.clone(),
        symmetric_key: symmetric_key.clone(),
        eth_addr: String::new(),
    };

    // Read then write to friends.json
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/friends.json")
        .await?;
    // Derialize friends.json to read friend objects 
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    // Convert each friend to a string representation and collect them into a vector
    let mut friends: Vec<Friend> = match serde_json::from_str(&contents) {
        Ok(friends) => friends,
        Err(_) => Vec::new(),
    };
    // Append the new friend to the vector
    friends.push(new_friend);
    // Write friends back to the file
    let contacts_json = serde_json::to_string(&friends)?; 
    let mut file = File::create("src/friends.json")?;
    file.write_all(contacts_json.as_bytes())?;

    Ok(())
} 