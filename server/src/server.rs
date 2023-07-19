// server.rs
#![allow(non_camel_case_types)]
#![allow(private_in_public)]

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use std::path::Path;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

use ark_ec::bls12::Bls12;
use arke_core::{ UserSecretKey, UserID, ThresholdObliviousIdNIKE,
    BlindIDCircuitParameters, BlindPartialSecretKey, IssuancePublicParameters,
    IssuerPublicKey, IssuerSecretKey, PartialSecretKey, RegistrarPublicKey, RegistrarSecretKey,
};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::io::{Write, Read};
use rand::{thread_rng, CryptoRng, Rng};
use ark_std::One;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_bw6_761::BW6_761;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
/// Total number of participants
const NUMBER_OF_PARTICIPANTS: usize = 10;
/// Maximum number of dishonest key-issuing authorities that the system can tolerate
const THRESHOLD: usize = 3;
/// Domain identifier for the registration authority of this example
const REGISTRAR_DOMAIN: &'static [u8] = b"registration";

#[derive(Serialize, Deserialize, Debug)]
struct User {
    nickname: String,
    id_string: String,
    eth_addr: String,
    finding: String,
    key_id: String,
}
#[derive(Clone)]
pub struct UserDatabase {
    path: PathBuf,
}

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug)]
pub struct sks {
    alice_sk: UserSecretKey<Bls12<Parameters>>,
    bob_sk: UserSecretKey<Bls12<Parameters>>,
}

impl UserDatabase {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub async fn load(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let contents = tokio::fs::read(&self.path).await?;
        let users: Vec<User> = serde_json::from_slice(&contents)?;
        Ok(users)
    }

    pub async fn save(&self, users: &[User]) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_json::to_vec(users)?;
        tokio::fs::write(&self.path, &contents).await?;
        Ok(())
    }
}   

pub struct Server {
    users_db: Arc<UserDatabase>,
    sks_db: Arc<Mutex<HashMap<String, sks>>>,
}

impl Server {
    pub async fn new(user_db_path: impl AsRef<Path>) -> Self {
        let users_db = Arc::new(UserDatabase::new(user_db_path));
        let sks_db = Arc::new(Mutex::new(HashMap::new()));
        Self { users_db, sks_db }
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        loop {
            let (mut socket, _) = listener.accept().await?;
            let users_db = Arc::clone(&self.users_db);
            let sks_db = Arc::clone(&self.sks_db);

            tokio::spawn(async move {
                let mut buf = vec![0; 1024];

                loop {
                    match socket.read(&mut buf).await {
                        Ok(n) if n == 0 => return,  // client closed connection
                        Ok(n) => {
                            let request: Value = serde_json::from_slice(&buf[..n]).unwrap();
                            // handle request here
                            let response = process_request(request, users_db.clone(), sks_db.clone()).await;
                            let response_bytes = serde_json::to_vec(&response).unwrap();
                            if let Err(e) = socket.write_all(&response_bytes).await {
                                eprintln!("failed to write to socket; err = {:?}", e);
                                return;
                            }
                        }
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    }
                }
            });
        }
    }
}


async fn process_request(request: Value, users_db: Arc<UserDatabase>, sks_db: Arc<Mutex<HashMap<String, sks>>>) -> Value {
    match request["action"].as_str() {
        Some("add_user") => {
            let nickname = request["nickname"].as_str().unwrap().to_string();
            let id_string = request["id_string"].as_str().unwrap().to_string();
            let eth_addr = request["eth_addr"].as_str().unwrap().to_string();
            let finding = request["finding"].as_str().unwrap().to_string();
            let key_id = request["key_id"].as_str().unwrap().to_string();
            let user = User { nickname, id_string, eth_addr, finding, key_id};

            // Load users from the JSON file
            let mut users = users_db.load().await.unwrap();

            // Add the new user and save the updated list
            users.push(user);
            users_db.save(&users).await.unwrap();

            Value::String("User added".into())
        },


        Some("find_user") => {
            let users = users_db.load().await.unwrap();
            if let Some(nickname) = request.get("nickname") {
                let user_exists = users.iter().find(|user| user.nickname == nickname.as_str().unwrap());
                if let Some(user) = user_exists {
                    json!({ "status": "success", "message": "User found", "id_string": user.id_string })
                } else {
                    json!({ "status": "error", "message": "User not found" })
                }
            } else {
                json!({ "status": "error", "message": "Missing nickname" })
            }
        },


        Some("compute_sks") => {
            let alice_id_string = request["alice_id_string"].as_str().unwrap().to_string();
            let bob_id_string = request["bob_id_string"].as_str().unwrap().to_string();
            let alice_id_string_clone = alice_id_string.clone();
            let bob_id_string_clone: String = bob_id_string.clone();

            let users = users_db.load().await.unwrap();
            let user_exists = users.iter().find(|user| user.id_string == bob_id_string_clone);
            if let Some(user) = user_exists {
                if user.finding == alice_id_string_clone {
                    json!({ "status": "success", "message": "SKs generated", "key_id": user.key_id })
                }
                else {
                    let key_id = tokio::task::spawn_blocking(move || {
                        /* Arke ID-NIKE */ 
                        let mut rng = thread_rng();
                        // Generate a random user ID
                        let alice_id = UserID::new(&alice_id_string);
                        // Generate a random user ID
                        let bob_id = UserID::new(&bob_id_string);
                        let num_of_domain_sep_bytes = REGISTRAR_DOMAIN.len();
                        let num_of_identifier_bytes = alice_id.0.as_bytes().len();
                        let num_of_blinding_factor_bits = ark_bls12_377::Fr::one().serialized_size() * 8;
                        // Simulate the SNARK trusted setup
                        println!("- Running trusted setup");
                        let pp_zk = ArkeIdNIKE::setup_blind_id_proof(
                            num_of_domain_sep_bytes,
                            num_of_identifier_bytes,
                            num_of_blinding_factor_bits,
                            &mut rng,
                        )
                        .unwrap();
            
                        // Simulate the DKG between issuers
                        println!("- Running DKG");
                        let (pp_issuance, honest_issuers_secret_keys, honest_issuers_public_keys) =
                            ArkeIdNIKE::simulate_issuers_DKG(THRESHOLD, NUMBER_OF_PARTICIPANTS, &mut rng).unwrap();
            
                        // Create a registration authority
                        println!("- Setup registration authority");
                        let (_pp_registration, registrar_secret_key, registrar_public_key) =
                            ArkeIdNIKE::setup_registration(&mut rng);
            
                        // Compute Alice and Bob's respective user secret keys
                        println!("- Alice gets her private keys:");
                        let alice_sk = get_user_secret_key(
                            &pp_zk,
                            &pp_issuance,
                            &alice_id,
                            THRESHOLD,
                            &registrar_secret_key,
                            &registrar_public_key,
                            REGISTRAR_DOMAIN,
                            &honest_issuers_secret_keys,
                            &honest_issuers_public_keys,
                            &mut rng,
                        );
            
                        println!("Bob gets his private keys:");
                        let bob_sk = get_user_secret_key(
                            &pp_zk,
                            &pp_issuance,
                            &bob_id,
                            THRESHOLD,
                            &registrar_secret_key,
                            &registrar_public_key,
                            REGISTRAR_DOMAIN,
                            &honest_issuers_secret_keys,
                            &honest_issuers_public_keys,
                            &mut rng,
                        );
            
                        let key_pair = sks { alice_sk, bob_sk };
                        let key_id = Uuid::new_v4().to_string(); // Use UUID to generate a unique key ID
                        sks_db.lock().unwrap().insert(key_id.clone(), key_pair);
        
                        key_id
                    }).await.unwrap();
        
                    let mut users = users_db.load().await.unwrap();
                    for user in users.iter_mut() {
                        if user.id_string == alice_id_string_clone {
                            user.finding = bob_id_string_clone;
                            user.key_id = key_id.clone();
                            break;
                        }
                    }
                    users_db.save(&users).await.unwrap(); 
                    json!({ "status": "success", "message": "SKs generated", "key_id": key_id })
                }
            } 
            else {
                json!({ "status": "error", "message": "SKs not generated"})
            }
        },


        Some("retrieve_sks") => {
            let key_id = request["key_id"].as_str().unwrap().to_string();
            let id_string = request["id_string"].as_str().unwrap().to_string();
            let mut users = users_db.load().await.unwrap();
            match sks_db.lock().unwrap().get(&key_id) {
                Some(key_pair) => {
                    let mut sk = key_pair.alice_sk;
                    // Ensure I and the user who I want to make contact discovery 
                    // getting corresponding user secret key
                    for user in users.iter_mut() {
                        if user.id_string == id_string {
                            if user.finding != String::new() {
                                break;
                            }
                            else {
                                sk = key_pair.bob_sk;
                                break;
                            }
                        }
                    }

                    let mut sk_bytes = Vec::new();
                    sk.serialize(&mut sk_bytes).unwrap();
                    let sk_str = base64::encode(&sk_bytes);
        
                    json!({ "status": "success", "message": "SK retrieved", "sk": sk_str })
                },

                None => {
                    json!({ "status": "error", "message": "Invalid key ID" })
                }
            }
        },

        _ => {
            json!({ "status": "error", "message": "Invalid action" })
        },
    }
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