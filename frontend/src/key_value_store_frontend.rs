use web3::{
    transports::Http,
    contract::{Contract, Options},
    types::Address
};
use std::str::FromStr;

use arke_core::{UnlinkableHandshake, StoreKey,};

// New type to better manage contract function handling.
pub struct KeyValueStore(Contract<Http>);

impl KeyValueStore {
    #![allow(non_snake_case)]
    #![allow(unused_variables)]
    #![allow(dead_code)]

    pub async fn new(web3: &web3::Web3<web3::transports::Http>, contract_address: String) -> Self {
        let contract_address = Address::from_str(&contract_address).unwrap();
        let contract =
            Contract::from_json(web3.eth(), contract_address, include_bytes!("key_value_store.abi")).unwrap();
        KeyValueStore(contract)
    }


    /* Write */ 
    pub async fn Write(&self, cipher: Vec<u8>, addr: Address, from: Address, id: String) {
        println!("Write cipher: {:?}", cipher);
        // Call to create the transaction
        let tx = self
            .0
            .call(
                "Write",
                (cipher, addr, id),
                from,
                Options {
                    gas: Some(5_000_000.into()),
                    ..Default::default()
                }
            )
            .await;
        match tx {
            Ok(_) => println!("Write completed"),
            Err(e) => eprintln!("Failed to Write: {:?}", e),
        }
    }


    /* Read */
    pub async fn Read(&self, addr: Address, from: Address, key: Vec<u8>, tag: StoreKey, iv: Vec<u8>) {
        // Call to create the transaction
        let result: Result<Vec<u8>, web3::contract::Error> = self
            .0
            .query(
                "Read",
                addr,
                from,
                Options {
                    gas: Some(5_000_000.into()),
                    ..Default::default()
                },
                None
            ).await;
        match result {
            Ok(cipher) => {
                println!("Read cipher: {:?}", cipher);
                let recovered_message = UnlinkableHandshake::decrypt_message(
                    &key,
                    &tag,
                    &iv,
                    &cipher,
                )
                .unwrap();   
                println!("Message: {:?}", recovered_message);   
                let recovered_message_text = String::from_utf8(recovered_message.to_vec()).unwrap();   
                println!("Message in text: {:?}", recovered_message_text); 
            },
            Err(e) => {
                println!("Failed to show Read result: {}", e);
                return;
            }
        }
        let tx = self
            .0
            .call(
                "Read",
                addr,
                from,
                Options {
                    gas: Some(5_000_000.into()),
                    ..Default::default()
                }
            )
            .await;
        match tx {
            Ok(tx_hash) => println!("Read completed"),
            Err(e) => eprintln!("Failed to Read: {:?}", e),
        }
    }


    /* Delete */
    pub async fn Delete(&self, addr: Address, from: Address) {
        // Call to create the transaction
        let tx = self
            .0
            .call(
                "Delete",
                addr,
                from,
                Options {
                    gas: Some(5_000_000.into()),
                    ..Default::default()
                }
            )
            .await;
        match tx {
            Ok(_) => println!("Delete completed"),
            Err(e) => eprintln!("Failed to Delete: {:?}", e),
        }
    }
}