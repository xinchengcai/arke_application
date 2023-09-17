#![allow(non_snake_case)]

use web3::types::H160;
use arke_core::{StoreKey, UserSecretKey};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use serde::{Serialize, Deserialize};
use std::fs::{OpenOptions, File};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read};
use ark_ec::bls12::Bls12;
use ark_bls12_377::Parameters;

#[derive(Serialize, Deserialize, Debug)]
struct Friend {
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

pub async fn deleteFriend() -> Result<(), Box<dyn std::error::Error>>{
    // Read friends.json
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("src/friends.json")
        .unwrap();
    // Check whether friends.json is empty or not, i.e. whether there are friends or not
    let metadata = file.metadata().unwrap();
    // If empty, return to the main menu
    if metadata.len() == 0 {
        println!("No friends");
        return Ok(());
    }

    // Derialize friends.json to read friend objects 
    let friends: Vec<Friend> = serde_json::from_reader(file).unwrap();
    // Convert each friend to a string representation and collect them into a vector
    let mut FriendsMenu: Vec<String> = friends.iter()
        .map(|contact| { format!("ID string: {}", contact.id_string)}).collect();
    // Add go back to the end of the vector
    FriendsMenu.push("Go back".to_string());

    // Display the vector as a menu
    loop {
        let FriendsMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Which friend would you like to delete?")
        .default(0)
        .items(&FriendsMenu[..])
        .interact()
        .unwrap();
        match FriendsMenuSelection {
            index if index < friends.len() => {
                // Delete in the saved friends
                // Read then write to friends.json
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("src/friends.json").unwrap();
                // Derialize friends.json to read friend objects 
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                // Convert each friend to a string representation and collect them into a vector
                let mut friends: Vec<Friend> = match serde_json::from_str(&contents) {
                    Ok(friends) => friends,
                    Err(_) => Vec::new(),
                };
                // remove the contact from the vector
                friends.remove(index);
                // Write friends back to the file
                let file = File::create("src/friends.json").unwrap();
                serde_json::to_writer(&file, &friends).unwrap();

                // Return to the main menu
                return Ok(());
            }

            // If selected go back
            _ => {
                // Return to the main menu
                return Ok(());
            }
        }
    }
}