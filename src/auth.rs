use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use hex;
use chrono::{DateTime, Utc};

use crate::handlers::delete_user;

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
    session_cache: HashMap<String, SessionToken>,
    one_time_codes: HashMap<String, String>,
    pre_registered: HashMap<String, User>
}

pub struct SessionToken {
    token: Uuid,
    user_id: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_used: DateTime<Utc>,
    device_info: String,
    is_revoked: bool,
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

    pub fn session_token_valid(&self, uuid: String) -> bool {
        match self.session_cache.get(&uuid) {
            Some(token) => token.is_valid(),
            None => false
        }
    }

    pub fn register(&mut self, uuid: String) -> Result<User, &str> {
        if let Some(user) = self.pre_registered.get(&uuid) {
            self.users_cache.insert(user.clone().email, user.clone());
            return Ok(user.clone());
        }
        Err("User not found")
    }

    pub fn print_out_session_cache(&self) {
        println!("Session cache:");
        for uuid in self.session_cache.keys() {
            println!("{}",uuid);
        }
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

    pub fn login(&mut self, password: String, user: User) -> Result<Uuid, &'static str> {
        let hashed = self.hash_password(password, user.salt.clone());

        if hashed != user.hashed {
            return Err("Password is incorrect");
        }

        let session_token = SessionToken::new(user.email, "".to_string());
        let uuid = session_token.token;
        self.session_cache.insert(uuid.to_string(), session_token);
        return Ok(uuid);
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
        // self.delete_user(&email);
        self.users_cache.insert(email.to_string(), user.clone());
        user
    }

    pub fn delete_user(&mut self, email: &str) {
        self.users_cache.retain(|x, _| !email.eq(x));
    }
}

impl SessionToken {
    fn new(user_id: String, device_info: String) -> Self {
        let now = Utc::now();
        SessionToken {
            token: Uuid::new_v4(),
            user_id,
            created_at: now,
            expires_at: now + chrono::Duration::hours(24),
            last_used: now,
            device_info,
            is_revoked: false,
        }
    }

    fn is_valid(&self) -> bool {
        !self.is_revoked && self.expires_at > Utc::now()
    }

    fn update_last_used(&mut self) {
        self.last_used = Utc::now();
    }
}

