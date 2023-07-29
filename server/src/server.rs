// server.rs
#![allow(non_camel_case_types)]
#![allow(private_in_public)]
#![allow(unused_imports)]

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use std::path::Path;
use serde_json::json;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use ark_ec::bls12::Bls12;
use arke_core::{  UserID, ThresholdObliviousIdNIKE, BlindIDCircuitParameters, 
    IssuerPublicKey, RegistrarPublicKey, BLSPublicParameters, random_id,
};
const IDENTIFIER_STRING_LENGTH: usize = 8;
use ark_serialize::CanonicalSerialize;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use ark_std::One;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_bw6_761::Parameters as Parameters761;
use ark_bw6_761::BW6_761;
use ark_ec::bw6::BW6;
use ark_bls12_377::FrParameters;
use ark_ff::Fp256;
use secret_sharing::shamir_secret_sharing::SecretShare;
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
    unread: bool,
    session: String,
}

#[derive(Clone)]
pub struct UserDatabase {
    path: PathBuf,
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

#[derive(Clone)]
pub struct Server {
    users_db: Arc<UserDatabase>,
    pp_zk: Arc<BlindIDCircuitParameters<BW6<Parameters761>>>,
    pp_issuance: Arc<BLSPublicParameters<Bls12<Parameters>>>,
    honest_issuers_secret_keys: Arc<Vec<SecretShare<Fp256<FrParameters>>>>,
    honest_issuers_public_keys: Arc<Vec<IssuerPublicKey<Bls12<Parameters>>>>,
    registrar_secret_key: Arc<Fp256<FrParameters>>,
    registrar_public_key: Arc<RegistrarPublicKey<Bls12<Parameters>>>,
}

impl Server {
    pub async fn new(user_db_path: impl AsRef<Path>) -> Self {
        let users_db = Arc::new(UserDatabase::new(user_db_path));

        let id_string = random_id!(IDENTIFIER_STRING_LENGTH);
        let id = UserID::new(&id_string);
        let mut rng = thread_rng();
        let num_of_domain_sep_bytes = REGISTRAR_DOMAIN.len();
        let num_of_identifier_bytes = id.0.as_bytes().len();
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

        println!("✓ Finished setup");

        Self { users_db,
                pp_zk: Arc::new(pp_zk),
                pp_issuance: Arc::new(pp_issuance),
                honest_issuers_secret_keys: Arc::new(honest_issuers_secret_keys),
                honest_issuers_public_keys: Arc::new(honest_issuers_public_keys),
                registrar_secret_key: Arc::new(registrar_secret_key),
                registrar_public_key: Arc::new(registrar_public_key),
        }
    }


    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        loop {
            let (mut socket, _) = listener.accept().await?;
            let users_db = Arc::clone(&self.users_db);
            let pp_zk = Arc::clone(&self.pp_zk);
            let pp_issuance = Arc::clone(&self.pp_issuance);
            let honest_issuers_secret_keys = Arc::clone(&self.honest_issuers_secret_keys);
            let honest_issuers_public_keys = Arc::clone(&self.honest_issuers_public_keys);
            let registrar_secret_key = Arc::clone(&self.registrar_secret_key);
            let registrar_public_key = Arc::clone(&self.registrar_public_key);

            tokio::spawn(async move {
                let mut buf = vec![0; 1024];

                loop {
                    match socket.read(&mut buf).await {
                        Ok(n) if n == 0 => return,  // client closed connection
                        Ok(n) => {
                            let request: Value = serde_json::from_slice(&buf[..n]).unwrap();
                            // handle request here
                            let response = process_request(request, users_db.clone(),
                                                                    &pp_zk, 
                                                                    &pp_issuance, 
                                                                    &honest_issuers_secret_keys, 
                                                                    &honest_issuers_public_keys, 
                                                                    &registrar_secret_key, 
                                                                    &registrar_public_key).await;
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


async fn process_request(request: Value, users_db: Arc<UserDatabase>,
                        pp_zk: &Arc<BlindIDCircuitParameters<BW6<Parameters761>>>,
                        pp_issuance: &Arc<BLSPublicParameters<Bls12<Parameters>>>,
                        honest_issuers_secret_keys: &Arc<Vec<SecretShare<Fp256<FrParameters>>>>,
                        honest_issuers_public_keys: &Arc<Vec<IssuerPublicKey<Bls12<Parameters>>>>,
                        registrar_secret_key: &Arc<Fp256<FrParameters>>,
                        registrar_public_key: &Arc<RegistrarPublicKey<Bls12<Parameters>>>,) -> Value {
    match request["action"].as_str() {
        Some("add_user") => {
            let nickname = request["nickname"].as_str().unwrap().to_string();
            let id_string = request["id_string"].as_str().unwrap().to_string();
            let unread: bool = request["unread"].as_bool().unwrap();
            let session = request["session"].as_str().unwrap().to_string();
            let user = User { nickname, id_string, unread, session};

            // Load users from the JSON file
            let mut users = users_db.load().await.unwrap();

            // Add the new user and save the updated list
            users.push(user);
            users_db.save(&users).await.unwrap();

            Value::String("✓ User added".into())
        },


        Some("find_user") => {
            let users = users_db.load().await.unwrap();
            if let Some(nickname) = request.get("nickname") {
                let user_exists = users.iter().find(|user| user.nickname == nickname.as_str().unwrap());
                if let Some(user) = user_exists {
                    json!({ "status": "success", "message": "✓ User found", "id_string": user.id_string })
                } else {
                    json!({ "status": "error", "message": "user not found" })
                }
            } else {
                json!({ "status": "error", "message": "missing nickname" })
            }
        },


        Some("update_session") => {
            let mut users = users_db.load().await.unwrap();
            let id_string = request["id_string"].as_str().unwrap().to_string();
            let session = request["session"].as_str().unwrap().to_string();
            for user in users.iter_mut() {
                if user.id_string == id_string {
                    user.session = session;
                    break;
                }
            }
            users_db.save(&users).await.unwrap();
            json!({ "status": "success", "message": "✓ Set session"})
        },


        Some("unread_flag") => {
            let id_string = request["id_string"].as_str().unwrap().to_string();
            let session = request["session"].as_str().unwrap().to_string();
            let rw = request["rw"].as_str().unwrap().to_string();
            let mut users = users_db.load().await.unwrap();
            let mut response = json!({ "status": "error", "message": "user not found" }); // Default error message
            for user in users.iter_mut() {
                if user.id_string == id_string {
                    if rw == "r" {
                        if user.session == session {
                            let result = user.unread;
                            response = json!({ "status": "success", "message": "✓ Got flag", "flag": result});
                            break;
                        }
                        else {
                            response = json!({ "status": "error", "message": "invalid session"});
                            break;
                        }
                    }
                    else if rw == "wt" {
                        user.unread = true;
                        users_db.save(&users).await.unwrap();
                        response = json!({ "status": "success", "message": "✓ Set flag to true"});
                        break;
                    }
                    else if rw == "wf" {
                        user.unread = false;
                        users_db.save(&users).await.unwrap();
                        response = json!({ "status": "success", "message": "✓ Set flag to false"});
                        break;
                    }
                    else {
                        response = json!({ "status": "error", "message": "invalid rw"});
                        break;
                    }
                }
            }
            response
        },


        Some("get_pp_zk") => {
            let mut pp_zk_bytes = Vec::new();
            pp_zk.serialize(&mut pp_zk_bytes).unwrap();
            let pp_zk_str = base64::encode(&pp_zk_bytes);   
            
            json!({ "status": "success", "message": "✓ Got pp_zk", 
                    "pp_zk": pp_zk_str, 
                 })
        },


        Some("get_pp_issuance") => {
            let mut pp_issuance_bytes = Vec::new();
            pp_issuance.serialize(&mut pp_issuance_bytes).unwrap();
            let pp_issuance_str = base64::encode(&pp_issuance_bytes);

            json!({ "status": "success", "message": "✓ Got pp_issuance", 
                    "pp_issuance": pp_issuance_str,
                 })
        },


        Some("get_honest_issuers_secret_keys") => {
            let honest_issuers_secret_keys_vec = Arc::try_unwrap(honest_issuers_secret_keys.clone()).unwrap_or_else(|shared_vec| (*shared_vec).clone());
            let mut serialized_keys = Vec::new();
            // Iterate through the vector, serializing each key individually
            for key in &honest_issuers_secret_keys_vec {
                // Create a Vec<u8> to hold the serialized version of each key
                let mut serialized_key = Vec::new();
                // Serialize each key into the `serialized_key` buffer
                key.serialize(&mut serialized_key).unwrap();
                // Extend our `serialized_keys` Vec with each `serialized_key`
                serialized_keys.extend(serialized_key);
            }
            let honest_issuers_secret_keys_str = base64::encode(&serialized_keys);  

            json!({ "status": "success", "message": "✓ Got honest_issuers_secret_keys", 
                    "honest_issuers_secret_keys": honest_issuers_secret_keys_str,
                 })
        },

        Some("get_honest_issuers_public_keys") => {
            let honest_issuers_public_keys_vec = Arc::try_unwrap(honest_issuers_public_keys.clone()).unwrap_or_else(|shared_vec| (*shared_vec).clone());
            let mut serialized_keys = Vec::new();
            // Iterate through the vector, serializing each key individually
            for key in &honest_issuers_public_keys_vec {
                // Create a Vec<u8> to hold the serialized version of each key
                let mut serialized_key = Vec::new();
                // Serialize each key into the `serialized_key` buffer
                key.serialize(&mut serialized_key).unwrap();
                // Extend our `serialized_keys` Vec with each `serialized_key`
                serialized_keys.extend(serialized_key);
            }
            let honest_issuers_public_keys_str = base64::encode(&serialized_keys);  

            json!({ "status": "success", "message": "✓ Got honest_issuers_public_keys", 
                    "honest_issuers_public_keys": honest_issuers_public_keys_str,
                 })
        },


        Some("get_registrar_secret_key") => {
            let mut registrar_secret_key_bytes = Vec::new();
            registrar_secret_key.serialize(&mut registrar_secret_key_bytes).unwrap();
            let registrar_secret_key_str = base64::encode(&registrar_secret_key_bytes);    

            json!({ "status": "success", "message": "✓ Got registrar_secret_key", 
                    "registrar_secret_key": registrar_secret_key_str,
                 })
        },


        Some("get_registrar_public_key") => {
            let mut registrar_public_key_bytes = Vec::new();
            registrar_public_key.serialize(&mut registrar_public_key_bytes).unwrap();
            let registrar_public_key_str = base64::encode(&registrar_public_key_bytes);      

            json!({ "status": "success", "message": "✓ Got registrar_public_key", 
                    "registrar_public_key": registrar_public_key_str,
                 })
        },


        _ => {
            json!({ "status": "error", "message": "invalid action" })
        },
    }
}