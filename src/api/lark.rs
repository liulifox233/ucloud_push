use super::Api;
use crate::model::UndoneList;
use anyhow::Result;
use tracing::info;

pub struct Lark {
    cookie: String,
}
impl Lark {
    pub fn new(cookie: String) -> Self {
        Self { cookie }
    }
}

impl Api for Lark {
    async fn push(&self, message: &UndoneList) -> Result<()> {
        let url = "https://internal-api-lark-api.feishu.cn/passport/users/details/";
        let mut headers = worker::Headers::new();
        headers.append("Content-Type", "application/json")?;
        headers.append("Cookie", &self.cookie)?;

        let ddl_count = message.undone_list.len();

        let message = format!("拼尽全力仍有 {} 个DDL", ddl_count);

        let body = serde_json::json!({"descriptionType": 0, "description": message});

        let request = worker::Request::new_with_init(
            url,
            &worker::RequestInit {
                method: worker::Method::Put,
                headers,
                body: Some(body.to_string().into()),
                ..Default::default()
            },
        )?;

        let mut response = worker::Fetch::Request(request).send().await?;
        let res = response.text().await?;
        info!("lark push response: {:?}", res);
        Ok(())
    }
}
