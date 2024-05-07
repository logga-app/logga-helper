use std::fs::File;
use std::io;
use std::os::unix::fs::MetadataExt;

// Extend fs::File with convenience functions
pub trait FileExtension {
    fn was_rotated(&self, curr_inode: u64) -> Result<bool, io::Error>;
    fn was_truncated(&self, curr_inode: u64, curr_pos: u64) -> Result<bool, io::Error>;
}

impl FileExtension for File {
    fn was_rotated(&self, curr_inode: u64) -> Result<bool, io::Error> {
        Ok(self.metadata()?.ino() != curr_inode)
    }

    fn was_truncated(&self, curr_inode: u64, curr_pos: u64) -> Result<bool, io::Error> {
        let md = self.metadata()?;
        Ok(curr_inode == md.ino() && md.len() < curr_pos)
    }
}
