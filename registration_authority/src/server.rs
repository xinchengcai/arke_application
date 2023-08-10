// server.rs
#![allow(non_camel_case_types)]
#![allow(private_in_public)]
#![allow(unused_variables)]

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use serde_json::json;
use std::sync::Arc;

use ark_ec::bls12::Bls12;
use arke_core::{ ThresholdObliviousIdNIKE, RegistrarPublicKey};
use ark_serialize::CanonicalSerialize;
use rand::thread_rng;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_bw6_761::BW6_761;
use ark_bls12_377::FrParameters;
use ark_ff::Fp256;
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;


#[derive(Clone)]
pub struct Server {
    registrar_secret_key: Arc<Fp256<FrParameters>>,
    registrar_public_key: Arc<RegistrarPublicKey<Bls12<Parameters>>>,
}

impl Server {
    pub async fn new() -> Self {
        let mut rng = thread_rng();

        // Create a registration authority
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
                        Ok(n) if n == 0 => return,  // client closed connection
                        Ok(n) => {
                            let request: Value = serde_json::from_slice(&buf[..n]).unwrap();
                            // handle request here
                            let response = process_request(request,
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


async fn process_request(request: Value,
                        registrar_secret_key: &Arc<Fp256<FrParameters>>,
                        registrar_public_key: &Arc<RegistrarPublicKey<Bls12<Parameters>>>,) -> Value {
    match request["action"].as_str() {
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