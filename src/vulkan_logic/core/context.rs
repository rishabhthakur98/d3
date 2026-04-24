// src/vulkan_logic/core/context.rs

use anyhow::{anyhow, Result};
use ash::{khr::surface, vk, Entry, Instance, Device};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;
use vk_mem::Allocator; 

pub struct VulkanContext {
    #[allow(dead_code)] // Vulkan requires this to live in memory, even if we don't explicitly read it later
    pub entry: Entry,
    pub instance: Instance,
    pub surface_loader: surface::Instance,
    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub graphics_queue_index: u32,
    pub present_queue_index: u32,
    pub allocator: Option<Allocator>, 
}

impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self> {
        let entry = unsafe { Entry::load() }?;
        let app_name = std::ffi::CString::new("D3")?;
        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .api_version(vk::API_VERSION_1_3);

        let display_handle = window.display_handle()?.as_raw();
        let extensions = ash_window::enumerate_required_extensions(display_handle)?;
        
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extensions);

        let instance = unsafe { entry.create_instance(&create_info, None) }?;
        let surface = unsafe { 
            ash_window::create_surface(&entry, &instance, display_handle, window.window_handle()?.as_raw(), None) 
        }?;
        let surface_loader = surface::Instance::new(&entry, &instance);

        let physical_device = match unsafe { instance.enumerate_physical_devices() }?.into_iter().next() {
            Some(device) => device,
            None => return Err(anyhow!("No compatible GPU found")),
        };

        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut graphics_index_opt = None;
        let mut present_index_opt = None;

        for (i, info) in queue_families.iter().enumerate() {
            if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) { 
                graphics_index_opt = Some(i as u32); 
            }
            let present = unsafe { 
                surface_loader.get_physical_device_surface_support(physical_device, i as u32, surface) 
            }?;
            if present { 
                present_index_opt = Some(i as u32); 
            }
            if graphics_index_opt.is_some() && present_index_opt.is_some() { 
                break; 
            }
        }

        let graphics_queue_index = match graphics_index_opt {
            Some(idx) => idx,
            None => return Err(anyhow!("No graphics queue found on GPU")),
        };
        let present_queue_index = match present_index_opt {
            Some(idx) => idx,
            None => return Err(anyhow!("No present queue found on GPU")),
        };

        let queue_priorities = [1.0];
        let mut queue_create_infos = vec![vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_queue_index)
            .queue_priorities(&queue_priorities)];
            
        if graphics_queue_index != present_queue_index {
            queue_create_infos.push(vk::DeviceQueueCreateInfo::default()
                .queue_family_index(present_queue_index)
                .queue_priorities(&queue_priorities));
        }

        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }?;
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_index, 0) };

        let allocator_info = vk_mem::AllocatorCreateInfo::new(&instance, &device, physical_device);
        let allocator = unsafe { Allocator::new(allocator_info) }
            .map_err(|e| anyhow!("Failed to create memory allocator: {}", e))?;

        Ok(Self {
            entry, instance, surface_loader, surface, physical_device, device,
            graphics_queue, present_queue, graphics_queue_index, present_queue_index,
            allocator: Some(allocator), 
        })
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.allocator = None; 
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}