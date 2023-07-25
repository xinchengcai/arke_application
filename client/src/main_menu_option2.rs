#![allow(unused_assignments)]
#![allow(dead_code)]

use web3::types::Address;
use web3::types::H160;
use std::str::FromStr;
use arke_core::{StoreKey, UserSecretKey, BlindIDCircuitParameters,
                BLSPublicParameters, IssuerPublicKey, RegistrarPublicKey, 
                UserID, IssuancePublicParameters, IssuerSecretKey, 
                ThresholdObliviousIdNIKE, RegistrarSecretKey, BlindPartialSecretKey,
                PartialSecretKey, };
use ark_ec::bls12::Bls12;
use ark_ec::bw6::BW6;
use ark_bw6_761::Parameters as Parameters761;
use ark_ff::Fp256;
use ark_bls12_377::FrParameters;
use ark_bw6_761::BW6_761;
use ark_bls12_377::{Bls12_377, Parameters};
use rand::{thread_rng, CryptoRng, Rng};
use secret_sharing::shamir_secret_sharing::SecretShare;
use crate::arke_frontend::Arke;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, Cursor};
use serde::{Serialize, Deserialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::fs::File;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
/// Maximum number of dishonest key-issuing authorities that the system can tolerate
const THRESHOLD: usize = 3;
/// Domain identifier for the registration authority of this example
const REGISTRAR_DOMAIN: &'static [u8] = b"registration";

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
struct MyInfo {
    nickname: String,
    id_string: String,
    eth_addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Contact {
    nickname: String,
    id_string: String,
    store_addr: H160,
    own_write_tag: StoreKey,
    own_read_tag: StoreKey,
    symmetric_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    nickname: String,
    id_string: String,
    eth_addr: String,
    finding: String,
    key_id: String,
    unread: bool,
}

pub async fn option2() -> Result<(), Box<dyn std::error::Error>> {
    let want_contact_discovery_nickname = dialoguer::Input::<String>::new()
        .with_prompt("Who do you want to make contact discovery?")
        .interact()
        .unwrap();
    let mut want_contact_discovery_id_string = String::new();

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
    let contact = contacts.iter().find(|&c| c.nickname == want_contact_discovery_nickname);
    match contact {
        // If the person is in my contact book already
        Some(contact) => {
            println!("{:?} is already in your contacts", contact.nickname);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "User already in contacts")));
        },
        // If the person is not in my contact book
        None => {
            // Connect to the server
            println!("About to connect to the server...");
            let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
            println!("Successfully connected to the server.");
            // Create the request for find_user, i.e. check whether the person is a user or not
            let request = json!({
                "action": "find_user",
                "nickname": want_contact_discovery_nickname,
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
            // Process the response based on the status field
            if let Some(status) = response.get("status") {
                match status.as_str() {
                    // If the person is a user
                    Some("success") => {
                        println!("User found");
                        if let Some(id_string) = response.get("id_string") {
                            // Get the id_string of the user
                            want_contact_discovery_id_string = id_string.as_str().unwrap().to_string();
                            // Close the stream
                            drop(stream);  
                        }
                    },
                    // If the person is not a user
                    Some("error") => {
                        if let Some(message) = response.get("message") {
                            println!("Error: {}", message.as_str().unwrap());
                        }   
                        // Close the stream
                        drop(stream);  
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "User not found")));
                    },
                    // If the server failed to respond
                    _ => {
                        println!("Invalid response from server");
                        // Close the stream
                        drop(stream);  
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid response from server")));
                    },
                }
            }
        },
    };


    // Read my_info.bin
    let mut my_info_file = File::open("src/my_info.bin")?;
    let mut deserialized: Vec<u8> = Vec::new();
    my_info_file.read_to_end(&mut deserialized)?;
    // Derialize my_info.bin to read my_info object
    let mut cursor = Cursor::new(&deserialized);
    let my_info = MyInfo::deserialize(&mut cursor)?;

    // Perform the rest part of id-nike (i.e. locally derive the secret key and the shared seed) 
    // and the entire handshake (i.e. locally derive symmetric key from shared seed, locally derive write and read tag )
    println!("About to connect to the server...");
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Successfully connected to the server.");

    // Create the request for get_pp_zk, 
    let request = json!({
        "action": "get_pp_zk",
    });
    // Convert the request to a byte array
    let request_bytes = serde_json::to_vec(&request)?;
    // Write the request to the stream
    stream.write_all(&request_bytes).await?;
    // Create a buffer to read the response into
    let mut buf = vec![0; 1024]; // change this to a size that suits your needs
    let mut response = Vec::new();
    loop {
        let n = match stream.read(&mut buf).await {
            Ok(n) if n == 0 => {
                break; // end of stream
            }
            Ok(n) => {
                n
            },
            Err(e) => {
                eprintln!("An error occurred while reading from the stream: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
        };
        response.extend_from_slice(&buf[..n]);
        if n < 1024 {
            break;
        }
    }   
    // Parse the response
    let response: serde_json::Value = serde_json::from_slice(&response[..])?;
    // Initialize pp_zk as None
    let mut pp_zk: Option<BlindIDCircuitParameters<BW6<Parameters761>>> = None;
    // Initialize pp_zk_base64 as None
    let mut pp_zk_base64: Option<String> = None;
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

    // Create the request for get_pp_issuance, 
    let request = json!({
        "action": "get_pp_issuance",
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
    // Initialize pp_issuance as None
    let mut pp_issuance: Option<BLSPublicParameters<Bls12<Parameters>>> = None;
    // Initialize pp_issuance_base64 as None
    let mut pp_issuance_base64: Option<String> = None;
    if let Some(status) = response.get("status") {
        match status.as_str() {
            Some("success") => {
                if let Some(pp_issuance_value) = response.get("pp_issuance") {
                    pp_issuance_base64 = Some(pp_issuance_value.as_str().unwrap().to_string());
                } 
            },
            _ => {
                println!("Invalid response from server");
            }
        }
    }

    // Create the request for get_honest_issuers_secret_keys, 
    let request = json!({
        "action": "get_honest_issuers_secret_keys",
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
    // Initialize honest_issuers_secret_keys as None
    let mut honest_issuers_secret_keys: Vec<SecretShare<Fp256<FrParameters>>> = Vec::new();
    // Initialize honest_issuers_secret_keys_base64 as None
    let mut honest_issuers_secret_keys_base64: Option<String> = None;
    if let Some(status) = response.get("status") {
        match status.as_str() {
            Some("success") => {
                if let Some(honest_issuers_secret_keys_value) = response.get("honest_issuers_secret_keys") {
                    honest_issuers_secret_keys_base64 = Some(honest_issuers_secret_keys_value.as_str().unwrap().to_string());
                } 
            },
            _ => {
                println!("Invalid response from server");
            }
        }
    }

    // Create the request for get_honest_issuers_public_keys, 
    let request = json!({
        "action": "get_honest_issuers_public_keys",
    });
    // Convert the request to a byte array
    let request_bytes = serde_json::to_vec(&request)?;
    // Write the request to the stream
    stream.write_all(&request_bytes).await?;
    // Create a buffer to read the response into
    let mut buf = vec![0; 1024]; // change this to a size that suits your needs
    let mut response = Vec::new();
    loop {
        let n = match stream.read(&mut buf).await {
            Ok(n) if n == 0 => {
                break; // end of stream
            }
            Ok(n) => {
                n
            },
            Err(e) => {
                eprintln!("An error occurred while reading from the stream: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
        };
        response.extend_from_slice(&buf[..n]);
        if n < 1024 {
            break;
        }
    }   
    // Parse the response
    let response: serde_json::Value = serde_json::from_slice(&response[..])?;
    // Initialize honest_issuers_public_keys as None
    let mut honest_issuers_public_keys: Vec<IssuerPublicKey<Bls12<Parameters>>> = Vec::new();
    // Initialize honest_issuers_public_keys_base64 as None
    let mut honest_issuers_public_keys_base64: Option<String> = None;
    if let Some(status) = response.get("status") {
        match status.as_str() {
            Some("success") => {
                if let Some(honest_issuers_public_keys_value) = response.get("honest_issuers_public_keys") {
                    honest_issuers_public_keys_base64 = Some(honest_issuers_public_keys_value.as_str().unwrap().to_string());
                } 
            },
            _ => {
                println!("Invalid response from server");
            }
        }
    }

    // Create the request for get_registrar_secret_key, 
    let request = json!({
        "action": "get_registrar_secret_key",
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
    // Initialize registrar_secret_key as None
    let mut registrar_secret_key: Option<Fp256<FrParameters>> = None;
    // Initialize registrar_secret_key_base64 as None
    let mut registrar_secret_key_base64: Option<String> = None;
    if let Some(status) = response.get("status") {
        match status.as_str() {
            Some("success") => {
                if let Some(registrar_secret_key_value) = response.get("registrar_secret_key") {
                    registrar_secret_key_base64 = Some(registrar_secret_key_value.as_str().unwrap().to_string());
                } 
            },
            _ => {
                println!("Invalid response from server");
            }
        }
    }

    // Create the request for get_registrar_public_key, 
    let request = json!({
        "action": "get_registrar_public_key",
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
    // Initialize registrar_public_key as None
    let mut registrar_public_key: Option<RegistrarPublicKey<Bls12<Parameters>>> = None;
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
    drop(stream);

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
    if let Some(pp_issuance_base64) = pp_issuance_base64 {
        // Decode from base64
        let pp_issuance_bytes = base64::decode(&pp_issuance_base64).unwrap();
        // CanonicalDeserialize 
        let mut pp_issuance_cursor = Cursor::new(&pp_issuance_bytes);
        pp_issuance = Some(BLSPublicParameters::<Bls12<Parameters>>::deserialize(&mut pp_issuance_cursor).unwrap());
    } 
    let pp_issuance = match pp_issuance {
        Some(pp_issuance) => pp_issuance,
        None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "pp_issuance is None"))),
    };
    if let Some(honest_issuers_secret_keys_base64) = honest_issuers_secret_keys_base64 {
        // Decode from base64
        let honest_issuers_secret_keys_bytes = base64::decode(&honest_issuers_secret_keys_base64).unwrap();
        // CanonicalDeserialize 
        let mut honest_issuers_secret_keys_cursor = Cursor::new(&honest_issuers_secret_keys_bytes);
        loop {
            match SecretShare::<Fp256<FrParameters>>::deserialize(&mut honest_issuers_secret_keys_cursor) {
                Ok(honest_issuers_secret_key) => {
                    honest_issuers_secret_keys.push(honest_issuers_secret_key);
                }
                Err(e) => {
                    eprintln!("Error during deserialization: {:?}", e);
                    break;
                }
            }
        }
    }
    if let Some(honest_issuers_public_keys_base64) = honest_issuers_public_keys_base64 {
        // Decode from base64
        let honest_issuers_public_keys_bytes = base64::decode(&honest_issuers_public_keys_base64).unwrap();
        // CanonicalDeserialize 
        let mut honest_issuers_public_keys_cursor = Cursor::new(&honest_issuers_public_keys_bytes);
        loop {
            match IssuerPublicKey::<Bls12<Parameters>>::deserialize(&mut honest_issuers_public_keys_cursor) {
                Ok(honest_issuers_public_key) => {
                    honest_issuers_public_keys.push(honest_issuers_public_key);
                }
                Err(e) => {
                    eprintln!("Error during deserialization: {:?}", e);
                    break;
                }
            }
        }
    } 
    if let Some(registrar_secret_key_base64) = registrar_secret_key_base64 {
        // Decode from base64
        let registrar_secret_key_bytes = base64::decode(&registrar_secret_key_base64).unwrap();
        // CanonicalDeserialize 
        let mut registrar_secret_key_cursor = Cursor::new(&registrar_secret_key_bytes);
        registrar_secret_key = Some(<Fp256<FrParameters>>::deserialize(&mut registrar_secret_key_cursor).unwrap());
    } 
    let registrar_secret_key = match registrar_secret_key {
        Some(registrar_secret_key) => registrar_secret_key,
        None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "registrar_secret_key is None"))),
    };
    if let Some(registrar_public_key_base64) = registrar_public_key_base64 {
        // Decode from base64
        let registrar_public_key_bytes = base64::decode(&registrar_public_key_base64).unwrap();
        // CanonicalDeserialize 
        let mut registrar_public_key_cursor = Cursor::new(&registrar_public_key_bytes);
        registrar_public_key = Some(RegistrarPublicKey::<Bls12<Parameters>>::deserialize(&mut registrar_public_key_cursor).unwrap());
    } 
    let registrar_public_key = match registrar_public_key {
        Some(registrar_public_key) => registrar_public_key,
        None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "registrar_public_key is None"))),
    };

    let id = UserID::new(&my_info.id_string);
    let mut rng = thread_rng();

    println!("- You get your private key:");
    let sk = get_user_secret_key(
        &pp_zk,
        &pp_issuance,
        &id,
        THRESHOLD,
        &registrar_secret_key,
        &registrar_public_key,
        REGISTRAR_DOMAIN,
        &honest_issuers_secret_keys,
        &honest_issuers_public_keys,
        &mut rng,
    );
                    
    let crypto = Arke::id_nike_and_handshake(my_info.id_string.clone(), 
                                    want_contact_discovery_id_string.clone(), 
                                                sk.clone());
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
        nickname: want_contact_discovery_nickname.clone(), 
        id_string: want_contact_discovery_id_string.clone(),
        store_addr: store_addr.clone(),
        own_write_tag: own_write_tag.clone(),
        own_read_tag: own_read_tag.clone(),
        symmetric_key: symmetric_key.clone(),
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


pub fn get_user_secret_key<R: Rng + CryptoRng>(
    pp_zk: &BlindIDCircuitParameters<BW6_761>,
    issuance_pp: &IssuancePublicParameters<Bls12_377>,
    user_id: &UserID,
    threshold: usize,
    registrar_secret_key: &RegistrarSecretKey<Bls12_377>,
    registrar_public_key: &RegistrarPublicKey<Bls12_377>,
    registrar_domain: &[u8],
    issuers_secret_keys: &[IssuerSecretKey<Bls12_377>],
    issuers_public_keys: &[IssuerPublicKey<Bls12_377>],
    rng: &mut R,
) -> UserSecretKey<Bls12_377> {
    println!("    - Registration");
    // Register our user
    let reg_attestation =
        ArkeIdNIKE::register(&registrar_secret_key, &user_id, registrar_domain).unwrap();

    // Blind the identifier and token
    println!("    - Blinding (and proof)");
    let (blinding_factor, blind_id, blind_reg_attestation) =
        ArkeIdNIKE::blind(pp_zk, user_id, registrar_domain, &reg_attestation, rng).unwrap();

    // Obtain blind partial secret keys from t+1 honest authorities
    println!("    - BlindPartialExtract (verify reg and proof)");
    let blind_partial_user_keys: Vec<BlindPartialSecretKey<Bls12_377>> = issuers_secret_keys
        .iter()
        .zip(issuers_public_keys.iter())
        .map(|(secret_key, _public_key)| {
            ArkeIdNIKE::blind_partial_extract(
                &issuance_pp,
                pp_zk,
                &registrar_public_key,
                secret_key,
                &blind_id,
                &blind_reg_attestation,
                registrar_domain,
            )
            .unwrap()
        })
        .collect();

    // Unblind each partial key
    println!("    - Unblind");
    let partial_user_keys: Vec<PartialSecretKey<Bls12_377>> = blind_partial_user_keys
        .iter()
        .map(|blind_partial_sk| ArkeIdNIKE::unblind(blind_partial_sk, &blinding_factor))
        .collect();

    // Combine the partial keys to obtain a user secret key
    println!("    - Combine");
    let user_secret_key = ArkeIdNIKE::combine(&partial_user_keys, threshold).unwrap();

    user_secret_key
}