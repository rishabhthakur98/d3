// src/vulkan_logic/gpu_buffers.rs

use anyhow::{anyhow, Result};
use ash::vk;
use vk_mem::{Allocation, AllocationCreateFlags, AllocationCreateInfo, Allocator, MemoryUsage};

// Required trait to use allocator.create_buffer
use vk_mem::Alloc; 

/// A memory buffer that lives on the GPU but is mapped so the CPU can write to it easily.
pub struct DynamicBuffer {
    pub buffer: vk::Buffer,
    pub allocation: Allocation,
    pub capacity: usize,
    pub usage: vk::BufferUsageFlags,
}

impl DynamicBuffer {
    pub fn new(allocator: &Allocator, capacity: usize, usage: vk::BufferUsageFlags) -> Result<Self> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(capacity as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = AllocationCreateInfo {
            usage: MemoryUsage::Auto, 
            flags: AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE | AllocationCreateFlags::MAPPED,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe { allocator.create_buffer(&buffer_info, &alloc_info) }
            .map_err(|e| anyhow!("Failed to allocate GPU buffer: {}", e))?;

        Ok(Self { buffer, allocation, capacity, usage })
    }

    /// Fast-copies a slice of Rust data directly into GPU memory
    pub fn upload_data<T>(&mut self, allocator: &Allocator, data: &[T]) -> Result<()> {
        let size = std::mem::size_of_val(data);
        if size == 0 { return Ok(()); }

        // Automatically grow the buffer if it gets too full
        if size > self.capacity {
            unsafe { allocator.destroy_buffer(self.buffer, &mut self.allocation); }

            self.capacity = size.next_power_of_two(); 

            let buffer_info = vk::BufferCreateInfo::default()
                .size(self.capacity as u64)
                .usage(self.usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let alloc_info = AllocationCreateInfo {
                usage: MemoryUsage::Auto,
                flags: AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE | AllocationCreateFlags::MAPPED,
                ..Default::default()
            };

            let (new_buffer, new_alloc) = unsafe { allocator.create_buffer(&buffer_info, &alloc_info) }
                .map_err(|e| anyhow!("Failed to re-allocate GPU buffer: {}", e))?;

            self.buffer = new_buffer;
            self.allocation = new_alloc;
        }

        // Safely extract the mapped memory pointer
        let info = allocator.get_allocation_info(&self.allocation);
        if info.mapped_data.is_null() {
            return Err(anyhow!("Buffer memory is not mapped!"));
        }
        
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, info.mapped_data as *mut u8, size);
        }

        Ok(())
    }

    pub fn destroy(&mut self, allocator: &Allocator) {
        unsafe { allocator.destroy_buffer(self.buffer, &mut self.allocation); }
    }
}