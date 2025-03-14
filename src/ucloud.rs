use crate::model;
use anyhow::Result;

pub struct UCloud {
    username: String,
    password: String,
    api_url: String,
    client: reqwest::Client,
}

impl UCloud {
    pub fn new(username: String, password: String, api_url: String) -> Self {
        Self {
            username,
            password,
            api_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_undone_list(&self) -> Result<model::UndoneList> {
        let undone_list = self
            .client
            .get(&format!("{}/undoneList", self.api_url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .json()
            .await?;
        Ok(undone_list)
    }
}
