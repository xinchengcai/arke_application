// ---------------------------------------
// File: create_group.rs
// Date: 11 Sept 2023
// Description: Group chat (client-side)
// ---------------------------------------
#![allow(unused_assignments)]
#![allow(dead_code)]

use web3::types::{Address, H160};
use std::str::FromStr;
use arke_core::{StoreKey, UserSecretKey, ThresholdObliviousIdNIKE};
use ark_bw6_761::BW6_761;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_ec::bls12::Bls12;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, Cursor};
use serde::{Serialize, Deserialize};
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use std::fs::File;
use crate::discovery_info::DiscoveryInfo;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
// Maximum number of dishonest participants that the system can tolerate
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
struct Group {
    id_string: String,
    store_addr: H160,
    own_write_tag: StoreKey,
    own_read_tag: StoreKey,
    symmetric_key: Vec<u8>,
    member_id_string: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id_string: String,
}

pub async fn createGroup() -> Result<(), Box<dyn std::error::Error>> {
    let want_create_group_id_string = dialoguer::Input::<String>::new()
        .with_prompt("What is the group name?")
        .interact()
        .unwrap();

    // Read groups.json
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/groups.json")
        .await?;
    // Derialize groups.json to read group objects 
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    // Convert each group to a string representation and collect them into a vector
    let groups: Vec<Group> = match serde_json::from_str(&contents) {
        Ok(groups) => groups,
        Err(_) => Vec::new(),
    };
    
    // Check whether the group the user want to create has already been created
    let group = groups.iter().find(|&c| c.id_string == want_create_group_id_string);
    match group {
        // If the group has already been created
        Some(group) => {
            println!("Group {:?} has already been created", group.id_string);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Group has already been created")));
        },
        // If the group has not been created
        None => {},
    };

    // Enter the member of the group
    let mut member_id_string: Vec<String> = Vec::new();
    loop {
        let input = dialoguer::Input::<String>::new()
            .with_prompt("Enter the member of the group (or 'done' to finish)")
            .interact_text()
            .unwrap();
        if input == "done" {
            break;
        }
        member_id_string.push(input);
    }


    // Read my_info.bin
    let mut my_info_file = File::open("src/my_info.bin")?;
    let mut deserialized: Vec<u8> = Vec::new();
    my_info_file.read_to_end(&mut deserialized)?;
    // Derialize my_info.bin to read my_info object
    let mut cursor = Cursor::new(&deserialized);
    let my_info = MyInfo::deserialize(&mut cursor)?;

    // Perform the rest part of id-nike (i.e. locally derive the shared seed) 
    // and the entire handshake (i.e. locally derive symmetric key from shared seed, locally derive write and read tag )
    let discovery = DiscoveryInfo::id_nike_and_handshake(my_info.id_string.clone(), 
                                    want_create_group_id_string.clone(), 
                                                my_info.sk.clone());
    let symmetric_key = discovery.symmetric_key;
    let own_write_tag = discovery.alice_write_tag.clone();
    let own_read_tag = discovery.alice_write_tag.clone();

    // Derive store address from the write tags
    let mut store_addr_string = String::new();
    // Ensure the user and the target user (in this case the group id) derive the same store address
    if my_info.id_string.clone() < want_create_group_id_string.clone() {
        store_addr_string = hex::encode(DiscoveryInfo::to_address(&own_write_tag));
    }
    else {
        store_addr_string = hex::encode(DiscoveryInfo::to_address(&own_read_tag));
    }
    let store_addr = Address::from_str(&store_addr_string).unwrap();

    // Create new group object
    let new_group = Group {
        id_string: want_create_group_id_string.clone(),
        store_addr: store_addr.clone(),
        own_write_tag: own_write_tag.clone(),
        own_read_tag: own_read_tag.clone(),
        symmetric_key: symmetric_key.clone(),
        member_id_string: member_id_string.clone(),
    };

    // Read then write to groups.json
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/groups.json")
        .await?;
    // Derialize groups.json to read group objects 
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    // Convert each group to a string representation and collect them into a vector
    let mut groups: Vec<Group> = match serde_json::from_str(&contents) {
        Ok(groups) => groups,
        Err(_) => Vec::new(),
    };
    // Append the new group to the vector
    groups.push(new_group);
    // Write groups back to the file
    let contacts_json = serde_json::to_string(&groups)?; 
    let mut file = File::create("src/groups.json")?;
    file.write_all(contacts_json.as_bytes())?;

    Ok(())
} 