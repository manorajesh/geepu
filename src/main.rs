use geepu::*;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ ActiveEventLoop, ControlFlow, EventLoop },
    window::{ Window, WindowId },
};
use std::sync::Arc;

// Define a simple vertex structure
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

struct App {
    window: Option<Arc<Window>>,
    context: Option<GpuContext>,
    vertex_buffer: Option<TypedBuffer<Vertex>>,
    pipeline: Option<RenderPipeline>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            context: None,
            vertex_buffer: None,
            pipeline: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Geepu Example")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            // Create GPU context
            let context = pollster::block_on(GpuContext::new_with_window(window.clone())).unwrap();

            // Create triangle vertices
            let vertices = [
                Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
                Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
                Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
            ];

            // Create vertex buffer
            let vertex_buffer = TypedBuffer::vertex(&context, &vertices).unwrap();

            // Create vertex layout
            let vertex_layout =
                vertex_layout![
                0 => VertexFormat::Float32x3, // position
                1 => VertexFormat::Float32x3, // color
            ];

            // Simple vertex shader
            let vertex_shader =
                r#"
                struct VertexInput {
                    @location(0) position: vec3<f32>,
                    @location(1) color: vec3<f32>,
                }

                struct VertexOutput {
                    @builtin(position) clip_position: vec4<f32>,
                    @location(0) color: vec3<f32>,
                }

                @vertex
                fn vs_main(model: VertexInput) -> VertexOutput {
                    var out: VertexOutput;
                    out.color = model.color;
                    out.clip_position = vec4<f32>(model.position, 1.0);
                    return out;
                }
            "#;

            // Simple fragment shader
            let fragment_shader =
                r#"
                @fragment
                fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                    return vec4<f32>(in.color, 1.0);
                }

                struct VertexOutput {
                    @builtin(position) clip_position: vec4<f32>,
                    @location(0) color: vec3<f32>,
                }
            "#;

            // Create render pipeline
            let surface_format = context.surface_format().unwrap();
            let pipeline = RenderPipeline::simple(
                &context,
                vertex_shader,
                fragment_shader,
                &[vertex_layout],
                surface_format,
                Some("Triangle Pipeline")
            ).unwrap();

            self.window = Some(window);
            self.context = Some(context);
            self.vertex_buffer = Some(vertex_buffer);
            self.pipeline = Some(pipeline);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(context) = &mut self.context {
                    context.resize(physical_size).unwrap();
                }
            }
            WindowEvent::RedrawRequested => {
                if
                    let (Some(context), Some(vertex_buffer), Some(pipeline)) = (
                        &self.context,
                        &self.vertex_buffer,
                        &self.pipeline,
                    )
                {
                    // Get surface texture
                    let output = match context.get_current_texture() {
                        Ok(output) => output,
                        Err(_) => {
                            return;
                        }
                    };

                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    // Create render commands
                    let mut commands = RenderCommands::new(context, Some("Frame Commands"));

                    // Begin render pass
                    let color_attachments = [Some(color_attachment(&view, Some(Color::BLACK)))];
                    let mut render_pass = commands.begin_render_pass(
                        &color_attachments,
                        None,
                        Some("Main Pass")
                    );

                    // Set pipeline and draw triangle
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_vertex_buffer(0, vertex_buffer);
                    render_pass.draw(0..3, 0..1);

                    // Finish render pass (drop it)
                    drop(render_pass);

                    // Submit commands
                    commands.submit(context);

                    // Present the frame
                    output.present();
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = pollster::block_on(GpuContext::new());
        assert!(context.is_ok());
    }

    #[test]
    fn test_buffer_creation() {
        let context = pollster::block_on(GpuContext::new()).unwrap();
        let data = [1.0f32, 2.0, 3.0, 4.0];
        let buffer = TypedBuffer::vertex(&context, &data);
        assert!(buffer.is_ok());
        let buffer = buffer.unwrap();
        assert_eq!(buffer.len(), 4);
    }
}
