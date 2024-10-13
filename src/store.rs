use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Write};
use std::path::Path;

#[derive(Debug)]
pub struct FileStore<'a> {
    file: File,
    path: &'a Path,
}

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
        let s = self.file.flush()?;
        self.file.unlock()?;
        Ok(())
    }
}

impl<'a> Read for FileStore<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.file.lock_exclusive()?;
        let mut buf = Vec::new();
        self.file.read_to_end(&mut buf)?;
        self.file.unlock()?;

        Ok(buf.len())
    }
}
