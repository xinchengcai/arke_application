#![allow(non_snake_case)]

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
                option0();
            }
            1 => {
                option1().await;
            }
            2 => {
                option2();
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








    /*loop {
        let MainMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&MainMenu[..])
            .interact()
            .unwrap();

        match MainMenuSelection {
            0 => {
                let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open("src/my_info.json")?;
            
                let metadata = file.metadata()?;
                if metadata.len() != 0 {
                    let my_info: MyInfo = serde_json::from_reader(file)?;
                    println!("ID: {}", my_info.id);
                }
                else {
                    let id = random_id!(IDENTIFIER_STRING_LENGTH);
                    let my_info = json!({
                        "id": id,
                    });
                    // Convert to a JSON string
                    let data_string = my_info.to_string();
                    // Write to the file
                    file.write_all(data_string.as_bytes())?;
                    println!("ID: {}", id);
                }
            }

            1 => {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("src/contacts.json")?;
                let contacts: Vec<Contact> = serde_json::from_reader(file)?;
        
                // Convert each Contact to a string representation and collect them into a vector
                let ContactsMenu: Vec<String> = contacts.iter().map(|contact| { format!("ID: {}",contact.id)}).collect();
                let ContactsMenuSelection = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Who would you like to contact?")
                    .default(0)
                    .items(&ContactsMenu[..])
                    .interact()
                    .unwrap();
                    match ContactsMenuSelection {
                        index => {
                            // Here, use the index to get the corresponding contact and perform your operations
                            let selected_contact = &contacts[index];
                            // Your operations on selected_contact here
                            let message = dialoguer::Input::<String>::new()
                                .with_prompt("What message do you want to send?")
                                .interact()
                                .unwrap();

                            let id = selected_contact.id.clone();
                            let store_addr = selected_contact.store_addr.clone();
                            let write_tag = selected_contact.write_tag.clone();
                            let read_tag = selected_contact.read_tag.clone();
                            let symmetric_key = selected_contact.symmetric_key.clone();
    
                            let mut rng = thread_rng();
                            let (iv, cipher) =
                            UnlinkableHandshake::encrypt_message(&symmetric_key, &write_tag, message.as_bytes(), &mut rng)
                            .unwrap();
    
                            /* Write */
                            // Assume Alice has the address 0xF0a16A9A70ddd46ab45ad029bFB749D5bA1a1E8a which has a memonic "abstract" in ganache
                            let writer_addr = Address::from_str("0xF0a16A9A70ddd46ab45ad029bFB749D5bA1a1E8a").unwrap();
                            println!("\nWriting");
                            println!("Message: {:?}", message);
                            Store.Write(cipher, store_addr, writer_addr, id).await;
                            println!("At store address: {:?}", store_addr);
    
                            /* Read */
                            // Assume Bob has the address 0x5fDd59bBE37d408317161076EDE1F84c2a055c84 which has a memonic "bundle" in ganache
                            let reader_addr = Address::from_str("0x5fDd59bBE37d408317161076EDE1F84c2a055c84").unwrap();
                            println!("\nReading");
                            Store.Read(store_addr, reader_addr, symmetric_key, read_tag, iv).await;
                            println!("At store address: {:?}", store_addr);
                        }
                    }
            }

            2 => {
                let want_contact_discovery_id = dialoguer::Input::<String>::new()
                    .with_prompt("Who do you want to make contact discovery?")
                    .interact()
                    .unwrap();

                let want_contact_discovery_id = random_id!(IDENTIFIER_STRING_LENGTH);

                let file = OpenOptions::new()
                    .read(true)
                    .open("src/my_info.json")?;
                let my_info: MyInfo = serde_json::from_reader(file)?;
                
                println!("Creating new contact object");
                let crypto = Arke::id_nike_and_handshake(my_info.id.clone(), want_contact_discovery_id.clone());
                let symmetric_key = crypto._symmetric_key;
                let write_tag = crypto._alice_write_tag;
                let read_tag = crypto._bob_read_tag;
                let store_addr_string = hex::encode(Arke::to_address(&write_tag));
                let store_addr = Address::from_str(&store_addr_string).unwrap();
                let new_contact = Contact {
                    id: want_contact_discovery_id.clone(),
                    store_addr: store_addr,
                    write_tag: write_tag,
                    read_tag: read_tag,
                    symmetric_key: symmetric_key,
                };
                // Write to the file
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("src/contacts.json")?;
                // Read the existing contacts
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let mut contacts: Vec<Contact> = match serde_json::from_str(&contents) {
                    Ok(contacts) => contacts,
                    Err(_) => Vec::new(), // If error while parsing, treat as empty list
                };
                // Append the new contact
                contacts.push(new_contact);
                // Write contacts back to the file
                let file = File::create("src/contacts.json")?;
                serde_json::to_writer(&file, &contacts)?;
            }

            3 => {
                break;
            }

            _ => {
                println!("Invalid selection");
            }
        }
    } */












    /*/* Two users (Alice and Bob) run id-nike and handshake */
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
*/

