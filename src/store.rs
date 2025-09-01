use fs2::FileExt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read, Seek, SeekFrom, Write};
use std::path::Path;

// std::io::{Read,Write} Supertrait
pub trait Store: Write + Read + BufRead + Seek {}

#[derive(Debug)]
pub struct MemoryStore {
    buf: Vec<u8>,
    position: usize,
}

impl MemoryStore {
    pub fn new(v: Vec<u8>) -> Result<Self, std::io::Error> {
        Ok(Self {
            buf: v,
            position: 0,
        })
    }
}

impl Store for MemoryStore {}
impl Write for MemoryStore {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        for v in buf {
            self.buf.push(*v);
        }
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        let _ = self.write(buf)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        // Does nothing
        Ok(())
    }
}

impl Read for MemoryStore {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // Calculate how many bytes can be read
        let bytes_to_read = std::cmp::min(buf.len(), self.buf.len() - self.position);

        if bytes_to_read == 0 {
            return Ok(0); // No more data to read
        }

        // Copy the data into the buffer
        buf[..bytes_to_read]
            .copy_from_slice(&self.buf[self.position..self.position + bytes_to_read]);

        // Update the position
        self.position += bytes_to_read;

        Ok(bytes_to_read) // Return the number of bytes read
    }
}

impl BufRead for MemoryStore {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(&self.buf[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position += amt;
    }
}

// TODO: is there a better way?
impl Seek for MemoryStore {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}

#[derive(Debug)]
pub struct FileStore {
    // File does not buffer reads and writes. For efficiency, consider wrapping the file in a BufReader or BufWriter
    // when performing many small read or write calls, unless unbuffered reads and writes are required.
    pub file: File,
    // The BufReader<R> struct adds buffering to any reader.
    // It can be excessively inefficient to work directly with a Read instance.
    // For example, every call to read on TcpStream results in a system call.
    // A BufReader<R> performs large, infrequent reads on the underlying Read and maintains an in-memory buffer of the
    // results.
    inner: BufReader<File>, // Not Copy-safe.
}

impl Store for FileStore {}
impl FileStore {
    pub fn new(p: &Path) -> Result<Self, std::io::Error> {
        // We have two separate file descriptors: one for writing
        // and one for reading. Primarily to avoid having to reset seek, etc.
        let w = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)?;

        let r = std::fs::OpenOptions::new().read(true).open(p)?;

        let inner = BufReader::new(r);
        Ok(FileStore { file: w, inner })
    }
}

impl Write for FileStore {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.file.lock_exclusive()?;
        let s = self.file.write(buf)?;
        fs2::FileExt::unlock(&self.file)?;

        Ok(s)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.file.lock_exclusive()?;
        self.file.write_all(buf)?;
        fs2::FileExt::unlock(&self.file)?;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.file.lock_exclusive()?;
        self.file.flush()?;
        fs2::FileExt::unlock(&self.file)?;
        Ok(())
    }
}

impl Read for FileStore {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.file.lock_exclusive()?;
        let u = self.inner.read(buf);
        fs2::FileExt::unlock(&self.file)?;

        u
    }
}

// Inspiration - https://doc.rust-lang.org/src/std/io/stdio.rs.html#546
impl BufRead for FileStore {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.inner.read_until(byte, buf)
    }

    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.inner.read_line(buf)
    }
}

impl Seek for FileStore {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        // seeks the file cursor for writing
        self.file.seek(pos)?;
        // seeks the read cursor
        self.inner.seek(pos)
    }
}
