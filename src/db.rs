use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;


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

}
