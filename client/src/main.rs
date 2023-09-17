#![allow(non_snake_case)]
#![allow(unused_must_use)]

// Libs for ethereum contract 
mod key_value_store_frontend;

// Libs for arke
mod discovery_info;

// Libs for UI
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

mod user;
use user::user;
mod private_chat_and_pay;
use private_chat_and_pay::privateChatAndPay;
mod contact_discovery;
use contact_discovery::contactDiscovery;
mod delete_friend;
use delete_friend::deleteFriend;
mod group_chat;
use group_chat::groupChat;
mod create_group;
use create_group::createGroup;


#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let MainMenu = &[
        "My info",
        "Friends",
        "Contact Discovery",
        "Delete Friend",
        "Groups",
        "Start Group",
        "Exit",
    ];

    // Display the main menu
    loop {
        let MainMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&MainMenu[..])
            .interact()
            .unwrap();

        match MainMenuSelection {
            0 => {
                // Sign up and initialize my info if not signed up
                // Print my info if signed up
                user().await; 
            }
            1 => {
                // Print the friend list
                // Select a friend in the friend list to private chat or pay
                privateChatAndPay().await; 
            }
            2 => {
                // Perform contact discovery with a target user
                // Add the discovered user into the friend list
                contactDiscovery().await; 
            }
            3 => {
                // Delete a friend from the friend list
                deleteFriend().await; 
            }
            4 => {
                // Print the group list
                // Select a group in the group list to group chat
                groupChat().await;
            }
            5 => {
                // Perform contact discovery with a desired group ID to create the group
                // Add the created group into the group list
                createGroup().await;
            }
            6 => {
                // Exit the application
                break; 
            }
            _ => {
                println!("Invalid selection");
            }
        }
    } 

    Ok(())
}