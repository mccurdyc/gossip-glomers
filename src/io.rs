use anyhow::Result;
use serde_json;
use std::fmt::Debug;
use std::io::Write;
use tracing::info;

pub fn to_writer<W: Write, T: serde::Serialize + Debug>(writer: &mut W, m: T) -> Result<()> {
    serde_json::to_writer(&mut *writer, &m)?;
    writeln!(writer)?; // adds a newline
    info!("<< output: {:?}", &m);
    Ok(())
}
