use std::env;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::thread;



use event_packet::{read_len_prefixed, Message};
use texture_verify::is_roughly_color_rgba8;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

const SHADER: &str = r#"
struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VSOut {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>( 1.0, -1.0), vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>( 1.0,  1.0), vec2<f32>(-1.0,  1.0)
    );
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 0.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(positions[idx], 0.0, 1.0);
    out.uv = uvs[idx];
    return out;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv);
}
"#;

fn usage() {
    println!("gpui-app-host --uds <path>");
}

enum Msg {
    FrameOk(bool),
    Frame { pixels: Vec<u8>, w: u32, h: u32, stride: u32, ok: bool },
    Quit,
}

struct FrameGpu {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    size: (u32, u32),
    stride: u32,
}

struct PendingFrame {
    pixels: Vec<u8>,
    w: u32,
    h: u32,
    stride: u32,
    ok: bool,
}

struct GpuState {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    format: wgpu::TextureFormat,
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    pipeline: wgpu::RenderPipeline,
    frame: Option<FrameGpu>,
    screenshot_tex: Option<wgpu::Texture>,
    screenshot_buf: Option<wgpu::Buffer>,
}

struct App {
    window: Option<Window>,
    gpu: Option<GpuState>,
    rx: mpsc::Receiver<Msg>,
    clear_ok: bool,
    pending: Option<PendingFrame>,
    screenshot_pending: bool,
    screenshot_done: bool,
    pending_quit: bool,
}

impl App {
    fn new(rx: mpsc::Receiver<Msg>) -> Self {
        Self { window: None, gpu: None, rx, clear_ok: true, pending: None, screenshot_pending: true, screenshot_done: false, pending_quit: false }
    }

    fn init_gpu(&mut self) {
        let window = match self.window.as_ref() {
            Some(w) => w,
            None => return,
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        // Create a temporary surface to query capabilities.
        let surface = instance.create_surface(window).expect("create surface");
        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
        })
        .expect("request adapter");
        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(&wgpu::DeviceDescriptor::default())
                .await
        })
        .expect("request device");
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        // Pipeline & resources
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        self.gpu = Some(GpuState { instance, device, queue, config, format, bgl, sampler, pipeline, frame: None, screenshot_tex: None, screenshot_buf: None });
    }

    fn reconfigure(&mut self) {
        if let (Some(gpu), Some(w)) = (self.gpu.as_mut(), self.window.as_ref()) {
            let size = w.inner_size();
            if size.width == 0 || size.height == 0 {
                return;
            }
            gpu.config.width = size.width.max(1);
            gpu.config.height = size.height.max(1);
            // Surface is created on demand in render(), so configure there.
        }
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let Some(gpu) = self.gpu.as_mut() else { return; };
        let Some(window) = self.window.as_ref() else { return; };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 { return; }

        // Upload pending frame (if any)
        if let Some(pf) = self.pending.take() {
            // (Re)create texture if size changed or not present
            let recreate = match gpu.frame.as_ref() {
                Some(f) => f.size != (pf.w, pf.h),
                None => true,
            };
            if recreate {
                let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("frame_tex"),
                    size: wgpu::Extent3d { width: pf.w, height: pf.h, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("frame_bg"),
                    layout: &gpu.bgl,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&gpu.sampler) },
                    ],
                });
                gpu.frame = Some(FrameGpu { texture, view, bind_group, size: (pf.w, pf.h), stride: pf.stride });
            }
            if let Some(frame_gpu) = gpu.frame.as_ref() {
                gpu.queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture: &frame_gpu.texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                    &pf.pixels,
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(pf.stride), rows_per_image: None },
                    wgpu::Extent3d { width: pf.w, height: pf.h, depth_or_array_layers: 1 },
                );
            }
        }

        // Create surface on demand, (re)configure, then draw.
        let surface = gpu.instance.create_surface(window).expect("create surface");
        surface.configure(&gpu.device, &gpu.config);

        let clear_color = if self.clear_ok {
            wgpu::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
        } else {
            wgpu::Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }
        };
        let mut need_reconfig = false;
        match surface.get_current_texture() {
            Ok(frame) => {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });
                // Optionally render to screenshot texture first
                if self.screenshot_pending {
                    // Ensure screenshot resources exist and match window size
                    let w = gpu.config.width.max(1);
                    let h = gpu.config.height.max(1);
                    let need_new = match gpu.screenshot_tex.as_ref() {
                        Some(tex) => {
                            let ds = tex.size();
                            ds.width != w || ds.height != h
                        }
                        None => true,
                    };
                    if need_new {
                        gpu.screenshot_tex = Some(gpu.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("screenshot_tex"),
                            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: gpu.config.format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                            view_formats: &[],
                        }));
                        let padded_bpr = ((w * 4 + 255) / 256) * 256;
                        let size_bytes = (padded_bpr * h) as u64;
                        gpu.screenshot_buf = Some(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("screenshot_buf"),
                            size: size_bytes,
                            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                            mapped_at_creation: false,
                        }));
                    }
                    if let (Some(sst), Some(_buf)) = (gpu.screenshot_tex.as_ref(), gpu.screenshot_buf.as_ref()) {
                        let sview = sst.create_view(&wgpu::TextureViewDescriptor::default());
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("render-pass-screenshot"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &sview,
                                resolve_target: None,
                                depth_slice: None,
                                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear_color), store: wgpu::StoreOp::Store },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });
                        if let Some(frame_gpu) = gpu.frame.as_ref() {
                            rpass.set_pipeline(&gpu.pipeline);
                            rpass.set_bind_group(0, &frame_gpu.bind_group, &[]);
                            rpass.draw(0..6, 0..1);
                        }
                        drop(rpass);
                        // Copy to buffer
                        let padded_bpr = ((gpu.config.width * 4 + 255) / 256) * 256;
                        encoder.copy_texture_to_buffer(
                            wgpu::TexelCopyTextureInfo { texture: sst, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                            wgpu::TexelCopyBufferInfo { buffer: gpu.screenshot_buf.as_ref().unwrap(), layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded_bpr), rows_per_image: Some(gpu.config.height) } },
                            wgpu::Extent3d { width: gpu.config.width, height: gpu.config.height, depth_or_array_layers: 1 },
                        );
                    }
                }

                // Render to swapchain
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("render-pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear_color), store: wgpu::StoreOp::Store },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                    if let Some(frame_gpu) = gpu.frame.as_ref() {
                        rpass.set_pipeline(&gpu.pipeline);
                        rpass.set_bind_group(0, &frame_gpu.bind_group, &[]);
                        rpass.draw(0..6, 0..1);
                    }
                }

                gpu.queue.submit(std::iter::once(encoder.finish()));

                // If screenshot requested, map and verify now (blocking once)
                if self.screenshot_pending {
                    if let Some(buf) = gpu.screenshot_buf.as_ref() {
                        let slice = buf.slice(..);
                        slice.map_async(wgpu::MapMode::Read, |_| {});
                        gpu.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).ok();
                        {
                            let data = slice.get_mapped_range();
                            let w = gpu.config.width as usize;
                            let h = gpu.config.height as usize;
                            let padded_bpr = ((w as u32 * 4 + 255) / 256 * 256) as usize;
                            let mut compact = Vec::with_capacity(w * h * 4);
                            for row in 0..h {
                                let start = row * padded_bpr;
                                compact.extend_from_slice(&data[start..start + w * 4]);
                            }
                            let ok = if matches!(gpu.config.format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb) {
                                let mut converted = compact.clone();
                                for px in converted.chunks_mut(4) { px.swap(0, 2); }
                                is_roughly_color_rgba8(&converted, (255, 0, 0), 16)
                            } else {
                                is_roughly_color_rgba8(&compact, (255, 0, 0), 16)
                            };
                            println!("SCREENSHOT {}", if ok { "OK" } else { "FAIL" });
                            self.screenshot_done = true;
                            self.screenshot_pending = false;
                        }
                        buf.unmap();
                    }
                }


                // Handle deferred quit after screenshot
                if self.pending_quit && (!self.screenshot_pending || self.screenshot_done) {
                    println!("QUIT");
                    event_loop.exit();
                }

                frame.present();
            }
            Err(err) => match err {
                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => { need_reconfig = true; }
                wgpu::SurfaceError::OutOfMemory => event_loop.exit(),
                wgpu::SurfaceError::Timeout => {/* just skip a frame */}
                _ => {}
            },
        }
        drop(surface);
        if need_reconfig { self.reconfigure(); }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default().with_title("Monazite Phase-0");
        self.window = Some(event_loop.create_window(attrs).expect("create window"));
        // Initialize GPU + swapchain config (surface created on demand during render).
        self.init_gpu();
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                self.reconfigure();
                if let Some(w) = self.window.as_ref() { w.request_redraw(); }
            }
            WindowEvent::RedrawRequested => {
                // Draw a solid color based on latest frame status.
                self.render(event_loop);
                if let Some(w) = self.window.as_ref() {
                    let title = if self.clear_ok { "FRAME OK" } else { "FRAME FAIL" };
                    w.set_title(&format!("Monazite Phase-0 — {}", title));
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Drain incoming messages from UDS thread.
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::FrameOk(ok) => self.clear_ok = ok,
                Msg::Frame { pixels, w, h, stride, ok } => {
                    self.clear_ok = ok;
                    self.pending = Some(PendingFrame { pixels, w, h, stride, ok });
                }
                Msg::Quit => {
                    // Defer quit until after screenshot is taken/presented
                    self.pending_quit = true;
                }
            }
        }
        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let uds_path = match args.iter().position(|a| a == "--uds").and_then(|i| args.get(i + 1)).cloned() {
        Some(p) => p,
        None => {
            usage();
            return;
        }
    };

    let (tx, rx) = mpsc::channel();

    // Spawn background thread for UDS accept + read loop, keep stdout protocol stable.
    thread::spawn(move || {
        let _ = std::fs::remove_file(&uds_path);
        let listener = UnixListener::bind(&uds_path).expect("bind uds");
        eprintln!("gpui-app-host: listening on {}", uds_path);
        match listener.accept() {
            Ok((mut stream, _addr)) => loop {
                match read_len_prefixed(&mut stream) {
                    Ok(Message::Frame { pixels, size, stride }) => {
                        let ok = is_roughly_color_rgba8(&pixels, (255, 0, 0), 16);
                        println!("FRAME {}x{} {}", size.0, size.1, if ok { "OK" } else { "FAIL" });
                        let _ = tx.send(Msg::Frame { pixels, w: size.0, h: size.1, stride, ok });
                        if !ok {
                            // Keep process alive but mark FAIL in title.
                        }
                    }
                    Ok(Message::Event(_)) => {
                        println!("EVENT");
                    }
                    Ok(Message::Quit) => {
                        let _ = tx.send(Msg::Quit);
                        break;
                    }
                    Err(e) => {
                        eprintln!("read error: {e}");
                        break;
                    }
                }
            },
            Err(e) => eprintln!("accept error: {e}"),
        }
    });

    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::new(rx);
    let _ = event_loop.run_app(&mut app);
}


#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::path::PathBuf;

    fn has(cmd: &str) -> bool {
        Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {} >/dev/null 2>&1", cmd))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    // Minimal integration smoke test that runs the full pipeline under Xvfb.
    // Skips when xvfb-run or python3 are missing.
    #[test]
    fn xvfb_e2e_smoke0() {
        if !has("xvfb-run") || !has("python3") {
            eprintln!("skip: xvfb-run or python3 not found");
            return;
        }
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
        let status = Command::new("xvfb-run")
            .current_dir(&workspace)
            .args(["-s", "-screen 0 800x600x24", "python3", "scripts/smoke-0.py"])
            .status()
            .expect("failed to run xvfb-run python3 scripts/smoke-0.py");
        assert!(status.success());
    }
}
