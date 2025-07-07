use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use wgpu::SurfaceTarget;
use anyhow::Result;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use raw_window_handle::{
    HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, WebDisplayHandle, WebWindowHandle
};// A new function to load binary data from a URL.
async fn fetch_binary(url: &str) -> Result<Vec<u8>, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    let buffer = JsFuture::from(resp.array_buffer()?).await?;
    let u8_array = js_sys::Uint8Array::new(&buffer);
    Ok(u8_array.to_vec())
}
#[wasm_bindgen]
pub struct CanvasProxy(HtmlCanvasElement);

#[wasm_bindgen]
impl CanvasProxy {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        CanvasProxy(canvas)
    }
}
unsafe impl Send for CanvasProxy {}
unsafe impl Sync for CanvasProxy {}
impl Clone for CanvasProxy {
    fn clone(&self) -> Self {
        CanvasProxy(self.0.clone())
    }
}
// CanvasProxy 타입에 대해 트레잇을 구현합니다.
impl HasWindowHandle for CanvasProxy {
    fn window_handle(
        &self,
    ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, HandleError> {
     // self.0을 js_sys::Object로 명확하게 캐스팅합니다.
        let raw_handle = RawWindowHandle::Web(
            WebWindowHandle::new(1)
        );
        unsafe { Ok(raw_window_handle::WindowHandle::borrow_raw(raw_handle)) }
    }
}

impl HasDisplayHandle for CanvasProxy {
    fn display_handle(
        &self,
    ) -> std::result::Result<raw_window_handle::DisplayHandle<'_>, HandleError> {
        let handle = WebDisplayHandle::new();
        let raw_handle = RawDisplayHandle::Web(handle);
        unsafe { Ok(raw_window_handle::DisplayHandle::borrow_raw(raw_handle)) }
    }
}
#[wasm_bindgen]
pub struct WgpuContext {
    _canvas: CanvasProxy,
    _instance: wgpu::Instance, // instance의 소유권을 저장할 필드 추가
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    _config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

#[wasm_bindgen]
impl WgpuContext {
    #[wasm_bindgen(constructor)]
    pub async fn new(canvas: CanvasProxy, image_url: String) -> Result<WgpuContext, JsValue> {
        console_error_panic_hook::set_once();
        canvas.0.set_attribute("data-raw-window-handle", "1")
            .map_err(|e| JsValue::from_str("Failed to set canvas attribute"))?;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
         // unsafe 블록을 사용하여 surface 생성
        let surface = instance.create_surface(canvas.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.map_err(|e| JsValue::from_str(&e.to_string()))?;

        let image_bytes = fetch_binary(&image_url).await?;
        let diffuse_image = image::load_from_memory(&image_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: dimensions.0,
            height: dimensions.1,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            // ... (rest of the pipeline setup is the same)
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default()
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            _canvas: canvas.clone(),
            _instance: instance, // instance를 구조체에 포함하여 소유권 이전

            device,
            queue,
            surface,
            _config: config,
            render_pipeline,
            bind_group,
        })
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let output = self.surface.get_current_texture().map_err(|e| JsValue::from_str(&e.to_string()))?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}