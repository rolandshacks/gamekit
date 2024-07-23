//!
//! Pipeline
//!

use ash::vk::Handle;
use ash::vk;

use log::{*};

use crate::api::Disposable;
use crate::constants::Constants;
use crate::error::Error;
use crate::device::Device;
use crate::image::{Image, ImageView};
use crate::instance::Instance;
use crate::swapchain::SwapChain;
use crate::types::{CommandBuffer, Frame, Framebuffer};

pub struct ImageViewsInfo {
    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>
}

pub struct DepthBufferInfo {
    pub depth_image: Image,
    pub depth_image_view: ImageView
}

pub struct RenderPassInfo {
    pub render_pass: vk::RenderPass
}

pub struct FramebufferInfo {
    pub frame_buffers: Vec<Framebuffer>
}

pub struct FramesInfo {
    pub frames: Vec<Frame>
}

pub struct Pipeline {
    pub swapchain: SwapChain,
    pub images: Vec<crate::image::Image>,
    pub image_views: Vec<crate::image::ImageView>,
    pub depth_image: crate::image::Image,
    pub depth_image_view: crate::image::ImageView,
    pub render_pass: ash::vk::RenderPass,
    pub frame_buffers: Vec<crate::types::Framebuffer>,
    pub frames: Vec<crate::types::Frame>,
    pub frame_count: usize,
    pub frame_index: usize,

    image_index: u32,
    need_reinit: bool
}

impl Disposable for Pipeline {
    fn dispose(&mut self) {
        trace!("Pipeline::dispose");
        self.destroy_pipeline();
    }
}

impl Pipeline {

    pub fn new() -> Result<Self, Error> {

        let device = crate::globals::device();
        let instance = crate::globals::instance();

        let swapchain = SwapChain::new()?;
        let image_views_info = Pipeline::create_image_views(&device, &swapchain)?;
        let depth_buffer_info = Pipeline::create_depth_buffer(&instance, &device, &swapchain)?;
        let render_pass_info = Pipeline::create_render_pass(&device, &swapchain, depth_buffer_info.depth_image.format)?;
        let frame_buffer_info = Pipeline::create_frame_buffers(&device, &swapchain, &image_views_info, &depth_buffer_info, &render_pass_info)?;

        let frames_info = Pipeline::create_frames(&device)?;
        let frame_count = frames_info.frames.len();

        Ok(Self {
            swapchain,
            images: image_views_info.images,
            image_views: image_views_info.image_views,
            depth_image: depth_buffer_info.depth_image,
            depth_image_view: depth_buffer_info.depth_image_view,
            render_pass: render_pass_info.render_pass,
            frame_buffers: frame_buffer_info.frame_buffers,
            frames: frames_info.frames,
            frame_count,
            frame_index: 0,
            image_index: 0,
            need_reinit: false
        })

    }

    fn create_pipeline(&mut self) -> Result<(), Error> {

        let device = crate::globals::device();
        let instance = crate::globals::instance();

        let swapchain = SwapChain::new()?;
        let image_views_info = Pipeline::create_image_views(&device, &swapchain)?;
        let depth_buffer_info = Pipeline::create_depth_buffer(&instance, &device, &swapchain)?;
        let render_pass_info = Pipeline::create_render_pass(&device, &swapchain, depth_buffer_info.depth_image.format)?;
        let frame_buffer_info = Pipeline::create_frame_buffers(&device, &swapchain, &image_views_info, &depth_buffer_info, &render_pass_info)?;

        let frames_info = Pipeline::create_frames(&device)?;
        let frame_count = frames_info.frames.len();

        self.swapchain = swapchain;
        self.images = image_views_info.images;
        self.image_views =  image_views_info.image_views;
        self.depth_image = depth_buffer_info.depth_image;
        self.depth_image_view = depth_buffer_info.depth_image_view;
        self.render_pass = render_pass_info.render_pass;
        self.frame_buffers = frame_buffer_info.frame_buffers;
        self.frames = frames_info.frames;
        self.frame_count = frame_count;
        self.frame_index = 0;
        self.image_index = 0;
        self.need_reinit = false;

        Ok(())
    }

    fn destroy_pipeline(&mut self) {
        Self::wait_idle();

        self.destroy_frames();
        self.destroy_frame_buffers();
        self.destroy_render_pass();
        self.destroy_depth_buffer();
        self.destroy_image_views();

        self.swapchain.dispose();
    }

    pub fn reinit(&mut self) -> Result<(), Error> {
        self.destroy_pipeline();
        self.create_pipeline()?;

        Ok(())
    }

    fn wait_idle() {
        let device = crate::globals::device();
        unsafe { let _ = device.obj.device_wait_idle(); }
    }

    fn create_image_views(device_context: &Device, swapchain: &SwapChain) -> Result<ImageViewsInfo, Error> {

        let swapchain_device = &swapchain.device;
        let swapchain = swapchain.obj;

        let images_handles = unsafe { swapchain_device.get_swapchain_images(swapchain).unwrap() };

        let mut images: Vec<Image> = vec![];
        let mut image_views: Vec<ImageView> = vec![];

        for image_handle in images_handles {
            let image = Image::attach(image_handle, Image::PIXEL_BUFFER, device_context.surface_format.format)?;
            let image_view = ImageView::create_ex(&device_context.obj, &image);
            images.push(image);
            image_views.push(image_view);
        }

        Ok(ImageViewsInfo {
            images,
            image_views
        })
    }

    fn destroy_image_views(&mut self) {

        for item in &mut self.image_views {
            item.dispose();
        }

        self.image_views.clear();

        for item in &mut self.images {
            item.detach(); // don't destroy, image has been acquired by swap chain
        }

        self.images.clear();
    }

    fn create_depth_buffer(instance: &Instance, device_context: &Device, swapchain: &SwapChain) -> Result<DepthBufferInfo, Error> {

        let supported_formats = vec![
            vk::Format::D24_UNORM_S8_UINT, vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT,
        ];

        let mut depth_format = vk::Format::UNDEFINED;

        for format in supported_formats {
            let properties = unsafe { instance.obj.get_physical_device_format_properties(device_context.physical_device, format) };
            if properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
                depth_format = format;
                break;
            }
        }

        if depth_format == vk::Format::UNDEFINED {
            return Err(Error::from("failed to find supported depth buffer format"));
        }

        let bytes_per_pixel = 4u32;
        let depth_image_size = swapchain.extent.width * swapchain.extent.height * bytes_per_pixel;

        let depth_image = Image::create(Image::DEPTH_BUFFER, swapchain.extent.width, swapchain.extent.height, depth_image_size as usize, depth_format)?;
        let depth_image_view = ImageView::create_ex(&device_context.obj, &depth_image);

        Ok(DepthBufferInfo {
            depth_image,
            depth_image_view
        })

    }

    fn destroy_depth_buffer(&mut self) {
        self.depth_image_view.dispose();
        self.depth_image.dispose();
    }

    fn create_render_pass(device_context: &Device, swapchain_info: &SwapChain, depth_buffer_format: vk::Format) -> Result<RenderPassInfo, Error> {

        let color_attachment = vk::AttachmentDescription::default()
            .format(swapchain_info.format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);


        let depth_attachment = vk::AttachmentDescription::default()
            .format(depth_buffer_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let color_attachments = vec![color_attachment_ref];

        let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let attachments = vec![
            color_attachment,
            depth_attachment
        ];

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .depth_stencil_attachment(&depth_attachment_ref);

        let subpasses = vec![subpass];

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0u32)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::NONE)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

        let dependencies = vec![dependency];

        let render_pass_create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe { device_context.obj.create_render_pass(&render_pass_create_info, None).unwrap() };

        Ok(RenderPassInfo {
            render_pass
        })
    }

    fn destroy_render_pass(&mut self) {
        if !self.render_pass.is_null() {
            let device = crate::globals::device();
            unsafe { device.obj.destroy_render_pass(self.render_pass, None ); }
            self.render_pass = vk::RenderPass::null();
        }
    }

    fn create_frame_buffers(device_context: &Device, swapchain: &SwapChain, image_views_info: &ImageViewsInfo, depth_buffer_info: &DepthBufferInfo, render_pass_info: &RenderPassInfo) -> Result<FramebufferInfo, Error> {

        let width = swapchain.extent.width;
        let height = swapchain.extent.height;

        let mut frame_buffers: Vec<crate::types::Framebuffer> = vec![];

        for image_view in &image_views_info.image_views {

            let frame_buffer = Framebuffer::new(
                &device_context.obj,
                render_pass_info.render_pass,
                image_view.obj,
                depth_buffer_info.depth_image_view.obj,
                width,
                height
            )?;

            frame_buffers.push(frame_buffer);

        }

        Ok(FramebufferInfo {
            frame_buffers
        })
    }

    fn destroy_frame_buffers(&mut self) {
        Self::wait_idle();

        for frame_buffer in &mut self.frame_buffers {
            frame_buffer.dispose();
        }

        self.frame_buffers.clear();
    }

    pub fn current_frame(&self) -> &Frame {
        &self.frames[self.frame_index]
    }

    pub fn current_command_buffer(&self) -> &CommandBuffer {
        let frame = self.current_frame();
        &frame.command_buffer
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    fn create_frames(device_context: &Device) -> Result <FramesInfo, Error> {

        let num_frames = Constants::FRAME_BUFFER_COUNT;
        let mut frames: Vec<Frame> = vec![];

        for i in 0..num_frames {
            let frame = Frame::new(device_context, i as u32)?;
            frames.push(frame);
        }

        Ok(FramesInfo {
            frames
        })
    }

    fn destroy_frames(&mut self) {
        Self::wait_idle();

        for frame in &mut self.frames {
            frame.dispose();
        }

    }

    pub fn render_pass(&self) -> &vk::RenderPass {
        &self.render_pass
    }

    pub fn begin_frame(&mut self) -> Result<bool, Error> {

        let mut reinitialized = false;

        if self.need_reinit {
            self.reinit()?;
            reinitialized = true;
        }

        let swapchain = &self.swapchain;
        let frame = self.current_frame();

        let mut needs_reinit = false;
        let image_index = unsafe {
            frame.command_buffers_completed.wait(u64::MAX);

            match self.swapchain.device.acquire_next_image(
                self.swapchain.obj,
                u64::MAX,
                frame.image_available.obj,
                ash::vk::Fence::null()
            ) {
                Ok((idx, is_suboptimal)) => {
                    if is_suboptimal {
                        needs_reinit = true;
                        0
                    } else {
                        idx
                    }
                },
                Err(_) => {
                    needs_reinit = true;
                    0
                }
            }

        };

        frame.command_buffers_completed.reset();

        if needs_reinit {
            return Err(Error::from("pipeline needs to be reinitialized"))
        }

        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
        ];

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.frame_buffers[image_index as usize].obj)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D{x:0,y:0},
                extent: vk::Extent2D{
                    width:swapchain.extent.width,
                    height:swapchain.extent.height
                }
            })
            .clear_values(&clear_values);

        unsafe {

            let device = crate::globals::device();

            let command_buffer = &frame.command_buffer;
            command_buffer.reset();
            command_buffer.begin();

            device.obj.cmd_begin_render_pass(
                command_buffer.obj,
                &render_pass_info,
                vk::SubpassContents::INLINE);

        };

        self.image_index = image_index;

        Ok(reinitialized)

    }

    pub fn end_frame(&mut self) -> Result<(), Error> {

        if self.need_reinit {
            return Err(Error::from("pipeline needs to be reinitialized"));
        }

        let swapchain: &_ = &self.swapchain;
        let frame = self.current_frame();
        let command_buffer = &frame.command_buffer;

        let device = crate::globals::device();
        unsafe { device.obj.cmd_end_render_pass(command_buffer.obj) };
        command_buffer.end();

        let wait_semaphores = [ frame.image_available.obj ];
        let wait_stages = [ vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT ];
        let signal_semaphores = [ frame.render_finished.obj ];
        let command_buffers = [ command_buffer.obj ];
        let swapchains = [ swapchain.obj ];
        let image_indices = [ self.image_index ];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .signal_semaphores(&signal_semaphores)
            .command_buffers(&command_buffers);

        let submit_infos = [ submit_info ];

        let device = crate::globals::device();
        unsafe { device.obj.queue_submit(device.graphics_queue, &submit_infos, frame.command_buffers_completed.obj).unwrap() };

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        self.need_reinit = unsafe {
            match swapchain.device.queue_present(device.present_queue, &present_info) {
                Ok(is_suboptimal) => {
                    is_suboptimal
                },
                Err(_) => {
                    true
                }
            }
        };

        if !self.need_reinit {
            self.frame_index = (self.frame_index + 1) % self.frame_count;
        }

        Ok(())

    }

}
