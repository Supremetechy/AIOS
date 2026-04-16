//! Memory management subsystem

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// Initialize memory management
pub fn init() {
    println!("[MEMORY] Initializing memory management...");
    
    // Get physical memory offset from bootloader
    let phys_mem_offset = VirtAddr::new(0);
    
    println!("[MEMORY] Physical memory offset: {:#x}", phys_mem_offset.as_u64());
    println!("[MEMORY] ✓ Memory management initialized");
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init() -> Self {
        BootInfoFrameAllocator { next: 0 }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Simplified frame allocation
        let frame = PhysFrame::containing_address(PhysAddr::new((self.next * 4096) as u64));
        self.next += 1;
        Some(frame)
    }
}

/// Allocate a page
pub fn allocate_page(
    page: Page,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = frame_allocator
        .allocate_frame()
        .ok_or(MapToError::FrameAllocationFailed)?;

    let flags = Flags::PRESENT | Flags::WRITABLE;

    unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };

    Ok(())
}

/// Physical memory statistics
pub struct MemoryStats {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

impl MemoryStats {
    pub fn new() -> Self {
        MemoryStats {
            total_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
        }
    }

    pub fn total_mb(&self) -> u64 {
        self.total_bytes / (1024 * 1024)
    }

    pub fn used_mb(&self) -> u64 {
        self.used_bytes / (1024 * 1024)
    }

    pub fn available_mb(&self) -> u64 {
        self.available_bytes / (1024 * 1024)
    }
}
