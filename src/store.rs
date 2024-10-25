use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Write};
use std::path::Path;

// std::io::{Read,Write} Supertrait
pub trait Store: std::io::Write + std::io::Read {}

#[derive(Debug)]
pub struct MemoryStore {
    store: Vec<u8>,
}

impl MemoryStore {
    pub fn new() -> MemoryStore {
        Self { store: Vec::new() }
    }
}

impl Store for MemoryStore {}
impl Write for MemoryStore {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.store = buf.to_owned();
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
        let src = self.store.as_slice();
        buf.copy_from_slice(src);
        Ok(buf.len())
    }
}

#[derive(Debug)]
pub struct FileStore<'a> {
    file: File,
    path: &'a Path,
}

impl<'a> Store for FileStore<'a> {}
impl<'a> FileStore<'a> {
    pub fn new(path: &'a Path) -> Result<Self, std::io::Error> {
        let f = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        Ok(FileStore { file: f, path })
    }
}

impl<'a> Write for FileStore<'a> {
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

impl<'a> Read for FileStore<'a> {
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
