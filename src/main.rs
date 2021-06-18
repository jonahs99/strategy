use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::time::{Duration, Instant};

mod model;
mod texture;
mod renderer;
use renderer::Renderer;

fn main() {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_title("simple strategy")
        .build(&event_loop)
        .expect("Failed to build a window :(");

    let dt = Duration::from_millis(16);
    let mut stepper = TimeStepper::new(Instant::now(), dt);

    event_loop.run(move |event, _target, control_flow| {
        if let Some(flow) = handle_event(&event) {
            *control_flow = flow;
            return;
        }

        *control_flow = ControlFlow::Poll;

        stepper.advance(Instant::now());

        while stepper.tick() {
            // TODO: Update
        }

        // TODO: Render
    });
}

fn handle_event(event: &Event<()>) -> Option<ControlFlow> {
    match event {
        Event::WindowEvent {
            event,
            window_id,
        } => match event {
            WindowEvent::CloseRequested => Some(ControlFlow::Exit),
            _ => None,
        },
        _ => None,
    }
}

struct TimeStepper {
    current: Instant,
    dt: Duration,
    residual: Duration,
}

impl TimeStepper {
    fn new(time: Instant, dt: Duration) -> Self {
        Self {
            current: time,
            dt,
            residual: Duration::from_millis(0),
        }
    }

    fn advance(&mut self, time: Instant) {
        self.residual += time - self.current;
        self.current = time;
    }

    fn tick(&mut self) -> bool {
        if self.residual > self.dt {
            self.residual -= self.dt;
            true
        } else {
            false
        }
    }
    
    fn blend(&self) -> f32 {
        self.residual.as_secs_f32() / self.dt.as_secs_f32()
    }
}
