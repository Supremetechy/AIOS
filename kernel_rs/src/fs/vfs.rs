//! Virtual File System (VFS) layer
//! Abstracts different filesystems for AI autonomous operations

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use super::{FileSystem, FileType, FileStat, DirEntry, FsError};

/// Mount point
struct MountPoint {
    path: String,
    filesystem: Box<dyn FileSystem>,
}

/// Virtual File System
pub struct VFS {
    mounts: BTreeMap<String, MountPoint>,
    open_files: BTreeMap<u32, OpenFile>,
    next_fd: u32,
}

struct OpenFile {
    inode: u64,
    position: u64,
    flags: u32,
    mount_path: String,
}

impl VFS {
    pub const fn new() -> Self {
        VFS {
            mounts: BTreeMap::new(),
            open_files: BTreeMap::new(),
            next_fd: 3, // 0=stdin, 1=stdout, 2=stderr
        }
    }
    
    /// Mount a filesystem
    pub fn mount(&mut self, path: String, fs: Box<dyn FileSystem>) -> Result<(), FsError> {
        self.mounts.insert(path.clone(), MountPoint {
            path: path.clone(),
            filesystem: fs,
        });
        Ok(())
    }
    
    /// Open a file
    pub fn open(&mut self, path: &str, flags: u32) -> Result<u32, FsError> {
        let (mount_path, rel_path) = self.find_mount(path)?;
        let mount = self.mounts.get_mut(&mount_path).ok_or(FsError::NotFound)?;
        
        // Traverse path to get inode
        let inode = self.path_to_inode(&mut mount.filesystem, rel_path)?;
        
        let fd = self.next_fd;
        self.next_fd += 1;
        
        self.open_files.insert(fd, OpenFile {
            inode,
            position: 0,
            flags,
            mount_path,
        });
        
        Ok(fd)
    }
    
    /// Read from file
    pub fn read(&mut self, fd: u32, buf: &mut [u8]) -> Result<usize, FsError> {
        let file = self.open_files.get_mut(&fd).ok_or(FsError::NotFound)?;
        let mount = self.mounts.get_mut(&file.mount_path).ok_or(FsError::NotFound)?;
        
        let bytes_read = mount.filesystem.read(file.inode, file.position, buf)?;
        file.position += bytes_read as u64;
        
        Ok(bytes_read)
    }
    
    /// Write to file
    pub fn write(&mut self, fd: u32, buf: &[u8]) -> Result<usize, FsError> {
        let file = self.open_files.get_mut(&fd).ok_or(FsError::NotFound)?;
        let mount = self.mounts.get_mut(&file.mount_path).ok_or(FsError::NotFound)?;
        
        let bytes_written = mount.filesystem.write(file.inode, file.position, buf)?;
        file.position += bytes_written as u64;
        
        Ok(bytes_written)
    }
    
    /// Close file
    pub fn close(&mut self, fd: u32) -> Result<(), FsError> {
        self.open_files.remove(&fd).ok_or(FsError::NotFound)?;
        Ok(())
    }
    
    /// Create file (for autonomous operations)
    pub fn create(&mut self, path: &str, file_type: FileType) -> Result<(), FsError> {
        let (mount_path, rel_path) = self.find_mount(path)?;
        let mount = self.mounts.get_mut(&mount_path).ok_or(FsError::NotFound)?;
        
        // Split path into parent and name
        let (parent_path, name) = split_path(rel_path);
        let parent_inode = self.path_to_inode(&mut mount.filesystem, parent_path)?;
        
        mount.filesystem.create(parent_inode, name, file_type)?;
        Ok(())
    }
    
    /// Delete file
    pub fn delete(&mut self, path: &str) -> Result<(), FsError> {
        let (mount_path, rel_path) = self.find_mount(path)?;
        let mount = self.mounts.get_mut(&mount_path).ok_or(FsError::NotFound)?;
        
        let (parent_path, name) = split_path(rel_path);
        let parent_inode = self.path_to_inode(&mut mount.filesystem, parent_path)?;
        
        mount.filesystem.delete(parent_inode, name)?;
        Ok(())
    }
    
    /// Get file stats
    pub fn stat(&mut self, path: &str) -> Result<FileStat, FsError> {
        let (mount_path, rel_path) = self.find_mount(path)?;
        let mount = self.mounts.get_mut(&mount_path).ok_or(FsError::NotFound)?;
        
        let inode = self.path_to_inode(&mut mount.filesystem, rel_path)?;
        mount.filesystem.stat(inode)
    }
    
    /// Read directory
    pub fn readdir(&mut self, path: &str) -> Result<Vec<DirEntry>, FsError> {
        let (mount_path, rel_path) = self.find_mount(path)?;
        let mount = self.mounts.get_mut(&mount_path).ok_or(FsError::NotFound)?;
        
        let inode = self.path_to_inode(&mut mount.filesystem, rel_path)?;
        mount.filesystem.readdir(inode)
    }
    
    // Helper functions
    
    fn find_mount(&self, path: &str) -> Result<(String, &str), FsError> {
        // Find longest matching mount point
        let mut best_match = String::new();
        
        for mount_path in self.mounts.keys() {
            if path.starts_with(mount_path.as_str()) {
                if mount_path.len() > best_match.len() {
                    best_match = mount_path.clone();
                }
            }
        }
        
        if best_match.is_empty() {
            return Err(FsError::NotFound);
        }
        
        let rel_path = &path[best_match.len()..];
        Ok((best_match, rel_path))
    }
    
    fn path_to_inode(&mut self, fs: &mut Box<dyn FileSystem>, path: &str) -> Result<u64, FsError> {
        if path.is_empty() || path == "/" {
            return Ok(2); // Root inode
        }
        
        let mut current_inode = 2; // Start at root
        
        for component in path.split('/').filter(|s| !s.is_empty()) {
            current_inode = fs.lookup(current_inode, component)?;
        }
        
        Ok(current_inode)
    }
}

fn split_path(path: &str) -> (&str, &str) {
    if let Some(pos) = path.rfind('/') {
        let parent = if pos == 0 { "/" } else { &path[..pos] };
        let name = &path[pos + 1..];
        (parent, name)
    } else {
        ("/", path)
    }
}

static VFS_INSTANCE: Mutex<VFS> = Mutex::new(VFS::new());

pub fn init() {
    use crate::println;
    println!("[VFS] Initializing Virtual File System...");
    println!("[VFS] ✓ VFS initialized");
}

// Public API
pub fn open(path: &str, flags: u32) -> Result<u32, FsError> {
    VFS_INSTANCE.lock().open(path, flags)
}

pub fn read(fd: u32, buf: &mut [u8]) -> Result<usize, FsError> {
    VFS_INSTANCE.lock().read(fd, buf)
}

pub fn write(fd: u32, buf: &[u8]) -> Result<usize, FsError> {
    VFS_INSTANCE.lock().write(fd, buf)
}

pub fn close(fd: u32) -> Result<(), FsError> {
    VFS_INSTANCE.lock().close(fd)
}

pub fn create(path: &str, file_type: FileType) -> Result<(), FsError> {
    VFS_INSTANCE.lock().create(path, file_type)
}

pub fn delete(path: &str) -> Result<(), FsError> {
    VFS_INSTANCE.lock().delete(path)
}

pub fn stat(path: &str) -> Result<FileStat, FsError> {
    VFS_INSTANCE.lock().stat(path)
}

pub fn readdir(path: &str) -> Result<Vec<DirEntry>, FsError> {
    VFS_INSTANCE.lock().readdir(path)
}
