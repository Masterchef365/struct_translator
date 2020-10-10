use anyhow::{format_err, Context, Result};
use erupt::{
    cstr,
    utils::{
        allocator::{Allocator, AllocatorCreateInfo, MemoryTypeFinder},
        decode_spv,
        loading::DefaultEntryLoader,
    },
    vk1_0 as vk, DeviceLoader, EntryLoader, InstanceLoader,
};
use std::ffi::CString;

pub struct ShaderExecutor {
    queue: vk::Queue,
    allocator: Allocator,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    descriptor_set_layout: vk::DescriptorSetLayout,
    device: DeviceLoader,
    instance: InstanceLoader,
    _entry: DefaultEntryLoader,
}

impl ShaderExecutor {
    pub fn new() -> Result<Self> {
        // Entry
        let entry = EntryLoader::new()?;

        // Instance
        let name = CString::new("Compute test")?;
        let app_info = vk::ApplicationInfoBuilder::new()
            .application_name(&name)
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(&name)
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 0, 0));

        // Instance and device layers and extensions
        let mut instance_layers = Vec::new();
        let mut instance_extensions = Vec::new();
        let mut device_layers = Vec::new();
        let device_extensions = Vec::new();

        // Vulkan layers and extensions
        const LAYER_KHRONOS_VALIDATION: *const i8 = cstr!("VK_LAYER_KHRONOS_validation");
        instance_extensions
            .push(erupt::extensions::ext_debug_utils::EXT_DEBUG_UTILS_EXTENSION_NAME);
        instance_layers.push(LAYER_KHRONOS_VALIDATION);
        device_layers.push(LAYER_KHRONOS_VALIDATION);

        // Instance creation
        let create_info = vk::InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&instance_layers);

        let instance = InstanceLoader::new(&entry, &create_info, None)?;

        // Hardware selection
        let (queue_family_index, physical_device) = select_device(&instance)?;

        // Create logical device and queues
        let create_info = [vk::DeviceQueueCreateInfoBuilder::new()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0])];

        let physical_device_features = vk::PhysicalDeviceFeaturesBuilder::new();
        let create_info = vk::DeviceCreateInfoBuilder::new()
            .queue_create_infos(&create_info)
            .enabled_features(&physical_device_features)
            .enabled_extension_names(&device_extensions)
            .enabled_layer_names(&device_layers);

        let device = DeviceLoader::new(&instance, physical_device, &create_info, None)?;
        let queue = unsafe { device.get_device_queue(queue_family_index, 0, None) };

        // Allocator
        let mut allocator =
            Allocator::new(&instance, physical_device, AllocatorCreateInfo::default()).result()?;

        // Descriptors
        // Pool:
        let pool_sizes = [vk::DescriptorPoolSizeBuilder::new()
            ._type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)];
        let create_info = vk::DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(&pool_sizes)
            .max_sets(1);
        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&create_info, None, None) }.result()?;

        // Layout:
        let bindings = [vk::DescriptorSetLayoutBindingBuilder::new()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE)];

        let create_info = vk::DescriptorSetLayoutCreateInfoBuilder::new().bindings(&bindings);

        let descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&create_info, None, None) }.result()?;

        // Set:
        let descriptor_set_layouts = [descriptor_set_layout];
        let create_info = vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&descriptor_set_layouts);

        let descriptor_set = unsafe { device.allocate_descriptor_sets(&create_info) }.result()?[0];

        // Create command buffer
        // Command pool:
        let create_info = vk::CommandPoolCreateInfoBuilder::new()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);
        let command_pool = unsafe { device.create_command_pool(&create_info, None, None) }.result()?;

        // Buffers:
        let allocate_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer =
            unsafe { device.allocate_command_buffers(&allocate_info) }.result()?[0];


        Ok(Self {
            queue,
            descriptor_set,
            descriptor_pool,
            allocator,
            command_pool,
            command_buffer,
            descriptor_set_layout,
            device,
            instance,
            _entry: entry,
        })
    }

    pub fn run_shader(&mut self, shader_spv: &[u8], shader_buf: &mut [u8], invocations: u32) -> Result<()> {
        // Load shader
        let shader_spirv = std::fs::read("shaders/op.comp.spv").context("Shader failed to load")?;
        let shader_decoded = decode_spv(&shader_spirv).context("Shader decode failed")?;
        let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&shader_decoded);
        let shader_module =
            unsafe { self.device.create_shader_module(&create_info, None, None) }.result()?;

        // Pipeline
        let descriptor_set_layouts = [self.descriptor_set_layout];
        let create_info =
            vk::PipelineLayoutCreateInfoBuilder::new().set_layouts(&descriptor_set_layouts);
        let pipeline_layout =
            unsafe { self.device.create_pipeline_layout(&create_info, None, None) }.result()?;

        let entry_point = CString::new("main")?;
        let stage = vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::COMPUTE)
            .module(shader_module)
            .name(&entry_point)
            .build();
        let create_info = vk::ComputePipelineCreateInfoBuilder::new()
            .stage(stage)
            .layout(pipeline_layout);
        let pipeline =
            unsafe { self.device.create_compute_pipelines(None, &[create_info], None) }.result()?[0];

        // Allocate I/O buffer
        let buffer_size = shader_buf.len();
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(buffer_size as u64);

        let buffer = unsafe { self.device.create_buffer(&create_info, None, None) }.result()?;
        let buffer_allocation = self.allocator
            .allocate(&self.device, buffer, MemoryTypeFinder::dynamic())
            .result()?;

        // Write command buffer
        unsafe {
            self.device.reset_command_buffer(self.command_buffer, None).result()?;
            let begin_info = vk::CommandBufferBeginInfoBuilder::new()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.device.begin_command_buffer(self.command_buffer, &begin_info).result()?;

            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline,
            );

            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline_layout,
                0,
                &[self.descriptor_set],
                &[],
            );

            self.device.cmd_dispatch(
                self.command_buffer,
                invocations,
                1,
                1,
            );

            self.device.end_command_buffer(self.command_buffer).result()?;
        }

        // Update descriptor set to include the buffer
        unsafe {
            self.device.update_descriptor_sets(
                &[vk::WriteDescriptorSetBuilder::new()
                .dst_set(self.descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&[vk::DescriptorBufferInfoBuilder::new()
                    .buffer(buffer)
                    .offset(buffer_allocation.region().start)
                    .range(buffer_size as u64)])],
                    &[],
            )
        };

        // Copy data to input
        let mut mapped = buffer_allocation.map(&self.device, ..).result()?;
        mapped.import(shader_buf);

        // Submit command buffer, and wait on data
        unsafe {
            let command_buffers = [self.command_buffer];
            let submit_infos = [vk::SubmitInfoBuilder::new()
                .command_buffers(&command_buffers)];
            self.device.queue_submit(
                self.queue,
                &submit_infos,
                None
            ).result()?;
            self.device.queue_wait_idle(self.queue).result()?;
        }

        // Read the values back
        shader_buf.copy_from_slice(mapped.read());

        self.allocator.free(&self.device, buffer_allocation);

        Ok(())
    }
}

fn select_device(instance: &InstanceLoader) -> Result<(u32, vk::PhysicalDevice)> {
    let physical_devices = unsafe { instance.enumerate_physical_devices(None) }.result()?;
    for device in physical_devices {
        let families =
            unsafe { instance.get_physical_device_queue_family_properties(device, None) };
        for (family, properites) in families.iter().enumerate() {
            if properites.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                return Ok((family as u32, device));
            }
        }
    }
    Err(format_err!("No suitable device found"))
}

impl Drop for ShaderExecutor {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(Some(self.command_pool), None);
            self.device.destroy_descriptor_pool(Some(self.descriptor_pool), None);
            self.device.destroy_descriptor_set_layout(Some(self.descriptor_set_layout), None);
            self.device.destroy_device(None);
        }
    }
}
