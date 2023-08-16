#![allow(unused_assignments)]
#![allow(dead_code)]

use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;
use arke_core::{StoreKey, UserSecretKey, ThresholdObliviousIdNIKE};
use ark_bw6_761::BW6_761;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_ec::bls12::Bls12;
use crate::arke_frontend::Arke;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, Cursor};
use serde::{Serialize, Deserialize};
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use std::fs::File;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
/// Maximum number of dishonest key-issuing authorities that the system can tolerate
const THRESHOLD: usize = 3;
/// Domain identifier for the registration authority of this example
const REGISTRAR_DOMAIN: &'static [u8] = b"registration";

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
struct MyInfo {
    id_string: String,
    eth_addr: String,
    sk: UserSecretKey<Bls12<Parameters>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Contact {
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

pub async fn option2() -> Result<(), Box<dyn std::error::Error>> {
    let want_contact_discovery_id_string = dialoguer::Input::<String>::new()
        .with_prompt("Who do you want to add to your contact book?")
        .interact()
        .unwrap();

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
    let contact = contacts.iter().find(|&c| c.id_string == want_contact_discovery_id_string);
    match contact {
        // If the person is in my contact book already
        Some(contact) => {
            println!("{:?} is already in your contacts", contact.id_string);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "User already in contacts")));
        },
        // If the person is not in my contact book
        None => {},
    };


    // Read my_info.bin
    let mut my_info_file = File::open("src/my_info.bin")?;
    let mut deserialized: Vec<u8> = Vec::new();
    my_info_file.read_to_end(&mut deserialized)?;
    // Derialize my_info.bin to read my_info object
    let mut cursor = Cursor::new(&deserialized);
    let my_info = MyInfo::deserialize(&mut cursor)?;

    // Perform the rest part of id-nike (i.e. locally derive the shared seed) 
    // and the entire handshake (i.e. locally derive symmetric key from shared seed, locally derive write and read tag )
    let crypto = Arke::id_nike_and_handshake(my_info.id_string.clone(), 
                                    want_contact_discovery_id_string.clone(), 
                                                my_info.sk.clone());
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
        id_string: want_contact_discovery_id_string.clone(),
        store_addr: store_addr.clone(),
        own_write_tag: own_write_tag.clone(),
        own_read_tag: own_read_tag.clone(),
        symmetric_key: symmetric_key.clone(),
        eth_addr: String::new(),
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