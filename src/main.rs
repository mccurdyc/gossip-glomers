use anyhow::Result;
use clap::{command, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::io::{self, Read, Write};
use tracing::info;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Eval {},
}

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged, rename_all = "lowercase")]
enum Message {
    Init {
        msg_id: u8,
        node_id: String,
        node_ids: Vec<String>,
    },
    Echo {
        src: String,
        dest: String,
        body: EchoReq,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct InitResp {
    #[serde(rename = "type")]
    typ: String,
    in_reply_to: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct EchoReq {
    #[serde(rename = "type")]
    typ: String,
    msg_id: u8,
    echo: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EchoResp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: EchoRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct EchoRespBody {
    #[serde(rename = "type")]
    typ: String,
    msg_id: u8,
    in_reply_to: u8,
    echo: String,
}

#[derive(Default)]
struct Node {
    id: String, // include it as the src of any message it sends.
    node_ids: Vec<String>,
}

impl Node {
    fn new(&mut self, node_id: String, node_ids: Vec<String>) -> Self {
        Self {
            id: node_id,
            node_ids,
        }
    }
}

fn listen<T, W>(lr: T, lw: &mut W) -> Result<()>
where
    T: Read,
    W: Write,
{
    info!("Starting listen loop");
    let node: &mut Node = &mut Default::default();

    // https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html
    // from_reader will read to end of deserialized object
    let msg: Message = serde_json::from_reader(lr)?;
    eprintln!("msg: {:?}", msg);

    match msg {
        Message::Init {
            msg_id,
            node_id,
            node_ids,
        } => {
            info!(
                "received init message -> node_id: {:?}, node_ids: {:?}",
                node_id, node_ids
            );
            // If the message is an Init message, we need to actually configure
            // the node object above.
            node.new(node_id, node_ids);
            let resp = InitResp {
                typ: "init_ok".to_string(),
                in_reply_to: msg_id,
            };
            serde_json::to_writer(&mut *lw, &resp)?;
        }
        Message::Echo { src, dest, body } => {
            info!("received echo message");
            let resp = EchoResp {
                src: dest,
                dest: src,
                body: EchoRespBody {
                    typ: "echo_ok".to_string(),
                    msg_id: body.msg_id,
                    in_reply_to: body.msg_id,
                    echo: body.echo,
                },
            };
            serde_json::to_writer(&mut *lw, &resp)?;
        }
    };
    Ok(())
}

#[test]
fn listen_init_message() {
    use std::io::Cursor;
    use std::vec::Vec;

    let input = r#"{
    "type": "init",
    "msg_id": 1,
    "node_id": "n3",
    "node_ids": ["n1", "n2", "n3"]
}"#;

    let expected = r#"{"type":"init_ok","in_reply_to":1}"#;

    // Necessary to implement Read trait on BufReader for bytes
    let mut vec: Vec<u8> = Vec::new();
    let mut write_cursor = Cursor::new(&mut vec);
    let read_cursor = Cursor::new(input.as_bytes());

    // println!("reader: {:?}", read_cursor.read_to_end(&mut vec).unwrap());
    listen(read_cursor, &mut write_cursor).expect("listen failed");

    assert_eq!(String::from_utf8(vec).unwrap(), expected);
}

#[test]
fn listen_echo_message() {
    use std::io::Cursor;
    use std::vec::Vec;

    let input = r#"{
    "src": "c1",
    "dest": "n1",
    "body": {
        "type": "echo",
        "msg_id": 1,
        "echo": "Please echo 35"
    }
}"#;

    let expected = r#"{"src":"n1","dest":"c1","body":{"type":"echo_ok","msg_id":1,"in_reply_to":1,"echo":"Please echo 35"}}"#;

    // Necessary to implement Read trait on BufReader for bytes
    let mut vec: Vec<u8> = Vec::new();
    let mut write_cursor = Cursor::new(&mut vec);
    let read_cursor = Cursor::new(input.as_bytes());

    listen(read_cursor, &mut write_cursor).expect("listen failed");

    assert_eq!(String::from_utf8(vec).unwrap(), expected);
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Starting lodiseval");
    match &cli.command {
        Some(Commands::Eval {}) => listen(stdin, &mut stdout)?,
        None => (),
    }

    Ok(())
}
