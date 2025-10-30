use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct Payload<T> {
    pub src: String,
    pub dest: String,
    pub body: T,
}

// Implement Serialize only when T is Serialize
impl<T> Serialize for Payload<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Payload", 3)?;
        state.serialize_field("src", &self.src)?;
        state.serialize_field("dest", &self.dest)?;
        state.serialize_field("body", &self.body)?;
        state.end()
    }
}

// Implement Deserialize only when T is DeserializeOwned
impl<'de, T> Deserialize<'de> for Payload<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PayloadHelper<T> {
            src: String,
            dest: String,
            body: T,
        }

        let helper = PayloadHelper::<T>::deserialize(deserializer)?;
        Ok(Payload {
            src: helper.src,
            dest: helper.dest,
            body: helper.body,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub struct RequestBody<T> {
    pub msg_id: u32,
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub struct ResponseBody<T> {
    #[serde(rename = "type")]
    pub typ: String,
    pub in_reply_to: u32,
    #[serde(flatten)]
    pub data: T,
}

pub type UnhandledMessage = std::collections::HashMap<String, serde_json::Value>;
