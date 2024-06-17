use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use hex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub newsletter: bool,
    pub hashed: String,
    pub salt: String,
    pub uuids: Vec<String>
}

pub struct UserManager {
    users_cache: HashMap<String, User>,
    session_cache: HashMap<String, String>,
    one_time_codes: HashMap<String, String>,
    pre_registered: HashMap<String, User>
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            users_cache: HashMap::new(),
            session_cache: HashMap::new(),
            one_time_codes: HashMap::new(),
            pre_registered: HashMap::new()
        }
    }
    pub fn user_exists(&self, email: &str) -> bool {
        self.users_cache.contains_key(email)
    }

    pub fn get_user(&self, email: &str) -> Option<&User> {
        self.users_cache.get(email)
    }


    pub fn remove_user(&mut self, email: &str) {
        self.users_cache.remove(email);
    }

    pub fn insert_user(&mut self, user: User) {
        self.users_cache.insert(user.email.clone(), user);
    }

    pub fn get_email_from_session_token(&self, uuid: String) -> Option<String> {
        return self.session_cache.get(&uuid).cloned();
    }

    pub fn register(&mut self, uuid: String) -> Result<User, &str> {
        if let Some(user) = self.pre_registered.get(&uuid) {
            self.users_cache.insert(user.clone().email, user.clone());
            return Ok(user.clone());
        }
        Err("User not found")
    }

    pub fn pre_register(&mut self, email: String, password: String, newsletter: bool) -> String {
        let salt = Uuid::new_v4().to_string();
        let salted = format!("{}{}", password, salt);

        let mut hasher = Sha256::new();
        hasher.update(salted.as_bytes());
        let result = hasher.finalize();

        let hashed = hex::encode(result);

        let user = User {
            email: email.clone(),
            newsletter,
            hashed,
            salt,
            uuids: Vec::new(),
        };
        let uuid = Uuid::new_v4().to_string();
        self.pre_registered.insert(uuid.clone(), user);
        uuid
    }

    pub fn hash_password(&self, password: String, salt: String) -> String {
        let salted = format!("{}{}", password, salt);
        let mut hasher = Sha256::new();
        hasher.update(salted.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn sign_in(&mut self, password: String, user: User) -> Result<Uuid, &'static str> {
        let hashed = self.hash_password(password, user.salt.clone());

        if hashed != user.hashed {
            return Err("Password is incorrect");
        }
        let session_id = Uuid::new_v4();
        self.session_cache.insert(session_id.to_string(), user.email.clone());
        return Ok(session_id);
    }

    pub fn logout(&mut self, uuid: String) {
        self.session_cache.retain(|x, _| !x.eq(&uuid));
    }

    pub fn insert_reset_email_code(&mut self, email: String) -> String {
        let one_time_code = Uuid::new_v4().to_string();
        self.one_time_codes.insert(one_time_code.clone(), email.clone());
        one_time_code
    }

    pub fn get_email_from_code(&self, uuid: &str) -> Option<String> {
        self.one_time_codes.get(uuid).cloned()
    }

    pub fn change_password(&mut self, email: &str, password: &str, newsletter: bool) -> User {
        let salt = Uuid::new_v4().to_string();
        let salted = format!("{}{}", password, salt);

        let mut hasher = Sha256::new();
        hasher.update(salted.as_bytes());
        let result = hasher.finalize();

        let hashed = hex::encode(result);
        let user = User {
            email: email.to_string(),
            newsletter,
            hashed,
            salt,
            uuids: Vec::new(),
        };
        self.users_cache.insert(email.to_string(), user.clone());
        user
    }

    pub fn delete_user(&mut self, email: &str) {
        self.users_cache.retain(|x, _| !email.eq(x));
    }
}

