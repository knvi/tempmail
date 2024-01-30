use chrono::prelude::*;
use serde::{Deserialize, Deserializer};
use std::{fmt::Display, future::IntoFuture};
use rand::{thread_rng, Rng};

/// Represents an attachment sent in an email message
#[derive(Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
}

/// Represents an email message
pub struct Message {
    pub id: usize,
    pub from: String,
    pub subject: String,
    pub timestamp: DateTime<Utc>,
    pub attachments: Vec<Attachment>,
    pub body: String,
    pub text_body: String, // text-only content of the body
    pub html_body: Option<String>, // html-only content of the body
}

pub struct RawMessage {
    pub id: usize,
    pub from: String,
    pub subject: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct MessageWrapper {
    id: usize,
    from: String,
    subject: String,
    date: String,
    attachments: Vec<Attachment>,
    body: String,
    text_body: String,
    html_body: Option<String>,
}

#[derive(Deserialize)]
pub struct RawMessageWrapper {
    id: usize,
    from: String,
    subject: String,
    date: String,
}

#[derive(Clone)]
pub enum Domain {
    SecMailCom,
    SecMailOrg,
    YoggmCom,
    EsiixCom,
    XojxeCom,
    SecMailNet,
    WwjmpCom,
}

pub struct Tempmail {
    pub username: String,
    pub domain: Domain,
}

pub type TempmailError = reqwest::Error;
pub type TempmailResult<T> = Result<T, TempmailError>;

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where 
        D: Deserializer<'de>,
    {
        let wrapper: MessageWrapper = Deserialize::deserialize(deserializer)?;
        
        let timestamp = NaiveDateTime::parse_from_str(&wrapper.date, "%Y-%m-%d %H:%M:%S")
            .map(|ndt| DateTime::<Utc>::from_utc(ndt, Utc))
            .map_err(serde::de::Error::custom)?;
        
        Ok(Message { id: wrapper.id, from: wrapper.from, subject: wrapper.subject, timestamp: timestamp, attachments: wrapper.attachments, body: wrapper.body, text_body: wrapper.text_body, html_body: wrapper.html_body })
    }
}

/// I DONT CARE ABOUT USING DEPRECATED FUNCTIONS !!!!!! >:3
impl<'de> Deserialize<'de> for RawMessage  {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de> {
        let wrapper: RawMessageWrapper = Deserialize::deserialize(deserializer)?;

        let timestamp = NaiveDateTime::parse_from_str(&wrapper.date, "%Y-%m-%d %H:%M:%S")
            .map(|ndt| DateTime::<Utc>::from_utc(ndt, Utc))
            .map_err(serde::de::Error::custom)?;
        
        Ok(RawMessage { id: wrapper.id, from: wrapper.from, subject: wrapper.subject, timestamp: timestamp })
    }
}

fn random_rng() -> f64 {
    let mut rng = thread_rng();
    rng.gen_range(0.0..1.0)
}

impl Domain {
    const DOMAINS: [Domain; 7] = [
        Domain::SecMailCom,
        Domain::SecMailOrg,
        Domain::SecMailNet,
        Domain::WwjmpCom,
        Domain::XojxeCom,
        Domain::EsiixCom,
        Domain::YoggmCom,
    ];

    pub fn random() -> Self {
        Self::DOMAINS[(random_rng() * Self::DOMAINS.len() as f64).round() as usize].clone()
    }
}

impl Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Domain::SecMailCom => f.write_str("1secmail.com"),
            Domain::SecMailOrg => f.write_str("1secmail.org"),
            Domain::SecMailNet => f.write_str("1secmail.net"),
            Domain::WwjmpCom => f.write_str("wwjmp.com"),
            Domain::EsiixCom => f.write_str("esiix.com"),
            Domain::XojxeCom => f.write_str("xojxe.com"),
            Domain::YoggmCom => f.write_str("yoggm.com"),
        }
    }
}

impl Default for Domain {
    fn default() -> Self {
        Domain::SecMailCom
    }
}

const API_URL: &str = "https://www.1secmail.com/api/v1/";

/// function to do a json get req and deserialize it
async fn reqjson<T, R>(query: T) -> TempmailResult<R>
where
    T: AsRef<str>,
    R: for<'de> Deserialize<'de>,
{
    reqwest::get(format!("{}?{}", API_URL, query.as_ref()))
        .await?
        .json()
        .await
}

fn random_string(length: usize) -> String {
    let mut random_string = String::with_capacity(length);

    let chars: Vec<char> = "abcdefghijklmnopqrstuwxyzABCDEFGHIJKLMNOPQRSTUWXYZ0123456789"
        .chars()
        .collect();

    for _ in 0..length {
        let random_index = (random_rng() * chars.len() as f64) as usize;
        random_string.push(chars[random_index]);
    }

    random_string
}

impl Tempmail {
    pub fn new<U>(username: U, domain: Option<Domain>) -> Self 
    where
        U: Into<String>
    {
        Self { username: username.into(), domain: domain.unwrap_or_default() }
    }

    pub fn random() -> Self {
        let len = (10.0 + random_rng() * 40.0).floor() as usize;
        let username = random_string(len);
        let domain = Domain::random();

        Self { username: username, domain: domain }
    }

    pub async fn get_raw_messages(&self) -> TempmailResult<Vec<RawMessage>> {
        reqjson(format!("action=getMessages&login={}&domain={}", self.username, self.domain)).await
    }

    pub async fn get_messages(&self) -> TempmailResult<Vec<Message>> {
        let raw_msgs = self.get_raw_messages().await?;

        let mut msgs = Vec::new();

        for raw_msg in raw_msgs {
            let msg = self.read_raw_messsage(&raw_msg).await?;
            msgs.push(msg);
        }

        Ok(msgs)
    }

    pub async fn read_raw_messsage(&self, raw_msg: &RawMessage) -> TempmailResult<Message> {
        let mut msg: Message = reqjson(format!("action=readMesage&login={}&domain={}&id={}", self.username, self.domain, raw_msg.id)).await?;

        if let Some(html_body) = msg.html_body.clone() {
            if html_body.is_empty() {
                msg.html_body = None;
            }
        }

        Ok(msg)
    }

    /// gets attachment of a msg_id and filename
    pub async fn get_attachment<T>(&self, msg_id: usize, filename: T) -> TempmailResult<Vec<u8>>
    where
        T: AsRef<str>,
    {
        reqwest::get(format!(
            "action=download&login={}&domain={}&id={}&file={}",
            self.username,
            self.domain,
            msg_id,
            filename.as_ref()
        ))
        .await?
        .bytes()
        .await
        .map(|b| b.to_vec())
    }
}
