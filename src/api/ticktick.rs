use crate::model::Task;

use super::Api;
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use tracing::info;
use worker::kv::KvStore;
use worker::{Fetch, Request};
use worker::{RequestInit, Url};
pub struct TickTick {
    client_id: String,
    client_secret: String,
    pub access_token: Option<String>,
}

impl TickTick {
    pub async fn new(client_id: String, client_secret: String, kv: KvStore) -> Self {
        let access_token = kv.get("access_token").text().await.unwrap();
        Self {
            client_id,
            client_secret,
            access_token,
        }
    }

    pub async fn login(
        &self,
        bot: &super::telegram::Telegram,
        redirect_uri: &str,
        kv: KvStore,
    ) -> Result<()> {
        let state = getrandom::u64().unwrap().to_string();
        kv.put("state", state.clone())
            .unwrap()
            .execute()
            .await
            .unwrap();

        let redirect_url = &format!(
            "https://dida365.com/oauth/authorize?scope=tasks:write&client_id={}&state={}&redirect_uri={redirect_uri}&response_type=code",
            self.client_id, state
        );

        let message = format!("请点击链接登录滴答清单：{}", redirect_url);
        bot.send(&message).await?;
        Ok(())
    }

    pub async fn auth(&self, url: Url, redirect_uri: &str, kv: KvStore) -> Result<()> {
        let code = url.query_pairs().find(|(key, _)| key == "code").unwrap().1;
        let state = url.query_pairs().find(|(key, _)| key == "state").unwrap().1;

        let saved_state = kv.get("state").text().await.unwrap();
        if let Some(saved_state) = saved_state {
            if saved_state != state {
                return Err(anyhow::anyhow!("state not match"));
            }
        } else {
            return Err(anyhow::anyhow!("state not found"));
        }

        let url = "https://dida365.com/oauth/token";
        let mut headers = worker::Headers::new();
        headers.append("Content-Type", "application/x-www-form-urlencoded")?;
        headers.append(
            "Authorization",
            &format!(
                "Basic {}",
                BASE64_STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret))
            ),
        )?;

        let body = format!(
            "code={}&grant_type=authorization_code&scope=tasks:write&redirect_uri={}",
            code, redirect_uri
        );

        let request = Request::new_with_init(
            url,
            &RequestInit {
                body: Some(body.into()),
                headers,
                method: worker::Method::Post,
                ..Default::default()
            },
        )?;

        let mut response = Fetch::Request(request).send().await?;

        let res = response.json::<serde_json::Value>().await?;

        info!("auth response: {:?}", res);
        let access_token = res["access_token"].as_str().unwrap();

        kv.put("access_token", access_token)
            .unwrap()
            .execute()
            .await
            .unwrap();
        Ok(())
    }
}

impl Api for TickTick {
    async fn push(&self, message: &crate::model::UndoneList) -> Result<()> {
        if message.undone_list.is_empty() {
            return Ok(());
        }

        for undone_item in &message.undone_list {
            let task = Task {
                title: undone_item.activity_name.clone(),
                start_date: Some(
                    chrono::Utc::now()
                        .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                        .format("%Y-%m-%dT%H:%M:%S%z")
                        .to_string(),
                ),
                due_date: {
                    Some(
                        chrono::NaiveDateTime::parse_from_str(
                            &undone_item.end_time.clone(),
                            "%Y-%m-%d %H:%M:%S",
                        )
                        .unwrap()
                        .and_local_timezone(chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                        .unwrap()
                        .format("%Y-%m-%dT%H:%M:%S%z")
                        .to_string(),
                    )
                },
                content: {
                    undone_item
                        .course_info
                        .clone()
                        .and_then(|ci| Some(format!("课程：{}\n教师：{}", ci.name, ci.teachers)))
                },
            };

            let url = "https://dida365.com/open/v1/task";

            let mut headers = worker::Headers::new();
            headers.append("Content-Type", "application/json")?;
            headers.append(
                "Authorization",
                &format!("Bearer {}", self.access_token.as_ref().unwrap()),
            )?;

            let request = Request::new_with_init(
                url,
                &RequestInit {
                    body: Some(serde_json::to_string(&task)?.into()),
                    headers,
                    method: worker::Method::Post,
                    ..Default::default()
                },
            )?;

            let response = Fetch::Request(request).send().await?;
            info!("ticktick push result: {:?}", response);
        }
        Ok(())
    }
}
