use fs2::FileExt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read, Write};

// std::io::{Read,Write} Supertrait
pub trait Store: Write + Read + BufRead {}

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

#[derive(Debug)]
pub struct FileStore<'a> {
    file: &'a File,
    br: BufReader<&'a File>,
}

impl<'a> Store for FileStore<'a> {}
impl<'a> FileStore<'a> {
    pub fn new(f: &'a File) -> Result<Self, std::io::Error> {
        Ok(FileStore {
            file: f,
            br: BufReader::new(f),
        })
    }
}

impl<'a> Write for FileStore<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.file.lock_exclusive()?;
        // I'd prefer to use a LineWriter - https://doc.rust-lang.org/std/io/struct.LineWriter.html
        // But need to figure out how to expose this nicely in the Store interface
        // Maybe the same as BufReader where it's not exposed, but an implementation detail?
        let s = self.file.write(buf)?;
        let _ = self.file.write(b"\n")?;
        self.file.flush()?;
        self.file.unlock()?;

        Ok(s)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.file.lock_exclusive()?;
        self.file.write_all(buf)?;
        self.file.unlock()?;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.file.lock_exclusive()?;
        self.file.flush()?;
        self.file.unlock()?;
        Ok(())
    }
}

impl<'a> Read for FileStore<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.file.lock_exclusive()?;
        let s = self.file.read(buf)?;
        self.file.unlock()?;
        Ok(s)
    }
}

impl<'a> BufRead for FileStore<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.file.lock_exclusive()?;
        let r = self.br.fill_buf();
        self.file.unlock()?;
        r
    }

    fn consume(&mut self, amt: usize) {
        self.br.consume(amt)
    }
}
