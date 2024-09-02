use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct Payload {
    pub src: String,
    pub dest: String,
    pub body: ReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqBody {
    #[serde(rename = "type")]
    pub typ: String,
    pub msg_id: u32,
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Resp {
    pub src: String,
    pub dest: String,
    // You can't nest structures in Rust for ownership reasons.
    pub body: RespBody,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RespBody {
    #[serde(rename = "type")]
    pub typ: String,
    pub in_reply_to: u32,
}
