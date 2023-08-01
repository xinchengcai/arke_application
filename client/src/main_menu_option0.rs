#![allow(unused_variables)]
#![allow(unused_assignments)]

use rand::{CryptoRng, thread_rng, Rng};
//const IDENTIFIER_STRING_LENGTH: usize = 8;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read, BufWriter, Cursor};
use serde::{Serialize, Deserialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::fs::File;
use tokio::time::Duration;

use arke_core::{ UserSecretKey, BlindIDCircuitParameters,
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
use secret_sharing::shamir_secret_sharing::SecretShare;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
/// Length of the id string
const IDENTIFIER_STRING_LENGTH: usize = 8;
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
struct User {
    id_string: String,
    unread: bool,
    session: String,
}


pub async fn option0 () -> Result<(), Box<dyn std::error::Error>> {
    // Read my_info.bin
    let mut my_info_file = File::open("src/my_info.bin").unwrap();
    let mut deserialized: Vec<u8> = Vec::new();
    my_info_file.read_to_end(&mut deserialized).unwrap();

    // Check whether my_info.bin is empty or not, i.e. whether a user or not
    let metadata = my_info_file.metadata().unwrap();

    // If not empty, I am a user, read my_info
    if metadata.len() != 0 {
        // Derialize my_info.bin to read my_info object
        let mut cursor = Cursor::new(&deserialized);
        let my_info = MyInfo::deserialize(&mut cursor).unwrap();
        // Print my_info
        println!("ID string: {}\nEth address: {}\nUser secret key: {:?}",
                my_info.id_string, my_info.eth_addr, my_info.sk);
    }

    // If empty, I am not user, create new user info
    else { 
        // Ask my eth_addr
        let eth_addr = dialoguer::Input::<String>::new()
            .with_prompt("What is your eth address")
            .interact()
            .unwrap();
        // Ask my id_string
        let mut id_string = String::new();
        loop {
            id_string = dialoguer::Input::<String>::new()
            .with_prompt("What ID would you like")
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

            println!("About to connect to the server for checking uniqueness of the ID ...");
            let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
            println!("Successfully connected to the server for for checking uniqueness of the ID.");
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
        

        println!("About to connect to the server for getting setup details ...");
        let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
        println!("Successfully connected to the server for getting setup details.");

        // Initialize pp_zk as None
        let mut pp_zk: Option<BlindIDCircuitParameters<BW6<Parameters761>>> = None;
        // Initialize pp_zk_base64 as None
        let mut pp_zk_base64: Option<String> = None;
        // Initialize pp_issuance as None
        let mut pp_issuance: Option<BLSPublicParameters<Bls12<Parameters>>> = None;
        // Initialize pp_issuance_base64 as None
        let mut pp_issuance_base64: Option<String> = None;

        // Create the request for get_pp_zk, 
        let request = json!({
            "action": "get_pp_zk",
        });
        // Convert the request to a byte array
        let request_bytes = serde_json::to_vec(&request)?;
        // Write the request to the stream
        stream.write_all(&request_bytes).await?;
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
                result = stream.read(&mut buf) => {
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
        // Print the response
        println!("Response: {}", response);
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
            let timeout = tokio::time::sleep(Duration::from_secs(5));
            tokio::pin!(timeout);
            tokio::select! {
                _ = &mut timeout => {
                    eprintln!("Timeout while reading from the stream");
                    break;
                },
                result = stream.read(&mut buf) => {
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
        // Print the response
        println!("Response: {}", response);
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
        // Print the response
        println!("Response: {}", response);
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
    
        println!("- Deserializing");
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
                        //eprintln!("Error during deserialization: {:?}", e);
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
                        //eprintln!("Error during deserialization: {:?}", e);
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
    
        let id = UserID::new(&id_string);
        let mut rng = thread_rng();
    
        println!("- Getting your private key:");
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
            unread: false,
            session: String::new(),
        };
        println!("About to connect to the server for adding your info to the user database...");
        let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
        println!("Successfully connected to the server for adding your info to the user database.");
        // Create the request for add_user, i.e. write the new user object to all_users.json in server
        let request = json!({
            "action": "add_user",
            "id_string": new_user.id_string,
            "unread": new_user.unread,
            "session": new_user.session,
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