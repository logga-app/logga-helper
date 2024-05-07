use crate::forwarder::error::TailError;
use log::debug;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::result;
use whoami;

use crate::forwarder::file::FileExtension;

pub struct Tail {
    path: String,
    checkpoint: u64,
    // We aim to reuse the file descriptor
    fd: File,
    buffer_size: usize,
    inode: u64,
}

type Result<T> = result::Result<T, TailError>;

impl Tail {
    pub fn new(path: String) -> Result<Tail> {
        let fd = fs::File::open(&path).map_err(|_| TailError::FileOpenError(path.clone()))?;

        let md = fd.metadata()?;

        let mut tail = Tail {
            path: path.clone(),
            checkpoint: 0,
            fd,
            inode: md.ino(),
            // hard code?
            buffer_size: 65536,
        };

        tail.load_checkpoint()?;

        Ok(tail)
    }

    pub fn next(&mut self) -> Result<Vec<u8>> {
        self.handle_file_operations()?;

        let mut buf_reader = BufReader::with_capacity(self.buffer_size, &self.fd);
        let mut data = vec![0; self.buffer_size];
        buf_reader
            .seek(SeekFrom::Start(self.checkpoint))
            .map_err(|err| TailError::StartSeekError(err))?;

        let bytes_read = buf_reader
            .read(&mut data)
            .map_err(|err| TailError::BufferReadError(err))?;

        self.checkpoint += bytes_read as u64;

        let data = data.into_iter().take(bytes_read).collect();
        Ok(data)
    }

    pub fn next_then<F: Fn(&[u8])>(&mut self, f: F) -> Result<()> {
        let data = self.next()?;
        f(&data);
        Ok(())
    }

    fn handle_file_operations(&mut self) -> Result<()> {
        // When a file was truncated, the content is deleted, but the inode number stays the same.
        // Checkpoint is reset, read starts from the beginning of the file.

        if self
            .fd
            .was_truncated(self.inode, self.checkpoint)
            .map_err(|err| TailError::FileOperationError(err))?
        {
            self.checkpoint = 0;
            debug!("file was truncated")
        }

        // When a file was rotated, it gets renamed & a new file is being created in its place,
        // thus the inode number will differ from our stored inode.
        // Checkpoint is reset, read starts from the beginning of the file.
        if self
            .fd
            .was_rotated(self.inode)
            .map_err(|err| TailError::FileOperationError(err))?
        {
            self.inode = self.fd.metadata()?.ino();
            self.checkpoint = 0;
            debug!("file was rotated")
        }

        Ok(())
    }

    // Tries to restore checkpoint from multiple locations.
    fn load_checkpoint(&mut self) -> Result<()> {
        // First look for .checkpoint in the same directory as the tailed file.
        let checkpoint_name = ".checkpoint";
        let path = PathBuf::from(self.path.clone());

        // Fallback to $HOME/.checkpoint
        let default_checkpoint_path = default_path(&checkpoint_name);
        let parent = path.parent().unwrap_or(default_checkpoint_path.as_path());
        let mut parent = parent.to_path_buf();
        parent.push(checkpoint_name);

        let parent_str = parent.to_path_buf().into_os_string().into_string().unwrap();

        let fd = fs::File::open(parent).map_err(|_| TailError::FileOpenError(parent_str))?;
        let mut reader = BufReader::new(fd);
        let mut first_line = String::new();
        let _ = reader.read_line(&mut first_line)?;

        self.checkpoint = first_line
            .trim()
            .parse::<u64>()
            .map_err(|err| TailError::CastError(err))?;

        debug!("checkpoint restored: {}", self.checkpoint);
        return Ok(());
    }

    // Save checkpoint on different SIGNALs
    pub fn save_checkpoint(&self) -> Result<()> {
        // TODO: implement
        debug!("saving checkpoint");
        Ok(())
    }
}

fn default_path(file_name: &str) -> PathBuf {
    let mut default_path = match std::env::home_dir() {
        Some(hd) => hd,
        None => PathBuf::from(["/Users", &whoami::username()].join("/")),
    };
    default_path.push(file_name);

    default_path
}
