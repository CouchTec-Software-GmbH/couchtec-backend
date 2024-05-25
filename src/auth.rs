use std::collections::HashMap;
use serde::{Deserialize, Serialize};
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
    pub fn user_exists(&self, email: &str) -> bool {
        self.users_cache.contains_key(email)
    }
    pub fn remove_user(&mut self, email: &str) {
        self.users_cache.remove(email);
    }

    pub fn insert_user(&mut self, user: User) {
        self.users_cache.insert(user.email.clone(), user);
    }

    pub fn register(&mut self, email: String, password: String) -> User {
        let salt = Uuid::new_v4().to_string();
        let salted = format!("{}{}", password, salt);

        let mut hasher = Sha256::new();
        hasher.update(salted.as_bytes());
        let result = hasher.finalize();

        let hashed = hex::encode(result);

        let user = User {
            email: email.clone(),
            hashed,
            salt,
        };
        self.users_cache.insert(email, user.clone());
        user
    }

    pub fn hash_password(&self, password: String, salt: String) -> String {
        let salted = format!("{}{}", password, salt);
        let mut hasher = Sha256::new();
        hasher.update(salted.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn get_user(&self, email: &str) -> Option<&User> {
        self.users_cache.get(email)
    }
    pub fn sign_in(&mut self, password: String, user: User) -> Result<Uuid, &'static str> {
        let hashed = self.hash_password(password, user.salt.clone());

        if hashed != user.hashed {
            return Err("Password is incorrect");
        }
        let session_id = Uuid::new_v4();
        self.session_cache.insert(session_id, user.email.clone());
        return Ok(session_id);
    }

}
