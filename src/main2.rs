use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod renderer;
mod model;
mod texture;

use renderer::Renderer;
use model::ModelInstance;

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    zoom: f32,
    znear: f32,
    zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::ortho(-self.aspect / self.zoom, self.aspect / self.zoom, -1. / self.zoom, 1. / self.zoom, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: f32,
    color: [f32; 3],
}

impl Instance {
    fn to_raw(&self) -> ModelInstance {
        let scale = cgmath::Matrix4::from_nonuniform_scale(0.5, 0.5 * 1.618, 0.5);
        ModelInstance {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from_angle_y(cgmath::Deg(self.rotation)) * scale).into(),
            normal: cgmath::Matrix3::from_angle_y(cgmath::Deg(self.rotation)).into(),
            color: self.color,
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = futures::executor::block_on(Renderer::new(&window));

    let mut camera =  Camera {
        // position the camera one unit up and 2 units back
        // +z is out of the screen
        eye: (2.0f32.sqrt(), 1.0, 2.0f32.sqrt()).into(),
        // have it look at the origin
        target: (0.0, 0.0, 0.0).into(),
        // which way is "up"
        up: cgmath::Vector3::unit_y(),
        aspect: state.size.width as f32 / state.size.height as f32,
        zoom: 0.0625,
        znear: -100.,
        zfar: 100.,
    };

    let mut instances: Vec<_> = (0..20).map(|i| {
        let t = 2. * std::f32::consts::PI / 20. * i as f32;
        let position = (t.cos() * 10., 0., t.sin() * 10.).into();
        let rotation = i as f32 * 20.;
        let mut color = [0.6, 0.2, 0.1];
        use rand::Rng;
        color[0] += rand::thread_rng().gen_range(-0.1..0.1);
        color[1] += rand::thread_rng().gen_range(-0.1..0.1);
        color[2] += rand::thread_rng().gen_range(-0.1..0.1);
        Instance { position, rotation, color }
    }).collect();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                    camera.aspect = state.size.width as f32 / state.size.height as f32;
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(*new_inner_size);
                },
                _ => (),
            },¡
            Event::RedrawRequested(_) => {
                let start = std::time::SystemTime::now();
                let since_the_epoch = start
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards");
                let now = since_the_epoch.as_millis() as f64 / 1000.;
                for (i, instance) in instances.iter_mut().enumerate() {
                    instance.position.z += (now + i as f64).sin() as f32 * 0.01;
                    instance.rotation += 0.1 * (i % 3) as f32;
                }
                
                let uniforms = renderer::Uniforms {
                    view_proj: camera.build_view_projection_matrix().into(),
                };
                let instance_data: Vec<_> = instances.iter().map(|instance| instance.to_raw()).collect();

                let scene = renderer::Scene {
                    uniforms,
                    instances: instance_data,
                };

                match state.render(&scene) {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            },
            _ => (),
        }
    });
}
