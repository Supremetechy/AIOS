//! Filesystem driver for AI-OS
//! Supports ext2/FAT32 for autonomous file operations

pub mod ext2;
pub mod fat32;
pub mod vfs;

use alloc::string::String;
use alloc::vec::Vec;

/// File types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    Device,
}

/// File permissions (Unix-style)
#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    pub user_read: bool,
    pub user_write: bool,
    pub user_execute: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub group_execute: bool,
    pub other_read: bool,
    pub other_write: bool,
    pub other_execute: bool,
}

impl Permissions {
    pub fn from_mode(mode: u16) -> Self {
        Permissions {
            user_read: (mode & 0o400) != 0,
            user_write: (mode & 0o200) != 0,
            user_execute: (mode & 0o100) != 0,
            group_read: (mode & 0o040) != 0,
            group_write: (mode & 0o020) != 0,
            group_execute: (mode & 0o010) != 0,
            other_read: (mode & 0o004) != 0,
            other_write: (mode & 0o002) != 0,
            other_execute: (mode & 0o001) != 0,
        }
    }
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileStat {
    pub file_type: FileType,
    pub size: u64,
    pub permissions: Permissions,
    pub created: u64,
    pub modified: u64,
    pub accessed: u64,
}

/// File handle
pub struct FileHandle {
    pub inode: u64,
    pub position: u64,
    pub flags: u32,
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub inode: u64,
    pub file_type: FileType,
}

/// Filesystem operations trait
pub trait FileSystem {
    fn read(&mut self, inode: u64, offset: u64, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write(&mut self, inode: u64, offset: u64, buf: &[u8]) -> Result<usize, FsError>;
    fn create(&mut self, parent: u64, name: &str, file_type: FileType) -> Result<u64, FsError>;
    fn delete(&mut self, parent: u64, name: &str) -> Result<(), FsError>;
    fn lookup(&mut self, parent: u64, name: &str) -> Result<u64, FsError>;
    fn stat(&mut self, inode: u64) -> Result<FileStat, FsError>;
    fn readdir(&mut self, inode: u64) -> Result<Vec<DirEntry>, FsError>;
}

/// Filesystem errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotDirectory,
    IsDirectory,
    PermissionDenied,
    NoSpace,
    InvalidPath,
    IOError,
    Corrupted,
}

pub fn init() {
    use crate::println;
    println!("[FS] Initializing filesystem drivers...");
    
    // Initialize VFS
    vfs::init();
    
    // Initialize ext2 driver
    ext2::init();
    
    // Initialize FAT32 driver
    fat32::init();
    
    println!("[FS] ✓ Filesystem drivers initialized");
}
