// server.rs
#![allow(non_camel_case_types)]
#![allow(private_in_public)]
#![allow(unused_variables)]

use ark_serialize::CanonicalSerialize;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use serde_json::json;
use std::sync::Arc;

use ark_ec::bls12::Bls12;
use arke_core::{ThresholdObliviousIdNIKE, IssuerPublicKey, BLSPublicParameters,};
use rand::thread_rng;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_bw6_761::BW6_761;
use ark_bls12_377::FrParameters;
use ark_ff::Fp256;
use secret_sharing::shamir_secret_sharing::SecretShare;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
/// Total number of participants
const NUMBER_OF_PARTICIPANTS: usize = 10;
/// Maximum number of dishonest key-issuing authorities that the system can tolerate
const THRESHOLD: usize = 3;


#[derive(Clone)]
pub struct Server {
    pp_issuance: Arc<BLSPublicParameters<Bls12<Parameters>>>,
    honest_issuers_secret_keys: Arc<Vec<SecretShare<Fp256<FrParameters>>>>,
    honest_issuers_public_keys: Arc<Vec<IssuerPublicKey<Bls12<Parameters>>>>,
}

impl Server {
    pub async fn new() -> Self {
        let mut rng = thread_rng();
        // Simulate the DKG between issuers
        println!("- Running SetupDKG");
        let (pp_issuance, honest_issuers_secret_keys, honest_issuers_public_keys) =
            ArkeIdNIKE::simulate_issuers_DKG(THRESHOLD, NUMBER_OF_PARTICIPANTS, &mut rng).unwrap();

        println!("✓ Finished SetupDKG");

        Self { pp_issuance: Arc::new(pp_issuance),
                honest_issuers_secret_keys: Arc::new(honest_issuers_secret_keys),
                honest_issuers_public_keys: Arc::new(honest_issuers_public_keys),
        }
    }


    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:8081").await?;

        loop {
            let (mut socket, _) = listener.accept().await?;
            let pp_issuance = Arc::clone(&self.pp_issuance);
            let honest_issuers_secret_keys = Arc::clone(&self.honest_issuers_secret_keys);
            let honest_issuers_public_keys = Arc::clone(&self.honest_issuers_public_keys);

            tokio::spawn(async move {
                let mut buf = vec![0; 1024];

                loop {
                    match socket.read(&mut buf).await {
                        Ok(n) if n == 0 => return,  // client closed connection
                        Ok(n) => {
                            let request: Value = serde_json::from_slice(&buf[..n]).unwrap();
                            // handle request here
                            let response = process_request(request, &pp_issuance, 
                                                                    &honest_issuers_secret_keys, 
                                                                    &honest_issuers_public_keys).await;
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


async fn process_request(request: Value,
                        pp_issuance: &Arc<BLSPublicParameters<Bls12<Parameters>>>,
                        honest_issuers_secret_keys: &Arc<Vec<SecretShare<Fp256<FrParameters>>>>,
                        honest_issuers_public_keys: &Arc<Vec<IssuerPublicKey<Bls12<Parameters>>>>,) -> Value {
    match request["action"].as_str() {
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
                // Serialize each key into the serialized_key buffer
                key.serialize(&mut serialized_key).unwrap();
                // Extend serialized_keys Vec with each serialized_key
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
                // Serialize each key into the serialized_key buffer
                key.serialize(&mut serialized_key).unwrap();
                // Extend serialized_keys Vec with each serialized_key
                serialized_keys.extend(serialized_key);
            }
            let honest_issuers_public_keys_str = base64::encode(&serialized_keys);  

            json!({ "status": "success", "message": "✓ Got honest_issuers_public_keys", 
                    "honest_issuers_public_keys": honest_issuers_public_keys_str,
                 })
        },

        _ => {
            json!({ "status": "error", "message": "invalid action" })
        },
    }
}