// ---------------------------------------
// File: key_issuing_authority.rs
// Date: 01 Sept 2023
// Description: System setup
//              Sign up new user (key-issuing authority-side)
// ---------------------------------------
#![allow(non_camel_case_types)]
#![allow(private_in_public)]
#![allow(unused_variables)]

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use serde_json::json;
use std::sync::Arc;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use ark_std::io::Cursor;

use ark_ec::bls12::Bls12;
use arke_core::{ThresholdObliviousIdNIKE, IssuerPublicKey, BLSPublicParameters,
                BlindIDCircuitParameters, RegistrarPublicKey, UserID,
                BlindPartialSecretKey, BlindRegistrationAttestation, BlindID};
use rand::thread_rng;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_ec::bw6::BW6;
use ark_bw6_761::Parameters as Parameters761;
use ark_bw6_761::BW6_761;
use ark_bls12_377::FrParameters;
use ark_ff::Fp256;
use ark_ff::One;
use secret_sharing::shamir_secret_sharing::SecretShare;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
/// Total number of participants
const NUMBER_OF_PARTICIPANTS: usize = 10;
/// Maximum number of dishonest key-issuing authorities that the system can tolerate
const THRESHOLD: usize = 3;
const REGISTRAR_DOMAIN: &'static [u8] = b"registration";


#[derive(Clone)]
pub struct keyIssuingAuthority {
    pp_zk: Arc<BlindIDCircuitParameters<BW6<Parameters761>>>,
    pp_issuance: Arc<BLSPublicParameters<Bls12<Parameters>>>,
    honest_issuers_secret_keys: Arc<Vec<SecretShare<Fp256<FrParameters>>>>,
    honest_issuers_public_keys: Arc<Vec<IssuerPublicKey<Bls12<Parameters>>>>,
}

impl keyIssuingAuthority {
    pub async fn new() -> Self {
        let mut rng = thread_rng();

        let id = UserID::new("00000000");
        let num_of_domain_sep_bytes = REGISTRAR_DOMAIN.len();
        let num_of_identifier_bytes = id.0.as_bytes().len();
        let num_of_blinding_factor_bits = ark_bls12_377::Fr::one().serialized_size() * 8;
        // Simulate the zk-SNARK trusted setup
        println!("- Running zk-SNARK trusted setup");
        let pp_zk = ArkeIdNIKE::setup_blind_id_proof(
            num_of_domain_sep_bytes,
            num_of_identifier_bytes,
            num_of_blinding_factor_bits,
            &mut rng,
        )
        .unwrap();
        println!("✓ Finished zk-SNARK trusted setup");

        // Simulate the ID-NIKE.SetupDKG between participants
        println!("- Running SetupDKG");
        let (pp_issuance, honest_issuers_secret_keys, honest_issuers_public_keys) =
            ArkeIdNIKE::simulate_issuers_DKG(THRESHOLD, NUMBER_OF_PARTICIPANTS, &mut rng).unwrap();
        println!("✓ Finished SetupDKG");

        Self { pp_zk: Arc::new(pp_zk),
               pp_issuance: Arc::new(pp_issuance),
               honest_issuers_secret_keys: Arc::new(honest_issuers_secret_keys),
               honest_issuers_public_keys: Arc::new(honest_issuers_public_keys),
        }
    }


    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:8081").await?;

        loop {
            let (mut socket, _) = listener.accept().await?;
            let pp_zk = Arc::clone(&self.pp_zk);
            let pp_issuance = Arc::clone(&self.pp_issuance);
            let honest_issuers_secret_keys = Arc::clone(&self.honest_issuers_secret_keys);
            let honest_issuers_public_keys = Arc::clone(&self.honest_issuers_public_keys);

            tokio::spawn(async move {
                let mut buf = vec![0; 4096];

                loop {
                    match socket.read(&mut buf).await {
                        // Connection closed
                        Ok(n) if n == 0 => return,
                        // Connection open  
                        Ok(n) => {
                            let request: Value = serde_json::from_slice(&buf[..n]).unwrap();
                            // Handle request 
                            let response = process_request(request, &pp_zk, &pp_issuance, 
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
                        pp_zk: &Arc<BlindIDCircuitParameters<BW6<Parameters761>>>,
                        pp_issuance: &Arc<BLSPublicParameters<Bls12<Parameters>>>,
                        honest_issuers_secret_keys: &Arc<Vec<SecretShare<Fp256<FrParameters>>>>,
                        honest_issuers_public_keys: &Arc<Vec<IssuerPublicKey<Bls12<Parameters>>>>,) -> Value {
    match request["action"].as_str() {
        Some("get_pp_zk") => {
            let mut pp_zk_bytes = Vec::new();
            pp_zk.serialize(&mut pp_zk_bytes).unwrap();
            let pp_zk_str = base64::encode(&pp_zk_bytes);   
            
            json!({ "status": "success", "message": "✓ Got pp_zk", 
                    "pp_zk": pp_zk_str, 
                 })
        },
        
        Some("to_VerifyID_and_BlindPartialExtract") => {
            let registrar_public_key_base64 = request["registrar_public_key_base64"].as_str().unwrap().to_string();
            // Decode from base64
            let registrar_public_key_bytes = base64::decode(&registrar_public_key_base64).unwrap();
            // CanonicalDeserialize 
            let mut registrar_public_key_cursor = Cursor::new(&registrar_public_key_bytes);
            let registrar_public_key = RegistrarPublicKey::<Bls12<Parameters>>::deserialize(&mut registrar_public_key_cursor).unwrap(); 

            let blind_id_base64 = request["blind_id_base64"].as_str().unwrap().to_string();
            // Decode from base64
            let blind_id_bytes = base64::decode(&blind_id_base64).unwrap();
            // CanonicalDeserialize 
            let mut blind_id_cursor = Cursor::new(&blind_id_bytes);
            let blind_id = BlindID::<Bls12<Parameters>, BW6_761>::deserialize(&mut blind_id_cursor).unwrap();
            
            let blind_reg_attestation_base64 = request["blind_reg_attestation_base64"].as_str().unwrap().to_string();
            // Decode from base64
            let blind_reg_attestation_bytes = base64::decode(&blind_reg_attestation_base64).unwrap();
            // CanonicalDeserialize 
            let mut blind_reg_attestation_cursor = Cursor::new(&blind_reg_attestation_bytes);
            let blind_reg_attestation = BlindRegistrationAttestation::<Bls12<Parameters>>::deserialize(&mut blind_reg_attestation_cursor).unwrap();

            // Run ID-NIKE.VerifyID and ID-NIKE.BlindPartialExtract
            println!("- Running VerifyID and BlindPartialExtract");
            let honest_issuers_secret_keys_vec = Arc::try_unwrap(honest_issuers_secret_keys.clone()).unwrap_or_else(|shared_vec| (*shared_vec).clone());
            let blind_partial_user_keys: Vec<BlindPartialSecretKey<Bls12_377>> = honest_issuers_secret_keys_vec
                .iter()
                .zip(honest_issuers_public_keys.iter())
                .map(|(secret_key, _public_key)| {
                    ArkeIdNIKE::blind_partial_extract(
                        &pp_issuance,
                        pp_zk,
                        &registrar_public_key,
                        secret_key,
                        &blind_id,
                        &blind_reg_attestation,
                        REGISTRAR_DOMAIN,
                    ).unwrap()
                }).collect();
            println!("✓ Finished VerifyID and BlindPartialExtract");

            let mut serialized_keys = Vec::new();
            // Iterate through the vector, serializing each key individually
            for key in &blind_partial_user_keys {
                // Create a Vec<u8> to hold the serialized version of each key
                let mut serialized_key = Vec::new();
                // Serialize each key into the serialized_key buffer
                key.serialize(&mut serialized_key).unwrap();
                // Extend serialized_keys Vec with each serialized_key
                serialized_keys.extend(serialized_key);
            }
            let blind_partial_user_keys_str = base64::encode(&serialized_keys); 

            json!({ "status": "success", "message": "✓ Finished VerifyID and BlindPartialExtract", 
                "blind_partial_user_keys": blind_partial_user_keys_str,
            })
        },

        _ => {
            json!({ "status": "error", "message": "invalid action" })
        },
    }
}