use std::collections::HashMap;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use hex;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub hashed: String,
    pub salt: String,
}

pub struct UserManager {
    users_cache: HashMap<String, User>,
    session_cache: HashMap<Uuid, String>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            users_cache: HashMap::new(),
            session_cache: HashMap::new(),
        }
    }
    pub fn register(&mut self, username: String, password: String) {
        let salt = Uuid::new_v4().to_string();
        let salted = format!("{}{}", password, salt);

        let mut hasher = Sha256::new();
        hasher.update(salted.as_bytes());
        let result = hasher.finalize();

        let hashed = hex::encode(result);
        let user_id = Uuid::new_v4().to_string();

        let user = User {
            username: username.clone(),
            hashed,
            salt,
            user_id,
        };

        self.users.insert(username, user);
    }

    pub fn sign_in(&self, username: String, password: String) -> Result<Uuid, &'static str> {
        let user = self.users.get(&username);

        if let Some(user) = user {
            let salted = format!("{}{}", password, user.salt);
            let mut hasher = Sha256::new();
            hasher.update(salted.as_bytes());
            let result = hasher.finalize();
            let hashed = hex::encode(result);

            if hashed == user.hashed {
                let session_id = Uuid::new_v4();
                return Ok(session_id);
            } else {
                return Err("Password is incorrect");
            }
        } else {
            return Err("User not found");
        }
    }
}
