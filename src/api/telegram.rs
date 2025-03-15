use crate::model::UndoneList;

use super::Api;
use anyhow::Result;
use tracing::info;
use worker::{Fetch, Request};

pub struct Telegram {
    token: String,
    chat_id: String,
}

impl Telegram {
    pub fn new(token: String, chat_id: String) -> Self {
        Self { token, chat_id }
    }

    pub async fn send(&self, message: &str) -> Result<()> {
        let url = &format!(
            "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
            self.token,
            self.chat_id,
            urlencoding::encode(message)
        )
        .to_string();

        let request = Request::new(&url, worker::Method::Get).unwrap();
        let response = Fetch::Request(request).send().await?;
        info!("telegram push result: {:?}", response);
        Ok(())
    }
}

impl Api for Telegram {
    async fn push(&self, message: &UndoneList) -> Result<()> {
        if message.undone_list.is_empty() {
            return Ok(());
        }
        let mut msg = String::new();
        msg.push_str("【❤️小助手提醒你写作业啦！】\n\n");
        message.undone_list.clone().into_iter().for_each(|item| {
            msg.push_str(
                if let Some(course_info) = &item.course_info {
                    format!(
                        "【{}】 【{}】\nDDL：【{}】\n\n",
                        course_info.name, item.activity_name, item.end_time,
                    )
                } else {
                    format!("【{}】\nDDL：【{}】\n\n", item.activity_name, item.end_time)
                }
                .as_str(),
            );
        });

        self.send(&msg).await?;
        Ok(())
    }
}
