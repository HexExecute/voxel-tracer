use app::State;
use winit::{window::Window, event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent}, dpi::PhysicalSize};

mod svo;
mod app;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::new(800, 500));

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => if !state.input(event) { // UPDATED!
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                _ => {}
            }
        }
        Event::RedrawRequested(window_id) if window_id == state.window.id() => {
            state.update();
            pollster::block_on(async {
                match state.render().await {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e)
                }
            });
        }
        Event::MainEventsCleared => {
            state.window.request_redraw();
        }
        _ => {}
    });
}
