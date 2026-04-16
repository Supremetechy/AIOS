//! FAT32 filesystem driver
//! Read/write support for FAT32 (common on USB drives)

use alloc::vec::Vec;
use alloc::string::String;
use super::{FileSystem, FileType, FileStat, DirEntry, FsError, Permissions};

/// FAT32 Boot Sector
#[repr(C, packed)]
struct Fat32BootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    root_entry_count: u16,
    total_sectors_16: u16,
    media: u8,
    fat_size_16: u16,
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    
    // FAT32 specific
    fat_size_32: u32,
    ext_flags: u16,
    fs_version: u16,
    root_cluster: u32,
    fs_info: u16,
    backup_boot_sector: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved1: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
}

/// FAT32 Directory Entry
#[repr(C, packed)]
struct Fat32DirEntry {
    name: [u8; 11],
    attributes: u8,
    reserved: u8,
    creation_time_tenths: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    first_cluster_high: u16,
    write_time: u16,
    write_date: u16,
    first_cluster_low: u16,
    file_size: u32,
}

pub struct Fat32FileSystem {
    disk_id: usize,
    bytes_per_sector: usize,
    sectors_per_cluster: usize,
    root_cluster: u32,
    fat_start: u64,
    data_start: u64,
}

impl Fat32FileSystem {
    pub fn new(disk_id: usize) -> Result<Self, FsError> {
        // TODO: Read boot sector from disk
        
        Ok(Fat32FileSystem {
            disk_id,
            bytes_per_sector: 512,
            sectors_per_cluster: 8,
            root_cluster: 2,
            fat_start: 0,
            data_start: 0,
        })
    }
    
    fn read_cluster(&mut self, cluster: u32, buf: &mut [u8]) -> Result<(), FsError> {
        // TODO: Read cluster from disk
        Ok(())
    }
    
    fn write_cluster(&mut self, cluster: u32, buf: &[u8]) -> Result<(), FsError> {
        // TODO: Write cluster to disk
        Ok(())
    }
    
    fn get_next_cluster(&mut self, cluster: u32) -> Result<u32, FsError> {
        // TODO: Read FAT to get next cluster in chain
        Ok(0xFFFFFFFF) // End of chain
    }
}

impl FileSystem for Fat32FileSystem {
    fn read(&mut self, inode: u64, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        let cluster = inode as u32;
        let cluster_size = self.bytes_per_sector * self.sectors_per_cluster;
        
        // TODO: Follow cluster chain and read data
        
        Ok(0)
    }
    
    fn write(&mut self, inode: u64, offset: u64, buf: &[u8]) -> Result<usize, FsError> {
        // TODO: Implement write
        Ok(buf.len())
    }
    
    fn create(&mut self, parent: u64, name: &str, file_type: FileType) -> Result<u64, FsError> {
        // TODO: Allocate cluster, add directory entry
        Ok(0)
    }
    
    fn delete(&mut self, parent: u64, name: &str) -> Result<(), FsError> {
        // TODO: Remove directory entry, free clusters
        Ok(())
    }
    
    fn lookup(&mut self, parent: u64, name: &str) -> Result<u64, FsError> {
        // TODO: Search directory for name
        Err(FsError::NotFound)
    }
    
    fn stat(&mut self, inode: u64) -> Result<FileStat, FsError> {
        // TODO: Read directory entry
        Ok(FileStat {
            file_type: FileType::Regular,
            size: 0,
            permissions: Permissions::from_mode(0o644),
            created: 0,
            modified: 0,
            accessed: 0,
        })
    }
    
    fn readdir(&mut self, inode: u64) -> Result<Vec<DirEntry>, FsError> {
        let mut entries = Vec::new();
        // TODO: Read directory entries
        Ok(entries)
    }
}

pub fn init() {
    use crate::println;
    println!("[FAT32] FAT32 filesystem driver loaded");
}
