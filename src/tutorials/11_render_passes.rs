
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::share,
    utility::debug::*,
    utility::constants::*,
};

extern crate winit;
extern crate ash;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::InstanceV1_0;
use ash::version::DeviceV1_0;

use std::path::Path;
use std::ptr;
use std::ffi::CString;

// Constants
const WINDOW_TITLE: &'static str = "11.Render Passes";
struct VulkanApp {
    // winit stuff
    events_loop          : EventsLoop,
    _window              : winit::Window,

    // vulkan stuff
    _entry               : ash::Entry,
    instance             : ash::Instance,
    surface_loader       : ash::extensions::Surface,
    surface              : vk::SurfaceKHR,
    debug_report_loader  : ash::extensions::DebugReport,
    debug_callback       : vk::DebugReportCallbackEXT,

    _physical_device     : vk::PhysicalDevice,
    device               : ash::Device,

    _graphics_queue      : vk::Queue,
    _present_queue       : vk::Queue,

    swapchain_loader     : ash::extensions::Swapchain,
    swapchain            : vk::SwapchainKHR,
    _swapchain_images    : Vec<vk::Image>,
    _swapchain_format    : vk::Format,
    _swapchain_extent    : vk::Extent2D,
    swapchain_imageviews : Vec<vk::ImageView>,

    render_pass          : vk::RenderPass,
    pipeline_layout      : vk::PipelineLayout,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = utility::window::init_window(&events_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = ash::Entry::new().unwrap();
        let instance = share::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = share::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_report_loader, debug_callback) = setup_debug_callback( VALIDATION.is_enable, &entry, &instance);
        let physical_device = share::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let (device, family_indices) = share::create_logical_device(&instance, physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(family_indices.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(family_indices.present_family as u32, 0) };
        let swapchain_stuff = share::create_swapchain(&instance, &device, physical_device, &window, &surface_stuff, &family_indices);
        let swapchain_imageviews = share::v1::create_image_views(&device, swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let render_pass = VulkanApp::create_render_pass(&device, swapchain_stuff.swapchain_format);
        let pipeline_layout = VulkanApp::create_graphics_pipeline(&device, swapchain_stuff.swapchain_extent);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            // winit stuff
            events_loop,
            _window: window,

            // vulkan stuff
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_report_loader,
            debug_callback,

            _physical_device: physical_device,
            device,

            _graphics_queue: graphics_queue,
            _present_queue : present_queue,

            swapchain_loader : swapchain_stuff.swapchain_loader,
            swapchain        : swapchain_stuff.swapchain,
            _swapchain_format: swapchain_stuff.swapchain_format,
            _swapchain_images: swapchain_stuff.swapchain_images,
            _swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,

            pipeline_layout,
            render_pass,
        }
    }

    fn create_graphics_pipeline(device: &ash::Device, swapchain_extent: vk::Extent2D) -> vk::PipelineLayout {

        let vert_shader_code = utility::tools::read_shader_code(Path::new("shaders/spv/09-shader-base.vert.spv"));
        let frag_shader_code = utility::tools::read_shader_code(Path::new("shaders/spv/09-shader-base.frag.spv"));

        let vert_shader_module = share::create_shader_module(device, vert_shader_code);
        let frag_shader_module = share::create_shader_module(device, frag_shader_code);

        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let _shader_stages = [
            vk::PipelineShaderStageCreateInfo { // Vertex Shader
                s_type                : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next                : ptr::null(),
                flags                 : vk::PipelineShaderStageCreateFlags::empty(),
                module                : vert_shader_module,
                p_name                : main_function_name.as_ptr(),
                p_specialization_info : ptr::null(),
                stage                 : vk::ShaderStageFlags::VERTEX,
            },
            vk::PipelineShaderStageCreateInfo { // Fragment Shader
                s_type                : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next                : ptr::null(),
                flags                 : vk::PipelineShaderStageCreateFlags::empty(),
                module                : frag_shader_module,
                p_name                : main_function_name.as_ptr(),
                p_specialization_info : ptr::null(),
                stage                 : vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        let _vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type                             : vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next                             : ptr::null(),
            flags                              : vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count : 0,
            p_vertex_attribute_descriptions    : ptr::null(),
            vertex_binding_description_count   : 0,
            p_vertex_binding_descriptions      : ptr::null(),
        };
        let _vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type                   : vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            flags                    : vk::PipelineInputAssemblyStateCreateFlags::empty(),
            p_next                   : ptr::null(),
            primitive_restart_enable : vk::FALSE,
            topology                 : vk::PrimitiveTopology::TRIANGLE_LIST,
        };

        let viewports = [
            vk::Viewport {
                x         : 0.0,
                y         : 0.0,
                width     : swapchain_extent.width  as f32,
                height    : swapchain_extent.height as f32,
                min_depth : 0.0,
                max_depth : 1.0,
            },
        ];

        let scissors = [
            vk::Rect2D {
                offset : vk::Offset2D { x: 0, y: 0 },
                extent : swapchain_extent,
            },
        ];

        let _viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type         : vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next         : ptr::null(),
            flags          : vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count  : scissors.len()  as u32,
            p_scissors     : scissors.as_ptr(),
            viewport_count : viewports.len() as u32,
            p_viewports    : viewports.as_ptr(),
        };

        let _rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type                     : vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next                     : ptr::null(),
            flags                      : vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable         : vk::FALSE,
            cull_mode                  : vk::CullModeFlags::BACK,
            front_face                 : vk::FrontFace::CLOCKWISE,
            line_width                 : 1.0,
            polygon_mode               : vk::PolygonMode::FILL,
            rasterizer_discard_enable  : vk::FALSE,
            depth_bias_clamp           : 0.0,
            depth_bias_constant_factor : 0.0,
            depth_bias_enable          : vk::FALSE,
            depth_bias_slope_factor    : 0.0,
        };
        let _multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type                   : vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            flags                    : vk::PipelineMultisampleStateCreateFlags::empty(),
            p_next                   : ptr::null(),
            rasterization_samples    : vk::SampleCountFlags::TYPE_1,
            sample_shading_enable    : vk::FALSE,
            min_sample_shading       : 0.0,
            p_sample_mask            : ptr::null(),
            alpha_to_one_enable      : vk::FALSE,
            alpha_to_coverage_enable : vk::FALSE,
        };

        let stencil_state = vk::StencilOpState {
            fail_op       : vk::StencilOp::KEEP,
            pass_op       : vk::StencilOp::KEEP,
            depth_fail_op : vk::StencilOp::KEEP,
            compare_op    : vk::CompareOp::ALWAYS,
            compare_mask  : 0,
            write_mask    : 0,
            reference     : 0,
        };

        let _depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type                   : vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next                   : ptr::null(),
            flags                    : vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable        : vk::FALSE,
            depth_write_enable       : vk::FALSE,
            depth_compare_op         : vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable : vk::FALSE,
            stencil_test_enable      : vk::FALSE,
            front                    : stencil_state,
            back                     : stencil_state,
            max_depth_bounds         : 1.0,
            min_depth_bounds         : 0.0,
        };

        let color_blend_attachment_states = [
            vk::PipelineColorBlendAttachmentState {
                blend_enable           : vk::FALSE,
                color_write_mask       : vk::ColorComponentFlags::all(),
                src_color_blend_factor : vk::BlendFactor::ONE,
                dst_color_blend_factor : vk::BlendFactor::ZERO,
                color_blend_op         : vk::BlendOp::ADD,
                src_alpha_blend_factor : vk::BlendFactor::ONE,
                dst_alpha_blend_factor : vk::BlendFactor::ZERO,
                alpha_blend_op         : vk::BlendOp::ADD,
            },
        ];

        let _color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type           : vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next           : ptr::null(),
            flags            : vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable  : vk::FALSE,
            logic_op         : vk::LogicOp::COPY,
            attachment_count : color_blend_attachment_states.len() as u32,
            p_attachments    : color_blend_attachment_states.as_ptr(),
            blend_constants  : [0.0, 0.0, 0.0, 0.0],
        };

        //        leaving the dynamic statue unconfigurated right now
        //        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        //        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
        //            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        //            p_next: ptr::null(),
        //            flags: vk::PipelineDynamicStateCreateFlags::empty(),
        //            dynamic_state_count: dynamic_state.len() as u32,
        //            p_dynamic_states: dynamic_state.as_ptr(),
        //        };

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type                    : vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next                    : ptr::null(),
            flags                     : vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count          : 0,
            p_set_layouts             : ptr::null(),
            push_constant_range_count : 0,
            p_push_constant_ranges    : ptr::null(),
        };

        let pipeline_layout = unsafe {
            device.create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout!")
        };

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        pipeline_layout
    }

    fn create_render_pass(device: &ash::Device, surface_format: vk::Format) -> vk::RenderPass {

        let color_attachment = vk::AttachmentDescription {
            flags            : vk::AttachmentDescriptionFlags::empty(),
            format           : surface_format,
            samples          : vk::SampleCountFlags::TYPE_1,
            load_op          : vk::AttachmentLoadOp::CLEAR,
            store_op         : vk::AttachmentStoreOp::STORE,
            stencil_load_op  : vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op : vk::AttachmentStoreOp::DONT_CARE,
            initial_layout   : vk::ImageLayout::UNDEFINED,
            final_layout     : vk::ImageLayout::PRESENT_SRC_KHR,
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment : 0,
            layout     : vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            flags                      : vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point        : vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count     : 0,
            p_input_attachments        : ptr::null(),
            color_attachment_count     : 1,
            p_color_attachments        : &color_attachment_ref,
            p_resolve_attachments      : ptr::null(),
            p_depth_stencil_attachment : ptr::null(),
            preserve_attachment_count  : 0,
            p_preserve_attachments     : ptr::null(),
        };

        let render_pass_attachments = [
            color_attachment,
        ];

        let renderpass_create_info = vk::RenderPassCreateInfo {
            s_type           : vk::StructureType::RENDER_PASS_CREATE_INFO,
            flags            : vk::RenderPassCreateFlags::empty(),
            p_next           : ptr::null(),
            attachment_count : render_pass_attachments.len() as u32,
            p_attachments    : render_pass_attachments.as_ptr(),
            subpass_count    : 1,
            p_subpasses      : &subpass,
            dependency_count : 0,
            p_dependencies   : ptr::null(),
        };

        unsafe {
            device.create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        }
    }
}

impl Drop for VulkanApp {

    fn drop(&mut self) {

        unsafe {

            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &imageview in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(imageview, None);
            }

            self.swapchain_loader.destroy_swapchain_khr(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface_khr(self.surface, None);

            if VALIDATION.is_enable {
                self.debug_report_loader.destroy_debug_report_callback_ext(self.debug_callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}




// Fix content -------------------------------------------------------------------------------
impl VulkanApp {

    pub fn main_loop(&mut self) {

        self.events_loop.run_forever(|event| {

            match event {
                // handling keyboard event
                | Event::WindowEvent { event, .. } => match event {
                    | WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            ControlFlow::Break
                        } else {
                            ControlFlow::Continue
                        }
                    }
                    | WindowEvent::CloseRequested => ControlFlow::Break,
                    | _ => ControlFlow::Continue,
                },
                | _ => ControlFlow::Continue,
            }
        });
    }
}

fn main() {

    let mut vulkan_app = VulkanApp::new();
    vulkan_app.main_loop();
}
// -------------------------------------------------------------------------------------------
