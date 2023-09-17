// ---------------------------------------
// File: registration_authority.rs
// Date: 01 Sept 2023
// Description: System setup
//              Sign up new user (registration authority-side)
// ---------------------------------------
#![allow(non_camel_case_types)]
#![allow(private_in_public)]
#![allow(unused_variables)]

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use serde_json::json;
use std::sync::Arc;

use ark_ec::bls12::Bls12;
use arke_core::{UserID, ThresholdObliviousIdNIKE, RegistrarPublicKey};
use ark_serialize::CanonicalSerialize;
use rand::thread_rng;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_bw6_761::BW6_761;
use ark_bls12_377::FrParameters;
use ark_ff::Fp256;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
const REGISTRAR_DOMAIN: &'static [u8] = b"registration";

#[derive(Clone)]
pub struct registrationAuthority {
    registrar_secret_key: Arc<Fp256<FrParameters>>,
    registrar_public_key: Arc<RegistrarPublicKey<Bls12<Parameters>>>,
}

impl registrationAuthority {
    pub async fn new() -> Self {
        let mut rng = thread_rng();
        // Run ID-NIKE.Setup
        println!("- Running Setup");
        let (_pp_registration, registrar_secret_key, registrar_public_key) =
            ArkeIdNIKE::setup_registration(&mut rng);
        println!("✓ Finished Setup");

        Self { registrar_secret_key: Arc::new(registrar_secret_key),
               registrar_public_key: Arc::new(registrar_public_key),
        }
    }


    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:8082").await?;

        loop {
            let (mut socket, _) = listener.accept().await?;
            let registrar_secret_key = Arc::clone(&self.registrar_secret_key);
            let registrar_public_key = Arc::clone(&self.registrar_public_key);

            tokio::spawn(async move {
                let mut buf = vec![0; 1024];

                loop {
                    match socket.read(&mut buf).await {
                        // Connection closed
                        Ok(n) if n == 0 => return,
                        // Connection open
                        Ok(n) => {
                            let request: Value = serde_json::from_slice(&buf[..n]).unwrap();
                            // Handle request
                            let response = process_request(request, &registrar_secret_key, &registrar_public_key).await;
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
                        registrar_secret_key: &Arc<Fp256<FrParameters>>,
                        registrar_public_key: &Arc<RegistrarPublicKey<Bls12<Parameters>>>,) -> Value {
    match request["action"].as_str() {
        Some("to_Register") => {
            let id_string = request["id_string"].as_str().unwrap().to_string();
            let id = UserID::new(&id_string);
            // Run ID-NIKE.Register
            println!("- Running Register");
            let reg_attestation = ArkeIdNIKE::register(&registrar_secret_key, &id, REGISTRAR_DOMAIN).unwrap();
            println!("✓ Finished Register");
            let mut reg_attestation_bytes = Vec::new();
            reg_attestation.serialize(&mut reg_attestation_bytes).unwrap();
            let reg_attestation_str = base64::encode(&reg_attestation_bytes);  
            json!({ "status": "success", "message": "✓ Registered", 
                    "reg_attestation": reg_attestation_str,
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