use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;

// Generic payload wrapper for all message types
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
#[serde(rename_all = "lowercase")]
pub struct Payload<T>
where
    T: DeserializeOwned + Send,
{
    pub src: String,
    pub dest: String,
    pub body: T,
}

// Generic request body with common fields
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct RequestBody<T>
where
    T: DeserializeOwned + Send,
{
    #[serde(rename = "type")]
    pub typ: String,
    pub msg_id: u32,
    #[serde(flatten)]
    pub data: Option<T>,
}

// Generic response body with common fields
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct ResponseBody<T>
where
    T: Serialize + DeserializeOwned + Send,
{
    #[serde(rename = "type")]
    pub typ: String,
    pub in_reply_to: u32,
    #[serde(flatten)]
    pub data: Option<T>,
}

pub type UnhandledMessage = HashMap<String, serde_json::Value>;
