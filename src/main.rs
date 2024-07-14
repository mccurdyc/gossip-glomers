use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::io::{self, BufRead, Write};
use tracing::info;
use tracing_subscriber::prelude::*;

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
    T: BufRead,
    W: Write,
{
    info!("Starting listen loop");
    let node: &mut Node = &mut Default::default();

    // Loop messages
    // https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html#a-more-efficient-approach
    for line in lr.lines() {
        if let Ok(l) = line {
            let msg: Message = serde_json::from_str(&l)?;
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
    }
    Ok(())
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
