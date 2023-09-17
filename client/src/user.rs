// ---------------------------------------
// File: user.rs
// Date: 11 Sept 2023
// Description: Sign up new user (client-side)
// ---------------------------------------
#![allow(unused_variables)]
#![allow(unused_assignments)]

use rand::thread_rng;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, BufWriter, Cursor};
use serde::{Serialize, Deserialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::fs::File;
use tokio::time::Duration;
use arke_core::{ UserSecretKey, BlindIDCircuitParameters, PartialSecretKey,
                 RegistrarPublicKey, UserID, RegistrationAttestation, 
                 BlindPartialSecretKey, ThresholdObliviousIdNIKE, 
            };
use ark_ec::bls12::Bls12;
use ark_ec::bw6::BW6;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_bw6_761::{BW6_761, Parameters as Parameters761};
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
// Length of the id string
const IDENTIFIER_STRING_LENGTH: usize = 8;
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
struct User {
    id_string: String,
}

pub async fn user () -> Result<(), Box<dyn std::error::Error>> {
    // Read my_info.bin
    let mut my_info_file = File::open("src/my_info.bin").unwrap();
    let mut deserialized: Vec<u8> = Vec::new();
    my_info_file.read_to_end(&mut deserialized).unwrap();

    // Check whether my_info.bin is empty or not (i.e. whether the client is a user or not)
    let metadata = my_info_file.metadata().unwrap();

    // If not empty, the client is a user, read my_info
    if metadata.len() != 0 {
        // Derialize my_info.bin to read my_info object
        let mut cursor = Cursor::new(&deserialized);
        let my_info = MyInfo::deserialize(&mut cursor).unwrap();
        // Print my_info
        println!("ID string: {}\nEth address: {}\nUser secret key: {:?}",
                my_info.id_string, my_info.eth_addr, my_info.sk);
    }

    // If empty, the client is not a user, create new user info
    else { 
        // Ask the client's eth_addr, which is used later for making transactions
        let eth_addr = dialoguer::Input::<String>::new()
            .with_prompt("What is your eth address")
            .interact()
            .unwrap();
        // Ask the client's id_string
        let mut id_string = String::new();
        loop {
            id_string = dialoguer::Input::<String>::new()
            .with_prompt("What is your ID")
            .interact()
            .unwrap();

            if id_string.chars().all(char::is_alphanumeric) == false {
                println!("Your ID has to be alphanumeric!");
                continue;
            }
            if id_string.len() != IDENTIFIER_STRING_LENGTH {
                println!("Your ID has to be {} digits long!", IDENTIFIER_STRING_LENGTH);
                continue;
            }

            // ============================
            // Contact the database server
            // ============================
            println!("About to connect to the database server for checking uniqueness of the ID ...");
            let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
            println!("Successfully connected to the database server for for checking uniqueness of the ID.");
            // Create the request for update_session
            let request = json!({
                "action": "check_uniqueness",
                "id_string": id_string.clone(),
            });
            // Convert the request to a byte array
            let request_bytes = serde_json::to_vec(&request).expect("Could not convert request");
            // Write the request to the stream
            stream.write_all(&request_bytes).await.expect("Could not write the stream");
            // Create a buffer to read the response into
            let mut buf = vec![0; 1024];
            let n = stream.read(&mut buf).await?;
            // Parse the response
            let response: serde_json::Value = match serde_json::from_slice(&buf[..n]) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Failed to parse the response: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>);
                }
            };
            // Print the response
            println!("Response: {}", response);
            if let Some(status) = response.get("status") {
                match status.as_str() {
                    Some("success") => {
                        break;
                    },
                    Some("error") => {
                        println!("This ID is taken!");
                        continue;
                    },
                    _ => {
                        println!("Invalid response from server");
                    }
                }
            }
        }
        

        // ==================================
        // Contact the registration authority
        // ==================================
        println!("About to connect to the registration authority...");
        let mut r_authority_stream = TcpStream::connect("127.0.0.1:8082").await?;
        println!("Successfully connected to the registration authority.");
        // Create the request for ID-NIKE.Register, 
        let request = json!({
            "action": "to_Register",
            "id_string": id_string.clone(),
        });
        // Convert the request to a byte array
        let request_bytes = serde_json::to_vec(&request)?;
        // Write the request to the stream
        r_authority_stream.write_all(&request_bytes).await?;
        // Create a buffer to read the response into
        let mut buf = vec![0; 1024];
        let n = r_authority_stream.read(&mut buf).await?;
        // Parse the response
        let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;
        // Print the response
        println!("Response: {}", response);
        // Initialize reg_attestation as None
        let mut reg_attestation: Option<RegistrationAttestation<Bls12<Parameters>>> = None;
        // Initialize reg_attestation_base64 as None
        let mut reg_attestation_base64: Option<String> = None;
        if let Some(status) = response.get("status") {
            match status.as_str() {
                Some("success") => {
                    if let Some(reg_attestation_value) = response.get("reg_attestation") {
                        reg_attestation_base64 = Some(reg_attestation_value.as_str().unwrap().to_string());
                    } 
                },
                _ => {
                    println!("Invalid response from server");
                }
            }
        }
        println!("- Deserializing reg_attestation");
        if let Some(reg_attestation_base64) = reg_attestation_base64 {
            // Decode from base64
            let reg_attestation_bytes = base64::decode(&reg_attestation_base64).unwrap();
            // CanonicalDeserialize 
            let mut reg_attestation_cursor = Cursor::new(&reg_attestation_bytes);
            reg_attestation = Some(RegistrationAttestation::<Bls12<Parameters>>::deserialize(&mut reg_attestation_cursor).unwrap());
        } 
        let reg_attestation = match reg_attestation {
            Some(reg_attestation) => reg_attestation,
            None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "reg_attestation is None"))),
        };

        // Create the request for getting the registrar_public_key rsk, 
        let request = json!({
            "action": "get_registrar_public_key",
        });
        // Convert the request to a byte array
        let request_bytes = serde_json::to_vec(&request)?;
        // Write the request to the stream
        r_authority_stream.write_all(&request_bytes).await?;
        // Create a buffer to read the response into
        let mut buf = vec![0; 1024];
        let n = r_authority_stream.read(&mut buf).await?;
        // Parse the response
        let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;
        // Print the response
        println!("Response: {}", response);
        // Initialize registrar_public_key as None
        let registrar_public_key: Option<RegistrarPublicKey<Bls12<Parameters>>> = None;
        // Initialize registrar_public_key_base64 as None
        let mut registrar_public_key_base64: Option<String> = None;
        if let Some(status) = response.get("status") {
            match status.as_str() {
                Some("success") => {
                    if let Some(registrar_public_key_value) = response.get("registrar_public_key") {
                        registrar_public_key_base64 = Some(registrar_public_key_value.as_str().unwrap().to_string());
                    } 
                },
                _ => {
                    println!("Invalid response from server");
                }
            }
        }
        drop(r_authority_stream);   


        // =================================
        // Contact the key-issuing authority 
        // =================================
        println!("About to connect to the key-issuing authority...");
        let mut k_authority_stream = TcpStream::connect("127.0.0.1:8081").await?;
        println!("Successfully connected to the key-issuing authority.");
        // Initialize pp_zk as None
        let mut pp_zk: Option<BlindIDCircuitParameters<BW6<Parameters761>>> = None;
        // Initialize pp_zk_base64 as None
        let mut pp_zk_base64: Option<String> = None;
         // Create the request for getting pp_zk, 
         let request = json!({
            "action": "get_pp_zk",
        });
        // Convert the request to a byte array
        let request_bytes = serde_json::to_vec(&request)?;
        // Write the request to the stream
        k_authority_stream.write_all(&request_bytes).await?;
        // Create a buffer to read the response into
        let mut buf = vec![0; 1024]; 
        let mut response = Vec::new();
        loop {
            let timeout = tokio::time::sleep(Duration::from_secs(5));
            tokio::pin!(timeout);
            tokio::select! {
                _ = &mut timeout => {
                    eprintln!("Timeout while reading from the stream");
                    break;
                },
                result = k_authority_stream.read(&mut buf) => {
                    match result {
                        Ok(n) if n == 0 => break,
                        Ok(n) => {
                            response.extend_from_slice(&buf[..n]);
                        },
                        Err(e) => {
                            eprintln!("An error occurred while reading from the stream: {}", e);
                            break;
                        }
                    }
                },
            };
        }
        // Parse the response
        let response: serde_json::Value = match serde_json::from_slice(&response[..]) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to parse the response: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
        };
        // Print the response
        println!("Response: {}", response);
        if let Some(status) = response.get("status") {
            match status.as_str() {
                Some("success") => {
                    if let Some(pp_zk_value) = response.get("pp_zk") {
                        pp_zk_base64 = Some(pp_zk_value.as_str().unwrap().to_string());
                    } 
                },
                _ => {
                    println!("Invalid response from server");
                }
            }
        }
        println!("- Deserializing pp_zk");
        if let Some(pp_zk_base64) = pp_zk_base64 {
            // Decode from base64
            let pp_zk_bytes = base64::decode(&pp_zk_base64).unwrap();
            // CanonicalDeserialize 
            let mut pp_zk_cursor = Cursor::new(&pp_zk_bytes);
            pp_zk = Some(BlindIDCircuitParameters::<BW6<Parameters761>>::deserialize(&mut pp_zk_cursor).unwrap());
        } 
        let pp_zk = match pp_zk {
            Some(pp_zk) => pp_zk,
            None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "pp_zk is None"))),
        };

        let mut rng = thread_rng();
        let user_id = UserID::new(&id_string);
        // Run ID-NIKE.Blind
        println!("- Running Blind");
        let (blinding_factor, blind_id, blind_reg_attestation) =
            ArkeIdNIKE::blind(&pp_zk, &user_id, REGISTRAR_DOMAIN, &reg_attestation, &mut rng).unwrap();
        println!("✓ Finished Blind");
        let mut blind_id_bytes = Vec::new();
        blind_id.serialize(&mut blind_id_bytes).unwrap();
        let blind_id_base64 = base64::encode(&blind_id_bytes);  
        let mut blind_reg_attestation_bytes = Vec::new();
        blind_reg_attestation.serialize(&mut blind_reg_attestation_bytes).unwrap();
        let blind_reg_attestation_base64 = base64::encode(&blind_reg_attestation_bytes);     

        // Create the request for ID-NIKE.VerifyID and ID-NIKE.BlindPartialExtract, 
        let request = json!({
            "action": "to_VerifyID_and_BlindPartialExtract",
            "registrar_public_key_base64": registrar_public_key_base64.unwrap(),
            "blind_id_base64": blind_id_base64,
            "blind_reg_attestation_base64": blind_reg_attestation_base64,
        });
        // Convert the request to a byte array
        let request_bytes = serde_json::to_vec(&request)?;
        // Write the request to the stream
        k_authority_stream.write_all(&request_bytes).await?;
        // Create a buffer to read the response into
        let mut buf = vec![0; 1024]; 
        let mut response = Vec::new();
        loop {
            let timeout = tokio::time::sleep(Duration::from_secs(5));
            tokio::pin!(timeout);
            tokio::select! {
                _ = &mut timeout => {
                    eprintln!("Timeout while reading from the stream");
                    break;
                },
                result = k_authority_stream.read(&mut buf) => {
                    match result {
                        Ok(n) if n == 0 => break,
                        Ok(n) => {
                            response.extend_from_slice(&buf[..n]);
                        },
                        Err(e) => {
                            eprintln!("An error occurred while reading from the stream: {}", e);
                            break;
                        }
                    }
                },
            };
        }
        // Parse the response
        let response: serde_json::Value = serde_json::from_slice(&response[..])?;
        // Print the response
        println!("Response: {}", response);
        // Initialize blind_partial_user_keys as None
        let mut blind_partial_user_keys: Vec<BlindPartialSecretKey<Bls12<Parameters>>> = Vec::new();
        // Initialize blind_partial_user_keys_base64 as None
        let mut blind_partial_user_keys_base64: Option<String> = None;
        if let Some(status) = response.get("status") {
            match status.as_str() {
                Some("success") => {
                    if let Some(blind_partial_user_keys_value) = response.get("blind_partial_user_keys") {
                        blind_partial_user_keys_base64 = Some(blind_partial_user_keys_value.as_str().unwrap().to_string());
                    } 
                },
                _ => {
                    println!("Invalid response from server");
                }
            }
        }
        println!("- Deserializing blind_partial_user_keys");
        if let Some(blind_partial_user_keys_base64) = blind_partial_user_keys_base64 {
            // Decode from base64
            let blind_partial_user_keys_bytes = base64::decode(&blind_partial_user_keys_base64).unwrap();
            // CanonicalDeserialize 
            let mut blind_partial_user_keys_cursor = Cursor::new(&blind_partial_user_keys_bytes);
            loop {
                match BlindPartialSecretKey::<Bls12<Parameters>>::deserialize(&mut blind_partial_user_keys_cursor) {
                    Ok(blind_partial_user_key) => {
                        blind_partial_user_keys.push(blind_partial_user_key);
                    }
                    Err(e) => {
                        //eprintln!("Error during deserialization: {:?}", e);
                        break;
                    }
                }
            }
        } 

        // Run ID-NIKE.Unblind 
        println!("- Running Unblind");
        let partial_user_keys: Vec<PartialSecretKey<Bls12_377>> = blind_partial_user_keys
            .iter()
            .map(|blind_partial_sk| ArkeIdNIKE::unblind(blind_partial_sk, &blinding_factor))
            .collect();
        println!("✓ Finished Unblind");
    
        // Run ID-NIKE.Combine
        println!("- Running Combine");
        let sk = ArkeIdNIKE::combine(&partial_user_keys, THRESHOLD).unwrap();
        println!("✓ Finished Combine");
        drop(k_authority_stream);


        // Create new my_info object
        let my_info = MyInfo {
            id_string: id_string,
            eth_addr: eth_addr,
            sk: sk,
        };
        // Serialize the new my_info object
        let mut serialized: Vec<u8> = Vec::new();
        my_info.serialize(&mut serialized).unwrap();
        // Write to my_info.bin
        let mut my_info_file = BufWriter::new(File::create("src/my_info.bin").unwrap());
        my_info_file.write_all(&serialized).unwrap();

        // Create new user object
        let new_user = User {
            id_string: my_info.id_string,
        };
        println!("About to connect to the server for adding your info to the user database...");
        let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
        println!("Successfully connected to the server for adding your info to the user database.");
        // Create the request for add_user, i.e. write the new user object to all_users.json in server
        let request = json!({
            "action": "add_user",
            "id_string": new_user.id_string,
        });
        // Convert the request to a byte array
        let request_bytes = serde_json::to_vec(&request)?;
        // Write the request to the stream
        stream.write_all(&request_bytes).await?;
        // Create a buffer to read the response into
        let mut buf = vec![0; 1024];
        let n = stream.read(&mut buf).await?;
        // Parse the response
        let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;
        // Print the response
        println!("Response: {}", response);
        drop(stream);
    }

    Ok(())
}