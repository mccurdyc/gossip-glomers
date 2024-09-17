use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

pub struct Store {
    f: File,
}

impl Store {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        let f = OpenOptions::new().read(true).write(true).open(path)?;
        Ok(Store { f })
    }

    pub fn set(&mut self, v: u32) -> Result<(), std::io::Error> {
        self.f.lock_exclusive()?;

        // write
        let buf = u32::to_be_bytes(v);
        self.f.write_all(&buf)?;

        self.f.unlock()?;
        Ok(())
    }

    /// get assumes that the calling function will have the lock
    pub fn get(&mut self) -> Result<u32, std::io::Error> {
        self.f.lock_exclusive()?;

        let v = self.get_as_u32()?;

        self.f.unlock()?;
        Ok(v)
    }

    fn get_as_u32(&mut self) -> Result<u32, std::io::Error> {
        // read
        let mut buf = [0u8; 4]; // 4 bytes for u32
        self.f.read_exact(&mut buf)?;
        // convert to u32
        //  Little Endian (LE): The least significant byte is stored first.
        //  This is commonly used in modern architectures, such as x86 and x86-64
        Ok(u32::from_be_bytes(buf))
    }
}
