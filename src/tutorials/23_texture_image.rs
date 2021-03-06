
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::share,
    utility::debug::*,
    utility::structures::*,
    utility::constants::*,
    utility::window::{ VulkanApp, ProgramProc },
};

extern crate winit;
extern crate ash;
extern crate cgmath;
extern crate image;

use ash::vk;
use ash::version::{ V1_0, InstanceV1_0 };
use ash::version::DeviceV1_0;
use ash::vk::types::uint32_t;
use cgmath::{ Matrix4, Deg, Point3, Vector3, SquareMatrix };
use image::GenericImage;

type EntryV1 = ash::Entry<V1_0>;

use std::path::Path;
use std::ptr;
use std::ffi::CString;

// Constants
const WINDOW_TITLE: &'static str = "23.Texture Image";
const TEXTURE_PATH: &'static str = "assets/texture.jpg";


struct VulkanApp23 {

    window: winit::Window,

    // vulkan stuff
    _entry                     : EntryV1,
    instance                   : ash::Instance<V1_0>,
    surface_loader             : ash::extensions::Surface,
    surface                    : vk::SurfaceKHR,
    debug_report_loader        : ash::extensions::DebugReport,
    debug_callback             : vk::DebugReportCallbackEXT,

    physical_device            : vk::PhysicalDevice,
    device                     : ash::Device<V1_0>,

    queue_family               : QueueFamilyIndices,
    graphics_queue             : vk::Queue,
    present_queue              : vk::Queue,

    swapchain_loader           : ash::extensions::Swapchain,
    swapchain                  : vk::SwapchainKHR,
    swapchain_images           : Vec<vk::Image>,
    swapchain_format           : vk::Format,
    swapchain_extent           : vk::Extent2D,
    swapchain_imageviews       : Vec<vk::ImageView>,
    swapchain_framebuffers     : Vec<vk::Framebuffer>,

    render_pass                : vk::RenderPass,
    ubo_layout                 : vk::DescriptorSetLayout,
    pipeline_layout            : vk::PipelineLayout,
    graphics_pipeline          : vk::Pipeline,

    texture_image              : vk::Image,
    texture_image_memory       : vk::DeviceMemory,

    vertex_buffer              : vk::Buffer,
    vertex_buffer_memory       : vk::DeviceMemory,
    index_buffer               : vk::Buffer,
    index_buffer_memory        : vk::DeviceMemory,

    uniform_transform          : UniformBufferObject,
    uniform_buffers            : Vec<vk::Buffer>,
    uniform_buffers_memory     : Vec<vk::DeviceMemory>,

    descriptor_pool            : vk::DescriptorPool,
    descriptor_sets            : Vec<vk::DescriptorSet>,

    command_pool               : vk::CommandPool,
    command_buffers            : Vec<vk::CommandBuffer>,

    image_available_semaphores : Vec<vk::Semaphore>,
    render_finished_semaphores : Vec<vk::Semaphore>,
    in_flight_fences           : Vec<vk::Fence>,
    current_frame              : usize,

    is_framebuffer_resized     : bool,
}

impl VulkanApp23 {

    pub fn new(event_loop: &winit::EventsLoop) -> VulkanApp23 {

        let window = utility::window::init_window(&event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = EntryV1::new().unwrap();
        let instance = share::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = share::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_report_loader, debug_callback) = setup_debug_callback(VALIDATION.is_enable, &entry, &instance);
        let physical_device = share::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let physical_device_memory_properties = instance.get_physical_device_memory_properties(physical_device);
        let (device, queue_family) = share::create_logical_device(&instance, physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(queue_family.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(queue_family.present_family as u32, 0) };
        let swapchain_stuff = share::create_swapchain(&instance, &device, physical_device, &window, &surface_stuff, &queue_family);
        let swapchain_imageviews = share::v1::create_image_views(&device, swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let render_pass = share::v1::create_render_pass(&device, swapchain_stuff.swapchain_format);
        let ubo_layout = share::v1::create_descriptor_set_layout(&device);
        let (graphics_pipeline, pipeline_layout) = VulkanApp23::create_graphics_pipeline(&device, render_pass, swapchain_stuff.swapchain_extent, ubo_layout);
        let swapchain_framebuffers = share::v1::create_framebuffers(&device, render_pass, &swapchain_imageviews, swapchain_stuff.swapchain_extent);
        let command_pool = share::v1::create_command_pool(&device, &queue_family);
        let (texture_image, texture_image_memory) = VulkanApp23::create_texture_image(&device, command_pool, graphics_queue, &physical_device_memory_properties, &Path::new(TEXTURE_PATH));
        let (vertex_buffer, vertex_buffer_memory) = share::v1::create_vertex_buffer(&device, &physical_device_memory_properties, command_pool, graphics_queue, &RECT_VERTICES_DATA);
        let (index_buffer, index_buffer_memory) = share::v1::create_index_buffer(&device, &physical_device_memory_properties, command_pool, graphics_queue, &RECT_INDICES_DATA);
        let (uniform_buffers, uniform_buffers_memory) = share::v1::create_uniform_buffers(&device, &physical_device_memory_properties, swapchain_stuff.swapchain_images.len());
        let descriptor_pool = share::v1::create_descriptor_pool(&device, swapchain_stuff.swapchain_images.len());
        let descriptor_sets = share::v1::create_descriptor_sets(&device, descriptor_pool, ubo_layout, &uniform_buffers, swapchain_stuff.swapchain_images.len());
        let command_buffers = VulkanApp23::create_command_buffers(&device, command_pool, graphics_pipeline, &swapchain_framebuffers, render_pass, swapchain_stuff.swapchain_extent, vertex_buffer, index_buffer, pipeline_layout, &descriptor_sets);
        let sync_ojbects = share::v1::create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp23 {
            // winit stuff
            window,

            // vulkan stuff
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_report_loader,
            debug_callback,

            physical_device,
            device,

            queue_family,
            graphics_queue,
            present_queue,

            swapchain_loader: swapchain_stuff.swapchain_loader,
            swapchain:        swapchain_stuff.swapchain,
            swapchain_format: swapchain_stuff.swapchain_format,
            swapchain_images: swapchain_stuff.swapchain_images,
            swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,
            swapchain_framebuffers,

            pipeline_layout,
            ubo_layout,
            render_pass,
            graphics_pipeline,

            texture_image,
            texture_image_memory,

            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,

            uniform_transform: UniformBufferObject {
                model: Matrix4::<f32>::identity(),
                view: Matrix4::look_at(Point3::new(2.0, 2.0, 2.0), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                proj: cgmath::perspective(Deg(45.0), swapchain_stuff.swapchain_extent.width as f32 / swapchain_stuff.swapchain_extent.height as f32, 0.1, 10.0),
            },
            uniform_buffers,
            uniform_buffers_memory,

            descriptor_pool,
            descriptor_sets,

            command_pool,
            command_buffers,

            image_available_semaphores: sync_ojbects.image_available_semaphores,
            render_finished_semaphores: sync_ojbects.render_finished_semaphores,
            in_flight_fences          : sync_ojbects.inflight_fences,
            current_frame: 0,

            is_framebuffer_resized: false,
        }
    }

    fn create_texture_image(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, image_path: &Path) -> (vk::Image, vk::DeviceMemory) {

        let mut image_object = image::open(image_path).unwrap(); // this function is slow in debug mode.
        image_object = image_object.flipv();
        let (image_width, image_height) = (image_object.width(), image_object.height());
        let image_size = (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        let image_data = match &image_object {
            | image::DynamicImage::ImageLuma8(_)
            | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba().into_raw(),
            | image::DynamicImage::ImageLumaA8(_)
            | image::DynamicImage::ImageRgba8(_) => image_object.raw_pixels(),
        };

        if image_size <= 0 {
            panic!("Failed to load texture image!")
        }

        let (staging_buffer, staging_buffer_memory) = share::create_buffer(
            device,
            image_size,
            vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
            vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
            device_memory_properties
        );

        unsafe {
            let data_ptr = device.map_memory(staging_buffer_memory, 0, image_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory");
            let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<u8>() as u64, image_size);
            align.copy_from_slice(&image_data);
            device.unmap_memory(staging_buffer_memory);
        }

        let (texture_image, texture_image_memory) = VulkanApp23::create_image(
            device,
            image_width, image_height,
            vk::Format::R8g8b8a8Unorm,
            vk::ImageTiling::Optimal,
            vk::IMAGE_USAGE_TRANSFER_DST_BIT | vk::IMAGE_USAGE_SAMPLED_BIT,
            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            device_memory_properties
        );

        VulkanApp23::transition_image_layout(device, command_pool, submit_queue, texture_image, vk::Format::R8g8b8a8Unorm, vk::ImageLayout::Undefined, vk::ImageLayout::TransferDstOptimal);

        VulkanApp23::copy_buffer_to_image(device, command_pool, submit_queue, staging_buffer, texture_image, image_width, image_height);

        VulkanApp23::transition_image_layout(device, command_pool, submit_queue, texture_image, vk::Format::R8g8b8a8Unorm, vk::ImageLayout::TransferDstOptimal, vk::ImageLayout::ShaderReadOnlyOptimal);

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (texture_image, texture_image_memory)
    }

    fn create_image(device: &ash::Device<V1_0>, width: uint32_t, height: uint32_t, format: vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, required_memory_properties: vk::MemoryPropertyFlags, device_memory_properties: &vk::PhysicalDeviceMemoryProperties)
        -> (vk::Image, vk::DeviceMemory) {

        let image_create_info = vk::ImageCreateInfo {
            s_type                   : vk::StructureType::ImageCreateInfo,
            p_next                   : ptr::null(),
            flags                    : vk::ImageCreateFlags::empty(),
            image_type               : vk::ImageType::Type2d,
            format,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels               : 1,
            array_layers             : 1,
            samples                  : vk::SAMPLE_COUNT_1_BIT,
            tiling,
            usage,
            sharing_mode             : vk::SharingMode::Exclusive,
            queue_family_index_count : 0,
            p_queue_family_indices   : ptr::null(),
            initial_layout           : vk::ImageLayout::Undefined,
        };

        let texture_image = unsafe {
            device.create_image(&image_create_info, None)
                .expect("Failed to create Texture Image!")
        };

        let image_memory_requirement = device.get_image_memory_requirements(texture_image);
        let memory_allocate_info = vk::MemoryAllocateInfo {
            s_type            : vk::StructureType::MemoryAllocateInfo,
            p_next            : ptr::null(),
            allocation_size   : image_memory_requirement.size,
            memory_type_index : share::find_memory_type(image_memory_requirement.memory_type_bits, required_memory_properties, device_memory_properties)
        };

        let texture_image_memory = unsafe {
            device.allocate_memory(&memory_allocate_info, None)
                .expect("Failed to allocate Texture Image memory!")
        };

        unsafe {
            device.bind_image_memory(texture_image, texture_image_memory, 0)
                .expect("Failed to bind Image Memmory!");
        }

        (texture_image, texture_image_memory)
    }

    fn begin_single_time_command(device: &ash::Device<V1_0>, command_pool: vk::CommandPool) -> vk::CommandBuffer {

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type               : vk::StructureType::CommandBufferAllocateInfo,
            p_next               : ptr::null(),
            command_buffer_count : 1,
            command_pool,
            level                : vk::CommandBufferLevel::Primary,
        };

        let command_buffer = unsafe {
            device.allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        }[0];

        let command_buffer_begin_info  = vk::CommandBufferBeginInfo {
            s_type             : vk::StructureType::CommandBufferBeginInfo,
            p_next             : ptr::null(),
            p_inheritance_info : ptr::null(),
            flags              : vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
        };

        unsafe {
            device.begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        command_buffer
    }

    fn end_single_time_command(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, command_buffer: vk::CommandBuffer) {

        unsafe {
            device.end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }

        let buffers_to_submit = [
            command_buffer,
        ];

        let submit_infos = [
            vk::SubmitInfo {
                s_type                 : vk::StructureType::SubmitInfo,
                p_next                 : ptr::null(),
                wait_semaphore_count   : 0,
                p_wait_semaphores      : ptr::null(),
                p_wait_dst_stage_mask  : ptr::null(),
                command_buffer_count   : 1,
                p_command_buffers      : buffers_to_submit.as_ptr(),
                signal_semaphore_count : 0,
                p_signal_semaphores    : ptr::null(),
            },
        ];

        unsafe {
            device.queue_submit(submit_queue, &submit_infos, vk::Fence::null())
                .expect("Failed to Queue Submit!");
            device.queue_wait_idle(submit_queue)
                .expect("Failed to wait Queue idle!");
            device.free_command_buffers(command_pool, &buffers_to_submit);
        }
    }

    fn transition_image_layout(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, image: vk::Image, _format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) {

        let command_buffer = VulkanApp23::begin_single_time_command(device, command_pool);

        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::Undefined && new_layout == vk::ImageLayout::TransferDstOptimal {

            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::ACCESS_TRANSFER_WRITE_BIT;
            source_stage = vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT;
            destination_stage = vk::PIPELINE_STAGE_TRANSFER_BIT;
        } else if old_layout == vk::ImageLayout::TransferDstOptimal && new_layout == vk::ImageLayout::ShaderReadOnlyOptimal {

            src_access_mask = vk::ACCESS_TRANSFER_WRITE_BIT;
            dst_access_mask = vk::ACCESS_SHADER_READ_BIT;
            source_stage = vk::PIPELINE_STAGE_TRANSFER_BIT;
            destination_stage = vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [
            vk::ImageMemoryBarrier {
                s_type                 : vk::StructureType::ImageMemoryBarrier,
                p_next                 : ptr::null(),
                src_access_mask,
                dst_access_mask,
                old_layout,
                new_layout,
                src_queue_family_index : vk::VK_QUEUE_FAMILY_IGNORED,
                dst_queue_family_index : vk::VK_QUEUE_FAMILY_IGNORED,
                image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask      : vk::IMAGE_ASPECT_COLOR_BIT,
                    base_mip_level   : 0,
                    level_count      : 1,
                    base_array_layer : 0,
                    layer_count      : 1,
                }
            },
        ];

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                source_stage, destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers
            );
        }

        VulkanApp23::end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    fn copy_buffer_to_image(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, buffer: vk::Buffer, image: vk::Image, width: uint32_t, height: uint32_t) {

        let command_buffer = VulkanApp23::begin_single_time_command(device, command_pool);

        let buffer_image_regions = [
            vk::BufferImageCopy {
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask      : vk::IMAGE_ASPECT_COLOR_BIT,
                    mip_level        : 0,
                    base_array_layer : 0,
                    layer_count      : 1,
                },
                image_extent: vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                },
                buffer_offset       : 0,
                buffer_image_height : 0,
                buffer_row_length   : 0,
                image_offset        : vk::Offset3D { x: 0, y: 0, z: 0 },
            },
        ];

        unsafe {
            device.cmd_copy_buffer_to_image(command_buffer, buffer, image, vk::ImageLayout::TransferDstOptimal, &buffer_image_regions);
        }

        VulkanApp23::end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    fn create_command_buffers(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, graphics_pipeline: vk::Pipeline, framebuffers: &Vec<vk::Framebuffer>, render_pass: vk::RenderPass, surface_extent: vk::Extent2D, vertex_buffer: vk::Buffer, index_buffer: vk::Buffer, pipeline_layout: vk::PipelineLayout, descriptor_sets: &Vec<vk::DescriptorSet>) -> Vec<vk::CommandBuffer> {

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type               : vk::StructureType::CommandBufferAllocateInfo,
            p_next               : ptr::null(),
            command_buffer_count : framebuffers.len() as u32,
            command_pool,
            level                : vk::CommandBufferLevel::Primary,
        };

        let command_buffers = unsafe {
            device.allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {

            let command_buffer_begin_info  = vk::CommandBufferBeginInfo {
                s_type             : vk::StructureType::CommandBufferBeginInfo,
                p_next             : ptr::null(),
                p_inheritance_info : ptr::null(),
                flags              : vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
            };

            unsafe {
                device.begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Failed to begin recording Command Buffer at beginning!");
            }

            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0]
                    },
                }
            ];

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type            : vk::StructureType::RenderPassBeginInfo,
                p_next            : ptr::null(),
                render_pass,
                framebuffer       : framebuffers[i],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: surface_extent,
                },
                clear_value_count : clear_values.len() as u32,
                p_clear_values    : clear_values.as_ptr(),
            };

            unsafe {
                device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::Inline);
                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::Graphics, graphics_pipeline);

                let vertex_buffers = [
                    vertex_buffer
                ];
                let offsets = [
                    0_u64
                ];
                let descriptor_sets_to_bind = [
                    descriptor_sets[i],
                ];

                device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::Uint32);
                device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::Graphics, pipeline_layout, 0, &descriptor_sets_to_bind, &[]);

                device.cmd_draw_indexed(command_buffer, RECT_INDICES_DATA.len() as u32, 1, 0, 0, 0);

                device.cmd_end_render_pass(command_buffer);

                device.end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        command_buffers
    }
}





// Fix content -------------------------------------------------------------------------------
impl VulkanApp23 {

    fn update_uniform_buffer(&mut self, current_image: usize, delta_time: f32) {

        self.uniform_transform.model = Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * delta_time) * self.uniform_transform.model;

        let ubos = [
            self.uniform_transform.clone(),
        ];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = self.device.map_memory(self.uniform_buffers_memory[current_image], 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory");
            let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<UniformBufferObject>() as u64, buffer_size);
            align.copy_from_slice(&ubos);
            self.device.unmap_memory(self.uniform_buffers_memory[current_image]);
        }
    }

    fn create_graphics_pipeline(device: &ash::Device<V1_0>, render_pass: vk::RenderPass, swapchain_extent: vk::Extent2D, ubo_set_layout: vk::DescriptorSetLayout) -> (vk::Pipeline, vk::PipelineLayout) {

        let vert_shader_code = utility::tools::read_shader_code(Path::new("shaders/spv/21-shader-ubo.vert.spv"));
        let frag_shader_code = utility::tools::read_shader_code(Path::new("shaders/spv/21-shader-ubo.frag.spv"));

        let vert_shader_module = share::create_shader_module(device, vert_shader_code);
        let frag_shader_module = share::create_shader_module(device, frag_shader_code);

        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo { // Vertex Shader
                s_type                : vk::StructureType::PipelineShaderStageCreateInfo,
                p_next                : ptr::null(),
                flags                 : vk::PipelineShaderStageCreateFlags::empty(),
                module                : vert_shader_module,
                p_name                : main_function_name.as_ptr(),
                p_specialization_info : ptr::null(),
                stage                 : vk::SHADER_STAGE_VERTEX_BIT,
            },
            vk::PipelineShaderStageCreateInfo { // Fragment Shader
                s_type                : vk::StructureType::PipelineShaderStageCreateInfo,
                p_next                : ptr::null(),
                flags                 : vk::PipelineShaderStageCreateFlags::empty(),
                module                : frag_shader_module,
                p_name                : main_function_name.as_ptr(),
                p_specialization_info : ptr::null(),
                stage                 : vk::SHADER_STAGE_FRAGMENT_BIT,
            },
        ];

        let binding_description   = VertexV1::get_binding_description();
        let attribute_description = VertexV1::get_attribute_descriptions();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type                             : vk::StructureType::PipelineVertexInputStateCreateInfo,
            p_next                             : ptr::null(),
            flags                              : vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count : attribute_description.len() as u32,
            p_vertex_attribute_descriptions    : attribute_description.as_ptr(),
            vertex_binding_description_count   : binding_description.len() as u32,
            p_vertex_binding_descriptions      : binding_description.as_ptr(),
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type                   : vk::StructureType::PipelineInputAssemblyStateCreateInfo,
            flags                    : vk::PipelineInputAssemblyStateCreateFlags::empty(),
            p_next                   : ptr::null(),
            primitive_restart_enable : vk::VK_FALSE,
            topology                 : vk::PrimitiveTopology::TriangleList,
        };

        let viewports = [
            vk::Viewport {
                x         : 0.0,
                y         : 0.0,
                width     : swapchain_extent.width as f32,
                height    : swapchain_extent.height as f32,
                min_depth : 0.0,
                max_depth : 1.0,
            },
        ];

        let scissors = [
            vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain_extent,
            },
        ];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type         : vk::StructureType::PipelineViewportStateCreateInfo,
            p_next         : ptr::null(),
            flags          : vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count  : scissors.len()  as u32,
            p_scissors     : scissors.as_ptr(),
            viewport_count : viewports.len() as u32,
            p_viewports    : viewports.as_ptr(),
        };

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type                     : vk::StructureType::PipelineRasterizationStateCreateInfo,
            p_next                     : ptr::null(),
            flags                      : vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable         : vk::VK_FALSE,
            cull_mode                  : vk::CULL_MODE_BACK_BIT,
            front_face                 : vk::FrontFace::Clockwise,
            line_width                 : 1.0,
            polygon_mode               : vk::PolygonMode::Fill,
            rasterizer_discard_enable  : vk::VK_FALSE,
            depth_bias_clamp           : 0.0,
            depth_bias_constant_factor : 0.0,
            depth_bias_enable          : vk::VK_FALSE,
            depth_bias_slope_factor    : 0.0,
        };

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type                   : vk::StructureType::PipelineMultisampleStateCreateInfo,
            flags                    : vk::PipelineMultisampleStateCreateFlags::empty(),
            p_next                   : ptr::null(),
            rasterization_samples    : vk::SAMPLE_COUNT_1_BIT,
            sample_shading_enable    : vk::VK_FALSE,
            min_sample_shading       : 0.0,
            p_sample_mask            : ptr::null(),
            alpha_to_one_enable      : vk::VK_FALSE,
            alpha_to_coverage_enable : vk::VK_FALSE,
        };

        let stencil_state = vk::StencilOpState {
            fail_op       : vk::StencilOp::Keep,
            pass_op       : vk::StencilOp::Keep,
            depth_fail_op : vk::StencilOp::Keep,
            compare_op    : vk::CompareOp::Always,
            compare_mask  : 0,
            write_mask    : 0,
            reference     : 0,
        };

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type                   : vk::StructureType::PipelineDepthStencilStateCreateInfo,
            p_next                   : ptr::null(),
            flags                    : vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable        : vk::VK_FALSE,
            depth_write_enable       : vk::VK_FALSE,
            depth_compare_op         : vk::CompareOp::LessOrEqual,
            depth_bounds_test_enable : vk::VK_FALSE,
            stencil_test_enable      : vk::VK_FALSE,
            front                    : stencil_state,
            back                     : stencil_state,
            max_depth_bounds         : 1.0,
            min_depth_bounds         : 0.0,
        };

        let color_blend_attachment_states = [
            vk::PipelineColorBlendAttachmentState {
                blend_enable           : vk::VK_FALSE,
                color_write_mask       : vk::ColorComponentFlags::all(),
                src_color_blend_factor : vk::BlendFactor::One,
                dst_color_blend_factor : vk::BlendFactor::Zero,
                color_blend_op         : vk::BlendOp::Add,
                src_alpha_blend_factor : vk::BlendFactor::One,
                dst_alpha_blend_factor : vk::BlendFactor::Zero,
                alpha_blend_op         : vk::BlendOp::Add,
            },
        ];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type           : vk::StructureType::PipelineColorBlendStateCreateInfo,
            p_next           : ptr::null(),
            flags            : vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable  : vk::VK_FALSE,
            logic_op         : vk::LogicOp::Copy,
            attachment_count : color_blend_attachment_states.len() as u32,
            p_attachments    : color_blend_attachment_states.as_ptr(),
            blend_constants  : [0.0, 0.0, 0.0, 0.0],
        };

        let set_layouts = [
            ubo_set_layout,
        ];

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type                    : vk::StructureType::PipelineLayoutCreateInfo,
            p_next                    : ptr::null(),
            flags                     : vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count          : set_layouts.len() as u32,
            p_set_layouts             : set_layouts.as_ptr(),
            push_constant_range_count : 0,
            p_push_constant_ranges    : ptr::null(),
        };

        let pipeline_layout = unsafe {
            device.create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout!")
        };

        let graphic_pipeline_create_infos = [
            vk::GraphicsPipelineCreateInfo {
                s_type                 : vk::StructureType::GraphicsPipelineCreateInfo,
                p_next                 : ptr::null(),
                flags                  : vk::PipelineCreateFlags::empty(),
                stage_count            : shader_stages.len() as u32,
                p_stages               : shader_stages.as_ptr(),
                p_vertex_input_state   : &vertex_input_state_create_info,
                p_input_assembly_state : &vertex_input_assembly_state_info,
                p_tessellation_state   : ptr::null(),
                p_viewport_state       : &viewport_state_create_info,
                p_rasterization_state  : &rasterization_statue_create_info,
                p_multisample_state    : &multisample_state_create_info,
                p_depth_stencil_state  : &depth_state_create_info,
                p_color_blend_state    : &color_blend_state,
                p_dynamic_state        : ptr::null(),
                layout                 : pipeline_layout,
                render_pass,
                subpass                : 0,
                base_pipeline_handle   : vk::Pipeline::null(),
                base_pipeline_index    : -1,
            },
        ];

        let graphics_pipelines = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &graphic_pipeline_create_infos, None)
                .expect("Failed to create Graphics Pipeline!.")
        };

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        (graphics_pipelines[0], pipeline_layout)
    }
}

impl Drop for VulkanApp23 {

    fn drop(&mut self) {

        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.cleanup_swapchain();

            self.device.destroy_descriptor_pool(self.descriptor_pool, None);

            for i in 0..self.uniform_buffers.len() {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device.free_memory(self.uniform_buffers_memory[i], None);
            }

            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

            self.device.destroy_image(self.texture_image, None);
            self.device.free_memory(self.texture_image_memory, None);

            self.device.destroy_descriptor_set_layout(self.ubo_layout, None);

            self.device.destroy_command_pool(self.command_pool, None);

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface_khr(self.surface, None);

            if VALIDATION.is_enable {
                self.debug_report_loader.destroy_debug_report_callback_ext(self.debug_callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanApp for VulkanApp23 {

    fn draw_frame(&mut self, delta_time: f32) {

        let wait_fences = [
            self.in_flight_fences[self.current_frame]
        ];

        unsafe {
            self.device.wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let image_index = unsafe {
            let result = self.swapchain_loader.acquire_next_image_khr(self.swapchain, std::u64::MAX, self.image_available_semaphores[self.current_frame], vk::Fence::null());
            match result {
                | Ok(image_index) => image_index,
                | Err(vk_result) => match vk_result {
                    | vk::types::Result::ErrorOutOfDateKhr => {
                        self.recreate_swapchain();
                        return
                    },
                    | _ => panic!("Failed to acquire Swap Chain Image!")
                }
            }
        };

        self.update_uniform_buffer(image_index as usize, delta_time);

        let wait_semaphores = [
            self.image_available_semaphores[self.current_frame],
        ];
        let wait_stages = [
            vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        ];
        let signal_semaphores = [
            self.render_finished_semaphores[self.current_frame],
        ];

        let submit_infos = [
            vk::SubmitInfo {
                s_type                 : vk::StructureType::SubmitInfo,
                p_next                 : ptr::null(),
                wait_semaphore_count   : wait_semaphores.len() as u32,
                p_wait_semaphores      : wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask  : wait_stages.as_ptr(),
                command_buffer_count   : 1,
                p_command_buffers      : &self.command_buffers[image_index as usize],
                signal_semaphore_count : signal_semaphores.len() as u32,
                p_signal_semaphores    : signal_semaphores.as_ptr(),
            }
        ];

        unsafe {
            self.device.reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device.queue_submit(self.graphics_queue, &submit_infos, self.in_flight_fences[self.current_frame])
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [
            self.swapchain
        ];

        let present_info = vk::PresentInfoKHR {
            s_type               : vk::StructureType::PresentInfoKhr,
            p_next               : ptr::null(),
            wait_semaphore_count : 1,
            p_wait_semaphores    : signal_semaphores.as_ptr(),
            swapchain_count      : 1,
            p_swapchains         : swapchains.as_ptr(),
            p_image_indices      : &image_index,
            p_results            : ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain_loader.queue_present_khr(self.present_queue, &present_info)
        };

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                | vk::Result::ErrorOutOfDateKhr
                | vk::Result::SuboptimalKhr => {
                    true
                }
                | _ => panic!("Failed to execute queue present.")
            }
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn recreate_swapchain(&mut self) {

        // parameters -------------
        let surface_suff = SurfaceStuff {
            surface_loader: self.surface_loader.clone(), surface: self.surface,
            screen_width: WINDOW_WIDTH, screen_height: WINDOW_HEIGHT,
        };
        // ------------------------

        self.device.device_wait_idle()
            .expect("Failed to wait device idle!");
        self.cleanup_swapchain();

        let swapchain_stuff = share::create_swapchain(&self.instance, &self.device, self.physical_device, &self.window, &surface_suff, &self.queue_family);
        self.swapchain_loader = swapchain_stuff.swapchain_loader;
        self.swapchain        = swapchain_stuff.swapchain;
        self.swapchain_images = swapchain_stuff.swapchain_images;
        self.swapchain_format = swapchain_stuff.swapchain_format;
        self.swapchain_extent = swapchain_stuff.swapchain_extent;

        self.swapchain_imageviews = share::v1::create_image_views(&self.device, self.swapchain_format, &self.swapchain_images);
        self.render_pass = share::v1::create_render_pass(&self.device, self.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = VulkanApp23::create_graphics_pipeline(&self.device, self.render_pass, swapchain_stuff.swapchain_extent, self.ubo_layout);
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        self.swapchain_framebuffers = share::v1::create_framebuffers(&self.device, self.render_pass, &self.swapchain_imageviews, self.swapchain_extent);
        self.command_buffers = VulkanApp23::create_command_buffers(&self.device, self.command_pool, self.graphics_pipeline, &self.swapchain_framebuffers, self.render_pass, self.swapchain_extent, self.vertex_buffer, self.index_buffer, self.pipeline_layout, &self.descriptor_sets);
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.device.free_command_buffers(self.command_pool, &self.command_buffers);
            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader.destroy_swapchain_khr(self.swapchain, None);
        }
    }

    fn wait_device_idle(&self) {
        self.device.device_wait_idle()
            .expect("Failed to wait device idle!");
    }

    fn resize_framebuffer(&mut self) {
        self.is_framebuffer_resized = true;
    }
}

fn main() {

    let mut program_proc = ProgramProc::new();
    let mut vulkan_app = VulkanApp23::new(&program_proc.events_loop);

    program_proc.main_loop(&mut vulkan_app);
}
// -------------------------------------------------------------------------------------------
