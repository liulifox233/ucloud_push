use crate::model::UndoneList;

use super::Api;
use anyhow::Result;
use html5tokenizer::{NaiveParser, Token};
use serde::Serialize;
use tracing::info;
use worker::{Fetch, Headers, Method, Request, RequestInit};

pub struct Telegram {
    token: String,
    chat_id: String,
}

#[derive(Serialize, Debug)]
struct TelegramMessage<'a> {
    chat_id: &'a str,
    text: &'a str,
    parse_mode: &'a str,
}

impl Telegram {
    pub fn new(token: String, chat_id: String) -> Self {
        Self { token, chat_id }
    }

    pub async fn send_message(&self, message: &str) -> Result<()> {
        let url = &format!("https://api.telegram.org/bot{}/sendMessage", self.token).to_string();

        let message_body = TelegramMessage {
            chat_id: &self.chat_id,
            text: message,
            parse_mode: "HTML",
        };
        let body = serde_json::to_string(&message_body)
            .map_err(|e| worker::Error::RustError(format!("Serialization failed: {}", e)))?;
        let mut headers = Headers::new();
        headers.append("Content-Type", "application/json")?;

        info!("message: {:?}", message_body);

        let request_init = RequestInit {
            method: Method::Post,
            headers,
            body: Some(body.into()),
            ..Default::default()
        };
        let request = Request::new_with_init(&url, &request_init)?;
        let mut response = Fetch::Request(request).send().await?;
        let res = response.json::<serde_json::Value>().await?;
        if res["ok"].as_bool().unwrap() {
            info!("telegram push success: {:?}", res);
        } else {
            info!("telegram push failed: {:?}", res);
        }
        Ok(())
    }

    pub async fn send_media_group(&self, media_urls: Vec<String>, caption: &str) -> Result<()> {
        let url = &format!("https://api.telegram.org/bot{}/sendMediaGroup", self.token).to_string();

        let mut media_group = media_urls
            .into_iter()
            .map(|url| {
                serde_json::json!({
                    "type": "photo",
                    "media": url,
                })
            })
            .collect::<Vec<_>>();

        if let Some(first) = media_group.first_mut() {
            if let Some(first_object) = first.as_object_mut() {
                first_object.insert("parse_mode".to_string(), "HTML".into());
                first_object.insert("caption".to_string(), caption.into());
            }
        }

        let message_body = serde_json::json!({
            "chat_id": self.chat_id,
            "media": media_group,
        });

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
        let mut response = Fetch::Request(request).send().await?;
        let res = response.json::<serde_json::Value>().await?;
        if res["ok"].as_bool().unwrap() {
            info!("telegram push success: {:?}", res);
        } else {
            info!("telegram push failed: {:?}", res);
        }
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
            msg.push_str("<b>❤️小助手提醒你写作业啦！</b>\n\n");
            let (description, image_urls) =
                filter_and_extract_image(item.description.as_ref().unwrap_or(&String::new()));

            msg.push_str(
                if let Some(course_info) = &item.course_info {
                    format!(
                        "<b>课程</b>：{}\n<b>作业</b>：{}\n<b>开始时间</b>：{}\n<b>结束时间</b>：{}\n<b>能否补交</b>：{}",
                        course_info.name,
                        item.activity_name,
                        item.start_time.as_ref().unwrap(),
                        item.end_time,
                        if item.is_overtime_commit.unwrap() {
                            "能"
                        } else {
                            "否"
                        }

                    )
                } else {
                    format!(
                        "<b>作业</b>：{}\n<b>开始时间</b>：{}\n<b>结束时间</b>：{}\n<b>能否补交</b>：{}",
                        item.activity_name,
                        item.start_time.as_ref().unwrap(),
                        item.end_time,
                        if item.is_overtime_commit.unwrap() {
                            "能"
                        } else {
                            "否"
                        }
                    )
                }
                .as_str(),
            );

            if !description.trim().is_empty() {
                msg.push_str(format!("\n\n<b>详细：</b>\n\n{}", description.trim()).as_str());
            }

            if image_urls.is_empty() {
                self.send_message(&msg).await?;
            } else {
                self.send_media_group(image_urls, &msg).await?;
            }
        }
        Ok(())
    }
}

fn filter_and_extract_image(html: &str) -> (String, Vec<String>) {
    let allowed_tags = vec![
        "b", "strong", "i", "em", "u", "ins", "s", "strike", "del", "a", "code", "pre",
    ];
    let mut new_html = String::new();
    let mut image_urls = Vec::new();
    for token in NaiveParser::new(html).flatten() {
        match token {
            Token::StartTag(tag) => {
                if tag.name == "img" {
                    if let Some(src) = tag.attributes.get("src") {
                        image_urls.push(src.to_owned());
                    }
                    continue;
                }
                if !allowed_tags.contains(&tag.name.as_str()) {
                    continue;
                }
                new_html.push_str(&format!("<{}>", tag.name));
            }
            Token::Char(c) => {
                new_html.push(c);
            }
            Token::EndTag(tag) => {
                if tag.name == "p" || tag.name == "br" {
                    new_html.push_str("\n");
                    continue;
                }
                if !allowed_tags.contains(&tag.name.as_str()) {
                    continue;
                }
                new_html.push_str(&format!("</{}>", tag.name));
            }
            Token::EndOfFile => {}
            _ => panic!("unexpected input"),
        }
    }
    (new_html, image_urls)
}
