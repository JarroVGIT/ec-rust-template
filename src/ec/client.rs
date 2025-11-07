use aes::Aes256;
use aes::cipher::BlockDecryptMut;
use block_padding::Pkcs7;
use cbc::{Decryptor, cipher::KeyIvInit};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

type Aes256CbcDec = Decryptor<Aes256>;

const BASE_URL: &str = "https://everybody.codes";
const CDN_URL: &str = "https://everybody-codes.b-cdn.net";

#[derive(Debug)]
pub enum ClientError {
    SessionNotFound,
    SeedNotConfigured,
    EventNotConfigured,
    HttpError(String),
    DecryptionError(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::IoError(err)
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::SessionNotFound => {
                write!(
                    f,
                    "Session cookie not found. Please create ~/.ec-session with your session cookie value."
                )
            }
            ClientError::SeedNotConfigured => {
                write!(
                    f,
                    "EC_SEED not configured. Add it to .cargo/config.toml:\n[env]\nEC_SEED = \"your_seed_here\""
                )
            }
            ClientError::EventNotConfigured => {
                write!(
                    f,
                    "EC_EVENT not configured. Add it to .cargo/config.toml:\n[env]\nEC_EVENT = \"2024\""
                )
            }
            ClientError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            ClientError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            ClientError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for ClientError {}

#[derive(Deserialize)]
struct UserResponse {
    seed: u32,
}

#[derive(Deserialize)]
struct QuestResponse {
    #[serde(rename = "key1")]
    part1_key: Option<String>,
    #[serde(rename = "key2")]
    part2_key: Option<String>,
    #[serde(rename = "key3")]
    part3_key: Option<String>,
}

#[derive(Serialize)]
struct AnswerPayload {
    answer: String,
}

pub struct Client {
    session: String,
    seed: u32,
    event: String,
    http_client: reqwest::blocking::Client,
}

impl Client {
    pub fn new() -> Result<Self, ClientError> {
        let session = Self::read_session()?;
        let event = Self::get_event()?;

        let http_client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        // Check if seed needs to be fetched
        let seed = match Self::get_seed() {
            Ok(s) => s,
            Err(_) => {
                // Seed not configured or empty, fetch it from API
                let temp_client = Client {
                    session: session.clone(),
                    seed: 0, // Temporary value
                    event: event.clone(),
                    http_client: http_client.clone(),
                };
                let fetched_seed = temp_client.fetch_user_seed()?;
                println!(
                    "\n INFO: Fetched your seed from the API: {}. You can ",
                    fetched_seed
                );
                println!("You can add this to .cargo/config.toml to avoid fetching it each time:");
                println!();
                fetched_seed
            }
        };

        Ok(Self {
            session,
            seed,
            event,
            http_client,
        })
    }

    fn read_session() -> Result<String, ClientError> {
        let home = std::env::var("HOME").map_err(|_| ClientError::SessionNotFound)?;
        let session_path = PathBuf::from(home).join(".ec-session");

        if !session_path.exists() {
            return Err(ClientError::SessionNotFound);
        }

        let session = fs::read_to_string(session_path)?.trim().to_string();

        Ok(session)
    }

    fn get_seed() -> Result<u32, ClientError> {
        let seed_str = std::env::var("EC_SEED").map_err(|_| ClientError::SeedNotConfigured)?;

        // Check if it's just whitespace or empty
        if seed_str.trim().is_empty() {
            return Err(ClientError::SeedNotConfigured);
        }

        seed_str
            .trim()
            .parse()
            .map_err(|_| ClientError::SeedNotConfigured)
    }

    fn get_event() -> Result<String, ClientError> {
        std::env::var("EC_EVENT").map_err(|_| ClientError::EventNotConfigured)
    }

    pub fn fetch_user_seed(&self) -> Result<u32, ClientError> {
        let url = format!("{}/api/user/me", BASE_URL);
        let response = self
            .http_client
            .get(&url)
            .header("Cookie", format!("everybody-codes={}", self.session))
            .send()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ClientError::HttpError(format!(
                "Status: {}",
                response.status()
            )));
        }

        let user: UserResponse = response
            .json()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        Ok(user.seed)
    }

    pub fn fetch_encrypted_input(&self, quest: u8, part: u8) -> Result<String, ClientError> {
        let url = format!(
            "{}/assets/{}/{}/input/{}.json",
            CDN_URL, self.event, quest, self.seed
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ClientError::HttpError(format!(
                "Status: {}",
                response.status()
            )));
        }

        let inputs: serde_json::Value = response
            .json()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        let encrypted = inputs
            .get(part.to_string())
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ClientError::HttpError(format!("Part {} not found in response", part))
            })?;

        Ok(encrypted.to_string())
    }

    pub fn fetch_decryption_key(&self, quest: u8, part: u8) -> Result<String, ClientError> {
        let url = format!("{}/api/event/{}/quest/{}", BASE_URL, self.event, quest);

        let response = self
            .http_client
            .get(&url)
            .header("Cookie", format!("everybody-codes={}", self.session))
            .send()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ClientError::HttpError(format!(
                "Status: {}",
                response.status()
            )));
        }

        let quest_data: QuestResponse = response
            .json()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        let key = match part {
            1 => quest_data.part1_key,
            2 => quest_data.part2_key,
            3 => quest_data.part3_key,
            _ => None,
        };

        key.ok_or_else(|| {
            ClientError::HttpError(format!(
                "Key for part {} not available (possibly not solved yet)",
                part
            ))
        })
    }

    pub fn decrypt_input(&self, encrypted_hex: &str, key: &str) -> Result<String, ClientError> {
        let encrypted_bytes =
            hex::decode(encrypted_hex).map_err(|e| ClientError::DecryptionError(e.to_string()))?;

        let key_bytes = key.as_bytes();
        if key_bytes.len() != 32 {
            return Err(ClientError::DecryptionError(format!(
                "Key must be 32 bytes, got {}",
                key_bytes.len()
            )));
        }

        let iv_bytes = &key_bytes[..16];

        let mut encrypted_clone = encrypted_bytes.clone();
        let decrypted = Aes256CbcDec::new(key_bytes.into(), iv_bytes.into())
            .decrypt_padded_mut::<Pkcs7>(&mut encrypted_clone)
            .map_err(|e| ClientError::DecryptionError(format!("Decryption failed: {:?}", e)))?;

        String::from_utf8(decrypted.to_vec())
            .map_err(|e| ClientError::DecryptionError(e.to_string()))
    }

    pub fn fetch_and_decrypt_input(&self, quest: u8, part: u8) -> Result<String, ClientError> {
        let encrypted = self.fetch_encrypted_input(quest, part)?;
        let key = self.fetch_decryption_key(quest, part)?;
        self.decrypt_input(&encrypted, &key)
    }

    pub fn submit_answer(&self, quest: u8, part: u8, answer: &str) -> Result<String, ClientError> {
        let url = format!(
            "{}/api/event/{}/quest/{}/part/{}/answer",
            BASE_URL, self.event, quest, part
        );

        let payload = AnswerPayload {
            answer: answer.to_string(),
        };

        let response = self
            .http_client
            .post(&url)
            .header("Cookie", format!("everybody-codes={}", self.session))
            .json(&payload)
            .send()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .map_err(|e| ClientError::HttpError(e.to_string()))?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(ClientError::HttpError(format!(
                "Submit failed ({}): {}",
                status, body
            )))
        }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn event(&self) -> &str {
        &self.event
    }
}
