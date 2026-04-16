//! ext2 filesystem driver
//! Read/write support for ext2 filesystems

use alloc::vec::Vec;
use alloc::string::String;
use super::{FileSystem, FileType, FileStat, DirEntry, FsError, Permissions};

/// ext2 superblock
#[repr(C, packed)]
struct Ext2Superblock {
    inodes_count: u32,
    blocks_count: u32,
    reserved_blocks: u32,
    free_blocks: u32,
    free_inodes: u32,
    first_data_block: u32,
    log_block_size: u32,
    log_frag_size: u32,
    blocks_per_group: u32,
    frags_per_group: u32,
    inodes_per_group: u32,
    mtime: u32,
    wtime: u32,
    mnt_count: u16,
    max_mnt_count: u16,
    magic: u16,
    state: u16,
    errors: u16,
    minor_rev_level: u16,
    lastcheck: u32,
    checkinterval: u32,
    creator_os: u32,
    rev_level: u32,
    def_resuid: u16,
    def_resgid: u16,
}

/// ext2 inode
#[repr(C, packed)]
struct Ext2Inode {
    mode: u16,
    uid: u16,
    size: u32,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    gid: u16,
    links_count: u16,
    blocks: u32,
    flags: u32,
    osd1: u32,
    block: [u32; 15],
    generation: u32,
    file_acl: u32,
    dir_acl: u32,
    faddr: u32,
    osd2: [u8; 12],
}

/// ext2 directory entry
#[repr(C, packed)]
struct Ext2DirEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
    // name follows (variable length)
}

pub struct Ext2FileSystem {
    disk_id: usize,
    block_size: usize,
    superblock: Ext2Superblock,
}

impl Ext2FileSystem {
    pub fn new(disk_id: usize) -> Result<Self, FsError> {
        // TODO: Read superblock from disk
        let superblock = unsafe { core::mem::zeroed() };
        
        Ok(Ext2FileSystem {
            disk_id,
            block_size: 1024,
            superblock,
        })
    }
    
    fn read_inode(&mut self, inode_num: u64) -> Result<Ext2Inode, FsError> {
        // TODO: Read inode from disk
        Ok(unsafe { core::mem::zeroed() })
    }
    
    fn read_block(&mut self, block_num: u32, buf: &mut [u8]) -> Result<(), FsError> {
        // TODO: Read block from disk via AHCI driver
        Ok(())
    }
    
    fn write_block(&mut self, block_num: u32, buf: &[u8]) -> Result<(), FsError> {
        // TODO: Write block to disk via AHCI driver
        Ok(())
    }
}

impl FileSystem for Ext2FileSystem {
    fn read(&mut self, inode: u64, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        let inode_data = self.read_inode(inode)?;
        
        // Calculate which block to read
        let block_offset = (offset / self.block_size as u64) as usize;
        let byte_offset = (offset % self.block_size as u64) as usize;
        
        // Read from direct blocks (simplified)
        if block_offset < 12 {
            let block_num = inode_data.block[block_offset];
            let mut block_buf = vec![0u8; self.block_size];
            self.read_block(block_num, &mut block_buf)?;
            
            let to_copy = core::cmp::min(buf.len(), self.block_size - byte_offset);
            buf[..to_copy].copy_from_slice(&block_buf[byte_offset..byte_offset + to_copy]);
            
            return Ok(to_copy);
        }
        
        // TODO: Handle indirect blocks
        Ok(0)
    }
    
    fn write(&mut self, inode: u64, offset: u64, buf: &[u8]) -> Result<usize, FsError> {
        // TODO: Implement write
        Ok(buf.len())
    }
    
    fn create(&mut self, parent: u64, name: &str, file_type: FileType) -> Result<u64, FsError> {
        // TODO: Allocate new inode, add directory entry
        Ok(0)
    }
    
    fn delete(&mut self, parent: u64, name: &str) -> Result<(), FsError> {
        // TODO: Remove directory entry, free inode
        Ok(())
    }
    
    fn lookup(&mut self, parent: u64, name: &str) -> Result<u64, FsError> {
        let parent_inode = self.read_inode(parent)?;
        
        // Read directory entries
        // TODO: Iterate through directory blocks
        
        Err(FsError::NotFound)
    }
    
    fn stat(&mut self, inode: u64) -> Result<FileStat, FsError> {
        let inode_data = self.read_inode(inode)?;
        
        let file_type = match inode_data.mode & 0xF000 {
            0x8000 => FileType::Regular,
            0x4000 => FileType::Directory,
            0xA000 => FileType::Symlink,
            _ => FileType::Regular,
        };
        
        Ok(FileStat {
            file_type,
            size: inode_data.size as u64,
            permissions: Permissions::from_mode(inode_data.mode),
            created: inode_data.ctime as u64,
            modified: inode_data.mtime as u64,
            accessed: inode_data.atime as u64,
        })
    }
    
    fn readdir(&mut self, inode: u64) -> Result<Vec<DirEntry>, FsError> {
        let inode_data = self.read_inode(inode)?;
        let mut entries = Vec::new();
        
        // TODO: Read directory entries from blocks
        
        Ok(entries)
    }
}

pub fn init() {
    use crate::println;
    println!("[EXT2] ext2 filesystem driver loaded");
}
