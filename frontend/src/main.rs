// Libs for ethereum contract 
use web3::types::Address;
use std::str::FromStr;

// Libs for arke
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use arke_core::{random_id, UnlinkableHandshake};
const IDENTIFIER_STRING_LENGTH: usize = 8;
mod frontend;
use frontend::Arke;


#[tokio::main]
pub async fn main() {
    #![allow(non_snake_case)]

    /*  Setup the contract and an interface to access it's functionality */
    let transport = web3::transports::Http::new("HTTP://127.0.0.1:9545").unwrap();
    let web3 = web3::Web3::new(transport);
    let Store = frontend::KeyValueStore::new(
        &web3,
        // Update to match the deployed address
        "0xff9b37815B953374F1E6da8c0A22C9432fc2df8E".to_string(),
    )
    .await;


    /* Two users (Alice and Bob) run id-nike and handshake */
    let alice_id_string = random_id!(IDENTIFIER_STRING_LENGTH);
    let bob_id_string = random_id!(IDENTIFIER_STRING_LENGTH);

    let crypto = Arke::id_nike_and_handshake(alice_id_string, bob_id_string);
    let symmetric_key = crypto._symmetric_key;
    let alice_write_tag = crypto._alice_write_tag;
    let bob_read_tag = crypto._bob_read_tag;
    let alice_id_string = crypto._alice_id_string;
    

    /* Alice Write */ 
    // Alice encrypts message to get cipher        
    let message = b"This is a message";
    let mut rng = thread_rng();
    let (iv, cipher) =
    UnlinkableHandshake::encrypt_message(&symmetric_key, &alice_write_tag, message, &mut rng)
    .unwrap();

    // Alice derives the store address
    let write_addr_string = hex::encode(Arke::to_address(&alice_write_tag));
    let write_addr = Address::from_str(&write_addr_string).unwrap();
    // Assume Alice has the address 0xF0a16A9A70ddd46ab45ad029bFB749D5bA1a1E8a which has a memonic "abstract" in ganache
    let writer_addr = Address::from_str("0xF0a16A9A70ddd46ab45ad029bFB749D5bA1a1E8a").unwrap();
    let id = alice_id_string;
    println!("\nWriting");
    println!("Message: {:?}", message);
    Store.Write(cipher, write_addr, writer_addr, id).await;
    println!("At store address: {:?}", write_addr);


    /* Bob Read */
    // Bob derives the store address
    let read_addr_string = hex::encode(Arke::to_address(&bob_read_tag));
    let read_addr = Address::from_str(&read_addr_string).unwrap();
    // Assume Bob has the address 0x5fDd59bBE37d408317161076EDE1F84c2a055c84 which has a memonic "bundle" in ganache
    let reader_addr = Address::from_str("0x5fDd59bBE37d408317161076EDE1F84c2a055c84").unwrap();
    println!("\nReading");
    Store.Read(read_addr, reader_addr, symmetric_key, alice_write_tag, iv).await;
    println!("At store address: {:?}", read_addr);


    /* Alice Delete */ 
    //let delete_addr = write_addr;
    //Store.Delete(delete_addr, delete_addr).await;
}
