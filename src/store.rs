use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Write};
use std::path::Path;

// std::io::{Read,Write} Supertrait
pub trait Store: std::io::Write + std::io::Read {}

#[derive(Debug)]
pub struct MemoryStore {
    data: Vec<u8>,
    position: usize,
}

impl MemoryStore {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            data: Vec::new(),
            position: 0,
        })
    }
}

impl Store for MemoryStore {}
impl Write for MemoryStore {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.data = buf.to_owned();
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
        let bytes_to_read = std::cmp::min(buf.len(), self.data.len() - self.position);

        if bytes_to_read == 0 {
            return Ok(0); // No more data to read
        }

        // Copy the data into the buffer
        buf[..bytes_to_read]
            .copy_from_slice(&self.data[self.position..self.position + bytes_to_read]);

        // Update the position
        self.position += bytes_to_read;

        Ok(bytes_to_read) // Return the number of bytes read
    }
}

pub struct FileStore<'a> {
    file: File,
    _path: &'a Path,
}

impl<'a: 'static> Store for FileStore<'a> {}
impl<'a: 'static> FileStore<'a> {
    pub fn new(path: &'a Path) -> Result<Self, std::io::Error> {
        let f = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        Ok(FileStore {
            file: f,
            _path: path,
        })
    }
}

impl<'a: 'static> Write for FileStore<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.file.lock_exclusive()?;
        let s = self.file.write(buf)?;
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

impl<'a: 'static> Read for FileStore<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.file.lock_exclusive()?;
        let mut b = Vec::new();
        self.file.read_to_end(&mut b)?;

        let src = b.as_slice();
        buf.copy_from_slice(src);
        self.file.unlock()?;

        Ok(buf.len())
    }
}
