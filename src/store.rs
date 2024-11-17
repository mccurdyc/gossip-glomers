use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Error, Read, Write};
use std::path::Path;

// std::io::{Read,Write} Supertrait
pub trait Store: Write + Read + BufRead {}

#[derive(Debug)]
pub struct MemoryStore<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> MemoryStore<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Result<Self, std::io::Error> {
        Ok(Self {
            buffer,
            position: 0,
        })
    }
}

impl<'a> Store for MemoryStore<'a> {}
impl<'a> Write for MemoryStore<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.buffer.copy_from_slice(&buf[..]);
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

impl<'a> Read for MemoryStore<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // Calculate how many bytes can be read
        let bytes_to_read = std::cmp::min(buf.len(), self.buffer.len() - self.position);

        if bytes_to_read == 0 {
            return Ok(0); // No more data to read
        }

        // Copy the data into the buffer
        buf[..bytes_to_read]
            .copy_from_slice(&self.buffer[self.position..self.position + bytes_to_read]);

        // Update the position
        self.position += bytes_to_read;

        Ok(bytes_to_read) // Return the number of bytes read
    }
}

impl<'a> BufRead for MemoryStore<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.position < self.buffer.len() {
            Ok(&self.buffer[self.position..])
        } else {
            Ok(&[]) // No more data
        }
    }
    fn consume(&mut self, amt: usize) {
        self.position += amt;
        if self.position > self.buffer.len() {
            self.position = self.buffer.len(); // Prevent overflow
        }
    }
}

#[derive(Debug)]
pub struct FileStore<'a> {
    file: File,
    buffer: &'a [u8],
    position: usize,
    buffer_size: usize,
    _path: &'a Path,
}

impl<'a> Store for FileStore<'a> {}
impl<'a> FileStore<'a> {
    pub fn new(path: &'a Path) -> Result<Self, std::io::Error> {
        let f = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        Ok(FileStore {
            file: f,
            buffer: &[],
            position: 0,
            buffer_size: 0,
            _path: path,
        })
    }
}

impl<'a> Write for FileStore<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.file.lock_exclusive()?;
        // I'd prefer to use a LineWriter - https://doc.rust-lang.org/std/io/struct.LineWriter.html
        // But need to figure out how to expose this nicely in the Store interface
        let s = self.file.write(buf)?;
        self.file.write(b"\n")?;
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
        let bytes_to_read = std::cmp::min(buf.len(), self.buffer_size - self.position);

        if bytes_to_read == 0 {
            return Ok(0); // No more data to read
        }

        // Copy the data into the buffer
        buf[..bytes_to_read]
            .copy_from_slice(&self.buffer[self.position..self.position + bytes_to_read]);

        // Update the position
        self.position += bytes_to_read;

        self.file.unlock()?;

        Ok(bytes_to_read) // Return the number of bytes read
    }
}

impl<'a> BufRead for FileStore<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.position < self.buffer.len() {
            Ok(&self.buffer[self.position..])
        } else {
            Ok(&[]) // No more data
        }
    }
    fn consume(&mut self, amt: usize) {
        self.position += amt;
        if self.position > self.buffer.len() {
            self.position = self.buffer.len(); // Prevent overflow
        }
    }
}
