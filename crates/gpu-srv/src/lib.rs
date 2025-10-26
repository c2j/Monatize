use anyhow::Result;

pub fn render_solid_rgba8(width: u32, height: u32, rgba: [f32; 4]) -> Result<Vec<u8>> {
    pollster::block_on(render_solid_rgba8_async(width, height, rgba))
}

async fn render_solid_rgba8_async(width: u32, height: u32, rgba: [f32; 4]) -> Result<Vec<u8>> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("request adapter");

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .expect("request device");

    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });
    {
        let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: rgba[0] as f64, g: rgba[1] as f64, b: rgba[2] as f64, a: rgba[3] as f64 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    // Copy texture to buffer and map for readback
    let bytes_per_pixel = 4u32;
    let padded_bytes_per_row = ((width * bytes_per_pixel + 255) / 256) * 256;
    let size = (padded_bytes_per_row * height) as usize;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: size as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded_bytes_per_row), rows_per_image: Some(height) } },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );

    queue.submit(std::iter::once(encoder.finish()));

    // Map and read back
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).ok();
    let data = slice.get_mapped_range();

    // Unpad rows
    let mut out = vec![0u8; (width * height * bytes_per_pixel) as usize];
    for y in 0..height as usize {
        let src_off = y * padded_bytes_per_row as usize;
        let dst_off = y * (width * bytes_per_pixel) as usize;
        out[dst_off..dst_off + (width * bytes_per_pixel) as usize]
            .copy_from_slice(&data[src_off..src_off + (width * bytes_per_pixel) as usize]);
    }
    drop(data);
    buf.unmap();

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_clear_works() {
        let w = 64; let h = 32;
        let img = render_solid_rgba8(w, h, [1.0, 0.0, 0.0, 1.0]).expect("render");
        assert_eq!(img.len(), (w*h*4) as usize);
        // top-left pixel should be red (sRGB approx 255,0,0,255)
        assert!(img[0] > 200 && img[1] < 30 && img[2] < 30 && img[3] > 200);
    }
}

