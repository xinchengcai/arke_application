// ---------------------------------------
// File: server.rs
// Date: 01 Sept 2023
// Description: Sign up new user (database server-side)
// ---------------------------------------
#![allow(non_camel_case_types)]
#![allow(private_in_public)]
#![allow(unused_variables)]

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use std::path::Path;
use serde_json::json;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id_string: String,
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
}

impl Server {
    pub async fn new(user_db_path: impl AsRef<Path>) -> Self {
        let users_db = Arc::new(UserDatabase::new(user_db_path));
        Self {users_db}
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        loop {
            let (mut socket, _) = listener.accept().await?;
            let users_db = Arc::clone(&self.users_db);

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
                            let response = process_request(request, users_db.clone()).await;
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

async fn process_request(request: Value, users_db: Arc<UserDatabase>) -> Value {
    match request["action"].as_str() {
        Some("add_user") => {
            let id_string = request["id_string"].as_str().unwrap().to_string();

            println!("- Adding user");
            let user = User {id_string};
            // Load users from the JSON file
            let mut users = users_db.load().await.unwrap();
            // Add the new user and save the updated list
            users.push(user);
            users_db.save(&users).await.unwrap();
            println!("✓ User added");

            Value::String("✓ User added".into())
        },

        Some("check_uniqueness") => {
            let users = users_db.load().await.unwrap();
            if let Some(id_string) = request.get("id_string") {
                println!("- Checking uniqueness of the id_string");
                let user_exists = users.iter().find(|user| user.id_string == id_string.as_str().unwrap());
                if let Some(user) = user_exists {
                    println!("X Check not passed");
                    json!({ "status": "error", "message": "User with same ID found"})
                } else {
                    println!("✓ Check passed");
                    json!({ "status": "success", "message": "✓ No user with same ID"})
                }
            } else {
                json!({ "status": "error", "message": "missing ID" })
            }
        },

        _ => {
            json!({ "status": "error", "message": "invalid action" })
        },
    }
}