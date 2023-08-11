#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use web3::{
    transports::WebSocket,
    contract::{Contract, Options},
    types::{Address, U256}
};
use std::str::FromStr;
use arke_core::{UnlinkableHandshake, StoreKey,};
use crossterm::terminal;
use chrono::{Local, Timelike};
use textwrap::wrap;


#[derive(Clone)]
pub struct KeyValueStore(Contract<WebSocket>);

impl KeyValueStore {
    pub async fn new(web3: &web3::Web3<web3::transports::WebSocket>, contract_address: String) -> Self {
        let contract_address = Address::from_str(&contract_address).unwrap();
        let contract =
            Contract::from_json(web3.eth(), contract_address, include_bytes!("key_value_store.abi")).unwrap();
        KeyValueStore(contract)
    }


    /* Write */ 
    pub async fn Write(&self, cipher: Vec<u8>, iv: Vec<u8>, addr: Address, from: Address, id: String) {
        //println!("Write cipher: {:?}", cipher);
        // Call to create the transaction
        let tx = self
            .0
            .call(
                "Write",
                (cipher, iv, addr, id),
                from,
                Options {
                    gas: Some(5_000_000.into()),
                    ..Default::default()
                }
            )
            .await;
        match tx {
            //Ok(_) => println!("Write completed"),
            Ok(_) => {},
            Err(e) => {}, //println!("Failed to Write: {:?}", e),
        }
    }


    /* Read */
    pub async fn Read(&self, addr: Address, from: Address, key: Vec<u8>, tag: StoreKey) {
        // Call to create the transaction
        let read_result: Result<(Vec<u8>, Vec<u8>), web3::contract::Error> = self
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
        match read_result {
            Ok((cipher, iv)) => {
                //println!("Read cipher: {:?}", cipher);
                let recover_result = UnlinkableHandshake::decrypt_message(
                    &key,
                    &tag,
                    &iv,
                    &cipher,
                );
                match recover_result {
                    Ok(recovered_message) => {
                        //println!("Message: {:?}", recovered_message);   
                        let recovered_message_text = String::from_utf8(recovered_message.to_vec()).unwrap();   
                        //println!("{:?}", recovered_message_text); 
                        Self::print_chatbox(&recovered_message_text);
                        //tui::start_tui(recovered_message_text);
                    }
                    Err(e) => {
                        //eprintln!("Failed to decrypt: {}", e);
                        return;
                    }
                }   
            },
            Err(e) => {
                //eprintln!("Failed to show Read result: {}", e);
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
            //Ok(_) => println!("Read completed"),
            Ok(_) => {},
            Err(e) => {
                //eprintln!("Failed to Read: {:?}", e);
                return;
            }
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
            //Ok(_) => println!("✓ Delete completed"),
            Ok(_) => {},
            Err(_) => {
                //eprintln!("Failed to Delete: {:?}", e),
                return;
            },
        }
    }


    /* sendEther */
    pub async fn sendEther(&self, to: Address, amount: U256, from: Address) {
        // Call to create the transaction
        let tx = self
            .0
            .call(
                "sendEther",
                to,
                from,
                Options {
                    gas: Some(5_000_000.into()),
                    value: Some(amount),
                    ..Default::default()
                }
            )
            .await;
        match tx {
            Ok(_) => println!("✓ sendEther completed"),
            Err(e) => {
                eprintln!("Failed to sendEther: {:?}", e);
                return;
            },
        }
    }


    fn print_chatbox(message: &str) {
        let term_width = terminal::size().unwrap().0 as usize;
        // Deducting space for the border and padding
        let wrap_width = term_width/2;
        // Wrapping the text based on the terminal width
        let wrapped_text = wrap(message, wrap_width);
        // Finding the maximum line length after wrapping
        let max_length = wrapped_text.iter().map(|line| line.len()).max().unwrap_or(0);
        let padding = term_width - max_length - 6; 
        println!("\n{:width$}┌{}┐", "", "─".repeat(max_length + 2), width = padding);
        for line in wrapped_text {
            println!("{:width$}│ {}{} │", "", line, " ".repeat(max_length - line.len()), width = padding);
        }
        println!("{:width$}└{}┘", "", "─".repeat(max_length + 2), width = padding);
        let local_time = Local::now();
        let time_str = format!("{:02}:{:02}:{:02}  ▼", local_time.hour(), local_time.minute(), local_time.second());
        println!("{:width$}{}", "", time_str, width = term_width as usize - 13);
    }
}