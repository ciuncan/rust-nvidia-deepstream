use pollster::FutureExt;
use shared::observability::*;

fn main() -> anyhow::Result<()> {
    let _droppables = setup_tracing()?;

    info!("Creating instance...");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });
    info!(report = ?instance.generate_report(), "Instance created.");

    info!("Creating adapter...");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .block_on()
        .ok_or(anyhow::anyhow!("Couldn't create the adapter"))?;

    info!(info = ?adapter.get_info(), "Adapter created");

    info!("Requesting device and queue...");

    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .block_on()?;

    device.on_uncaptured_error(Box::new(|e| {
        error!("Uncaptured error from device: {:?}", e);
    }));

    info!(
        "Device found, features: {:?}, limits: {:?}",
        device.features(),
        device.limits()
    );

    info!("Loading image...");

    let input_image = image::load_from_memory(include_bytes!("../assets/sushi.png"))?.to_rgba8();
    let (width, height) = input_image.dimensions();

    info!("Creating texture...");

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let input_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("input texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    info!("Writing texture...");

    queue.write_texture(
        input_texture.as_image_copy(),
        bytemuck::cast_slice(input_image.as_raw()),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
        },
        texture_size,
    );

    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("output texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
    });

    info!("Creating shader...");

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Grayscale shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/grayscale.wgsl").into()),
    });

    info!("Creating pipeline...");

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Grayscale pipeline"),
        layout: None,
        module: &shader,
        entry_point: "grayscale_main",
    });

    info!("Creating bind group...");

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Texture bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(
                    &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
    });

    info!("Creating encoder...");

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // do compute pass then dispose it, drops exclusive borrow of encoder
    info!("Defining compute pass...");
    {
        let (dispatch_width, dispatch_height) =
            compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Grayscale pass"),
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &texture_bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_width, dispatch_height, 1);
    }

    let padded_bytes_per_row = padded_bytes_per_row(width);
    let unpadded_bytes_per_row = width as usize * 4;

    info!("Creating output buffer...");

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output buffer"),
        size: height as u64 * padded_bytes_per_row as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    info!("Copying texture to buffer...");

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture: &output_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row as u32),
                rows_per_image: Some(height),
            },
        },
        texture_size,
    );

    info!("Submitting encoder work to queue...");

    queue.submit(Some(encoder.finish()));

    info!("Attempting to map buffer...");

    let buffer_slice = output_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |r| match r {
        Err(error) => error!(?error, "Failed to read buffer"),
        Ok(()) => info!("Buffer read successfully"),
    });

    info!("Polling device to finish queue...");

    device.poll(wgpu::Maintain::Wait);

    info!("Reading mapped buffer into vector...");

    let padded_data = buffer_slice.get_mapped_range();
    let mut pixels: Vec<u8> = vec![0; unpadded_bytes_per_row * height as usize];
    for (padded, pixels) in padded_data
        .chunks_exact(padded_bytes_per_row)
        .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row))
    {
        pixels.copy_from_slice(&padded[..unpadded_bytes_per_row]);
    }

    info!("Creating image from vector...");

    if let Some(output_image) =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, &pixels[..])
    {
        info!("Saving image...");
        output_image.save("sushi-grayscale.png")?;
    }

    info!("Done!");

    Ok(())
}

fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let x = (width + workgroup_width - 1) / workgroup_width;
    let y = (height + workgroup_height - 1) / workgroup_height;

    (x, y)
}

/// Compute the next multiple of 256 for texture retrieval padding.
fn padded_bytes_per_row(width: u32) -> usize {
    let bytes_per_row = width as usize * 4;
    let padding = (256 - bytes_per_row % 256) % 256;
    bytes_per_row + padding
}
