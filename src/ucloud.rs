use crate::model::{self, Detail, UndoneList};
use anyhow::Result;
use htmd::HtmlToMarkdown;

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
        let mut undone_list: UndoneList = self
            .client
            .get(&format!("{}/undoneList", self.api_url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .json()
            .await?;

        let converter = HtmlToMarkdown::builder().build();

        for item in &mut undone_list.undone_list {
            let detail = self.get_detail(&item.activity_id).await?;
            let desc = converter
                .convert(&detail.assignment_content)
                .unwrap()
                .replace("![](", "\n![](");
            item.description = Some(desc);
            item.start_time = Some(detail.assignment_begin_time);
        }
        Ok(undone_list)
    }

    pub async fn get_detail(&self, id: &str) -> Result<Detail> {
        let detail = self
            .client
            .get(&format!("{}/homework?id={}", self.api_url, id))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .json()
            .await?;
        Ok(detail)
    }
}
