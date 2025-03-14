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
}

impl Api for Telegram {
    async fn push(&self, message: &UndoneList) -> Result<()> {
        let mut msg = String::new();
        message.undone_list.iter().for_each(|item| {
            msg.push_str(
                &urlencoding::encode(
                    format!(
                        "Activity: {}\nEnd Time: {}\n\n",
                        item.activity_name, item.end_time
                    )
                    .as_str(),
                )
                .into_owned(),
            );
        });

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
            self.token, self.chat_id, msg
        );

        let request = Request::new(&url, worker::Method::Get).unwrap();
        info!("telegram push url: {:?}", url);
        let response = Fetch::Request(request).send().await?;
        info!("telegram push result: {:?}", response);

        Ok(())
    }
}
