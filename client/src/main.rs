#![allow(non_snake_case)]
#![allow(unused_must_use)]

// Libs for ethereum contract 
mod key_value_store_frontend;

// Libs for arke
mod arke_frontend;

// Libs for UI
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
mod main_menu_option0;
use main_menu_option0::option0;
mod main_menu_option1;
use main_menu_option1::option1;
mod main_menu_option2;
use main_menu_option2::option2;
mod main_menu_option3;
use main_menu_option3::option3;



#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let MainMenu = &[
        "My info",
        "Contacts",
        "Contact Discovery",
        "Delete Contact",
        "Exit",
    ];

    loop {
        let MainMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&MainMenu[..])
            .interact()
            .unwrap();

        match MainMenuSelection {
            0 => {
                option0().await;
            }
            1 => {
                option1().await;
            }
            2 => {
                option2().await;
            }
            3 => {
                option3().await;
            }
            4 => {
                break;
            }
            _ => {
                println!("Invalid selection");
            }
        }
    } 

    Ok(())
}