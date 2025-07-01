use geepu::*;

async fn texture_example() -> Result<()> {
    // Create GPU context
    let context = GpuContext::new().await?;

    println!("GPU Context created successfully!");

    // Create a simple 4x4 RGBA texture with a checkerboard pattern
    let width = 4u32;
    let height = 4u32;
    let mut texture_data = Vec::new();

    for y in 0..height {
        for x in 0..width {
            // Create checkerboard pattern
            let is_white = (x + y) % 2 == 0;
            if is_white {
                texture_data.extend_from_slice(&[255u8, 255, 255, 255]); // White
            } else {
                texture_data.extend_from_slice(&[0u8, 0, 0, 255]); // Black
            }
        }
    }

    println!("Created {}x{} checkerboard texture with {} bytes", width, height, texture_data.len());

    // Create texture from bytes
    let texture = Texture::from_bytes(
        &context,
        &texture_data,
        width,
        height,
        TextureFormat::Rgba8UnormSrgb,
        Some("Checkerboard Texture")
    )?;

    println!("Texture created successfully!");
    println!("Texture size: {:?}", texture.size());
    println!("Texture format: {:?}", texture.format());

    // Create a render target texture
    let render_target = Texture::create_render_target(
        &context,
        256,
        256,
        TextureFormat::Rgba8UnormSrgb,
        Some("Render Target")
    )?;

    println!("Render target created successfully!");
    println!("Render target size: {:?}", render_target.size());

    // Create depth texture
    let depth_texture = Texture::create_depth_texture(&context, 256, 256, Some("Depth Buffer"))?;

    println!("Depth texture created successfully!");
    println!("Depth texture size: {:?}", depth_texture.size());
    println!("Depth texture format: {:?}", depth_texture.format());

    // Test texture builder
    let custom_texture = TextureBuilder::new(512, 512)
        .format(TextureFormat::Rgba8UnormSrgb)
        .usage(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
        .label("Custom Built Texture")
        .build(&context)?;

    println!("Custom texture built successfully!");
    println!("Custom texture size: {:?}", custom_texture.size());

    // Test writing data to texture
    let solid_red_data: Vec<u8> = (0..width * height).flat_map(|_| [255u8, 0, 0, 255]).collect();
    texture.write_data(&context, &solid_red_data, width, height)?;

    println!("Successfully wrote solid red data to texture!");

    println!("✅ All texture operations completed successfully!");

    Ok(())
}

fn main() {
    env_logger::init();

    match pollster::block_on(texture_example()) {
        Ok(()) => println!("Texture example completed successfully!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
