use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::auth::User;


#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(rename = "_rev")]
    pub rev: Option<String>,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPayload {
    pub email: String,
    pub newsletter: bool,
    pub hashed: String,
    pub salt: String,
    pub uuids: Vec<String>,
    #[serde(rename = "_rev")]
    rev: String,
}

pub struct CouchDB {
    client: Client,
    url: String,
    auth: (String, String)
}



impl CouchDB {
    pub fn new(url: String, username: String, password: String) -> Self {
        CouchDB {
            client: Client::new(),
            url,
            auth: (username, password)
        }
    }

    pub async fn get_document(&self, id: &str) -> Result<Document, reqwest::Error> {
        let url = format!("{}/projects/{}", self.url, id);
        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .basic_auth(&self.auth.0, Some(&self.auth.1))
            .send()
        .await?;


        let response = response.error_for_status()?;
        let document: Document = response.json().await?;
        Ok(document)
    }

    pub async fn put_document(&self, id: &str, data: Value) -> Result<Document, reqwest::Error> {
        let url = format!("{}/projects/{}", self.url, id);
        match self.get_document(id).await {
            Ok(doc) => {
                let updated_doc = Document {
                    id: doc.id.clone(),
                    rev: doc.rev.clone(),
                    data,
                };
                let response = self
                    .client
                    .put(&url)
                    .header("Content-Type", "application/json")
                    .basic_auth(&self.auth.0, Some(&self.auth.1))
                    .json(&updated_doc)
                    .send()
                    .await?;

                let _response = response.error_for_status()?;
                Ok(updated_doc)
            }
            Err(e) if e.status() == Some(reqwest::StatusCode::NOT_FOUND) => {
                let new_doc = NewDocument {
                    id: id.to_string(),
                    data,
                };
                let response = self
                    .client
                    .put(&url)
                    .header("Content-Type", "application/json")
                    .basic_auth(&self.auth.0, Some(&self.auth.1))
                    .json(&new_doc)
                    .send()
                    .await?;

                let _response = response.error_for_status()?;
                let document: Document = self.get_document(id).await?;
                Ok(document)
            }
            Err(e) => return Err(e),
        }
    }


    pub async fn put_user(&self, user: User) -> Result<User , reqwest::Error> {
        let url = format!("{}/users/{}", self.url, user.email);
        match self.get_user_payload(&user.email).await {
            Ok(user_payload) => {
                let updated_user = UserPayload {
                    email: user.email.clone(),
                    newsletter: user.newsletter.clone(),
                    hashed: user_payload.hashed.clone(),
                    salt: user_payload.salt.clone(),
                    uuids: user.uuids.clone(),
                    rev: user_payload.rev.clone(),
                };
                println!("Updated user: {:?}", updated_user);
                let response = self
                    .client
                    .put(&url)
                    .header("Content-Type", "application/json")
                    .basic_auth(&self.auth.0, Some(&self.auth.1))
                    .json(&updated_user)
                    .send()
                    .await?;

                let _response = response.error_for_status()?;
                Ok(user)
            }
        Err(_)  => {
                println!("404");
            let new_user = User {
                email: user.email.clone(),
                newsletter: user.newsletter.clone(),
                hashed: user.hashed.clone(),
                salt: user.salt.clone(),
                uuids: user.uuids.clone(),
            };
            let response = self
                .client
                .put(&url)
                .header("Content-Type", "application/json")
                .basic_auth(&self.auth.0, Some(&self.auth.1))
                .json(&new_user)
                .send()
                .await?;

            let _response = response.error_for_status()?;
            let user: User = self.get_user(&user.email).await?;
            Ok(user)
        }
        }
    }

    pub async fn get_user_payload(&self, email: &str) -> Result<UserPayload, reqwest::Error> {
        let url = format!("{}/users/{}", self.url, email);
        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .basic_auth(&self.auth.0, Some(&self.auth.1))
            .send()
            .await?;

        let user: UserPayload = response.json().await?;
        println!("User: {:?}", user);
        Ok(user)
    }

    pub async fn get_user(&self, email: &str) -> Result<User, reqwest::Error> {
        let url = format!("{}/users/{}", self.url, email);
        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .basic_auth(&self.auth.0, Some(&self.auth.1))
            .send()
            .await?;

        let response = response.error_for_status()?;
        let user: User = response.json().await?;
        Ok(user)
    }
}
