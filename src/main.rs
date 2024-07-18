use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};
use std::default::Default;
use std::io::{self, BufRead, BufReader, Read, Write};
use tracing::{error, info};

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

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    Init {
        #[serde(alias = "type")]
        typ: String,
        msg_id: u8,
        node_id: String,
        node_ids: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct InitResp {
    #[serde(alias = "type")]
    typ: String,
    in_reply_to: u8,
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

fn listen<T, W>(lr: &mut T, lw: &mut W) -> Result<()>
where
    T: Read,
    W: Write,
{
    info!("Starting listen loop");
    let node: &mut Node = &mut Default::default();

    let buf_reader = BufReader::new(lr);
    let stream = Deserializer::from_reader(buf_reader).into_iter::<Message>();

    // Loop messages
    // https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html#a-more-efficient-approach
    for l in stream {
        match l {
            Ok(msg) => {
                println!("Decoded message: {:?}", msg);
                match msg {
                    Message::Init {
                        typ: _,
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
                }
            }
            Err(e) => {
                error!("Error reading message: {}", e);
            }
        }
    }

    Ok(())
}

#[test]
fn listen_test() {
    use std::io::{BufReader, Cursor, Read};
    use std::vec::Vec;

    let input = b"
        {
        \"type\": \"init\",
        \"msg_id\": 1,
        \"node_id\": 'n3',
        \"node_ids\": [\"n1\", \"n2\", \"n3\"]
        }"
    .to_vec();

    let expected = b"{
  \"type\": \"init_ok\",
  \"in_reply_to\": 1
}";

    // Necessary to implement Read trait on BufReader for bytes
    let vec: Vec<u8> = Vec::new();
    let mut write_cursor = Cursor::new(vec);

    let read_cursor = Cursor::new(input);
    let mut line_reader = BufReader::new(read_cursor);

    listen(&mut line_reader, &mut write_cursor).expect("listen failed");

    let mut actual: Vec<u8> = Vec::new();
    write_cursor.read(actual.as_mut_slice()).unwrap();

    assert_eq!(actual, expected);
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Starting lodiseval");
    match &cli.command {
        Some(Commands::Eval {}) => listen(&mut stdin, &mut stdout)?,
        None => (),
    }

    Ok(())
}
