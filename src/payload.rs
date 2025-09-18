use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Generic payload wrapper for all message types
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct Payload<T> {
    pub src: String,
    pub dest: String,
    pub body: T,
}

// Generic request body with common fields
#[derive(Serialize, Deserialize, Debug)]
pub struct RequestBody<T> {
    #[serde(rename = "type")]
    pub typ: String,
    pub msg_id: u32,
    #[serde(flatten)]
    pub data: Option<T>,
}

// Generic response body with common fields
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ResponseBody<T> {
    #[serde(rename = "type")]
    pub typ: String,
    pub in_reply_to: u32,
    #[serde(flatten)]
    pub data: Option<T>,
}

pub type UnhandledMessage = HashMap<String, serde_json::Value>;
