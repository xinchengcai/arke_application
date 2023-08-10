// server.rs
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
}

impl Server {
    pub async fn new(user_db_path: impl AsRef<Path>) -> Self {
        let users_db = Arc::new(UserDatabase::new(user_db_path));

        Self { users_db,
        }
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
                        Ok(n) if n == 0 => return,  // client closed connection
                        Ok(n) => {
                            let request: Value = serde_json::from_slice(&buf[..n]).unwrap();
                            // handle request here
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
            let unread: bool = request["unread"].as_bool().unwrap();
            let session = request["session"].as_str().unwrap().to_string();
            let user = User {id_string, unread, session};

            // Load users from the JSON file
            let mut users = users_db.load().await.unwrap();

            // Add the new user and save the updated list
            users.push(user);
            users_db.save(&users).await.unwrap();

            Value::String("✓ User added".into())
        },


        Some("check_uniqueness") => {
            let users = users_db.load().await.unwrap();
            if let Some(id_string) = request.get("id_string") {
                let user_exists = users.iter().find(|user| user.id_string == id_string.as_str().unwrap());
                if let Some(user) = user_exists {
                    json!({ "status": "error", "message": "User with same ID found"})
                } else {
                    json!({ "status": "success", "message": "✓ No user with same ID"})
                }
            } else {
                json!({ "status": "error", "message": "missing ID" })
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

        _ => {
            json!({ "status": "error", "message": "invalid action" })
        },
    }
}