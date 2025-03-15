use crate::model::UndoneList;

use super::Api;
use anyhow::Result;
use serde::Serialize;
use tracing::info;
use worker::{Fetch, Headers, Method, Request, RequestInit};

pub struct Telegram {
    token: String,
    chat_id: String,
}

#[derive(Serialize)]
struct TelegramMessage<'a> {
    chat_id: &'a str,
    text: &'a str,
    parse_mode: &'a str,
}

impl Telegram {
    pub fn new(token: String, chat_id: String) -> Self {
        Self { token, chat_id }
    }

    pub async fn send(&self, message: &str) -> Result<()> {
        let url = &format!("https://api.telegram.org/bot{}/sendMessage", self.token).to_string();

        let message_body = TelegramMessage {
            chat_id: &self.chat_id,
            text: message,
            parse_mode: "MarkdownV2",
        };
        let body = serde_json::to_string(&message_body)
            .map_err(|e| worker::Error::RustError(format!("Serialization failed: {}", e)))?;
        let mut headers = Headers::new();
        headers.append("Content-Type", "application/json")?;

        let request_init = RequestInit {
            method: Method::Post,
            headers,
            body: Some(body.into()),
            ..Default::default()
        };
        let request = Request::new_with_init(&url, &request_init)?;
        let response = Fetch::Request(request).send().await?;
        info!("telegram push result: {:?}", response);
        Ok(())
    }
}

impl Api for Telegram {
    async fn push(&self, undone_list: &UndoneList) -> Result<()> {
        if undone_list.undone_list.is_empty() {
            return Ok(());
        }
        for item in &undone_list.undone_list {
            info!("pushing message: {:?}", item);
            let mut msg = String::new();
            msg.push_str("# ❤️小助手提醒你写作业啦！\n\n");

            msg.push_str(
                if let Some(course_info) = &item.course_info {
                    format!(
                        "- **课程**：{}\n- **作业**：{}\n- **DDL**：【{}】",
                        course_info.name, item.activity_name, item.end_time,
                    )
                } else {
                    format!(
                        "- **作业**：{}\n- **DDL**：【{}】",
                        item.activity_name, item.end_time,
                    )
                }
                .as_str(),
            );

            if let Some(description) = &item.description {
                msg.push_str(format!("\n\n## 详细：\n{}", description).as_str());
            }

            self.send(&msg).await?;
        }
        Ok(())
    }
}
