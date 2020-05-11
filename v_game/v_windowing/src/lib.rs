use glium::*;
use glutin::event_loop::EventLoop;
use glutin::event::*;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use glutin::window::Theme;
use glium::debug::Source::Application;
use glutin::dpi::{PhysicalSize, PhysicalPosition};

pub struct GliumState{
    pub event_loop: EventLoop<()>,
    pub display: glium::Display,
    pub inputs: Arc<Mutex<Vec<ApplicationEvent>>>,
}

/// Contains all state used by renderer, also used for creating pipelines
impl GliumState{

    pub fn new() -> Self{
        let event_loop = glutin::event_loop::EventLoop::new();
        let wb = glutin::window::WindowBuilder::new();
        let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();
        let inputs = Arc::new(Mutex::new(Vec::new()));

        Self{
            event_loop,
            display,
            inputs
        }
    }

    pub fn input_queue(&self) -> Arc<Mutex<Vec<ApplicationEvent>>>{
        self.inputs.clone()
    }

    /// This will start the update loop of your game/program
    /// pass in the loop function for your game
    pub fn run_event_loop<F>(self, mut game: F)
        where F: GameState + 'static
    {
        //Need to destructure glium_state here in order to run loop
        //TODO: fix destructuring maybe?

        let mut display = self.display;
        let inputs = self.inputs;
        self.event_loop.run(move |e, _, flow|{
            // Poll window events for inputs
            match e{
                glutin::event::Event::WindowEvent{event, ..} => {
                    match event {
                        glutin::event::WindowEvent::CloseRequested => {
                            *flow = glutin::event_loop::ControlFlow::Exit;
                            return;
                        },
                        _ => *flow = glutin::event_loop::ControlFlow::Poll,
                    }
                    inputs.lock().unwrap().push(ApplicationEvent::from(event));
                },
                Event::MainEventsCleared => {
                    // Do logic
                    *flow = glutin::event_loop::ControlFlow::Poll;

                    game.game_loop();

                    game.render_loop(&mut display);

                    return;
                },
                _ => *flow = glutin::event_loop::ControlFlow::Poll,
            }
            return;
        });
    }
}

pub enum ApplicationEvent {
    Resized(PhysicalSize<u32>),
    Moved(PhysicalPosition<i32>),
    CloseRequested,
    Destroyed,
    DroppedFile(PathBuf),
    HoveredFile(PathBuf),
    HoveredFileCancelled,
    ReceivedCharacter(char),
    Focused(bool),
    KeyboardInput {
        device_id: DeviceId,
        input: KeyboardInput,
        is_synthetic: bool,
    },
    ModifiersChanged(ModifiersState),
    CursorMoved {
        device_id: DeviceId,
        position: PhysicalPosition<f64>,
        modifiers: ModifiersState,
    },
    CursorEntered {
        device_id: DeviceId,
    },
    CursorLeft {
        device_id: DeviceId,
    },
    MouseWheel {
        device_id: DeviceId,
        delta: MouseScrollDelta,
        phase: TouchPhase,
        modifiers: ModifiersState,
    },
    MouseInput {
        device_id: DeviceId,
        state: ElementState,
        button: MouseButton,
        modifiers: ModifiersState,
    },
    TouchpadPressure {
        device_id: DeviceId,
        pressure: f32,
        stage: i64,
    },
    AxisMotion {
        device_id: DeviceId,
        axis: u32,
        value: f64,
    },
    ScaleFactorChanged {
        scale_factor: f64,
        new_inner_size: PhysicalSize<u32>,
    },
    Touch(Touch),
    ThemeChanged(Theme),
}

impl From<WindowEvent<'_>> for ApplicationEvent{
    fn from(event: WindowEvent) -> Self{
        match event{
            WindowEvent::Resized(ps) => ApplicationEvent::Resized(ps),
            WindowEvent::Moved(pp) => ApplicationEvent::Moved(pp),
            WindowEvent::CloseRequested => ApplicationEvent::CloseRequested,
            WindowEvent::Destroyed => ApplicationEvent::Destroyed,
            WindowEvent::DroppedFile(path) => ApplicationEvent::DroppedFile(path),
            WindowEvent::HoveredFile(path) => ApplicationEvent::HoveredFile(path),
            WindowEvent::HoveredFileCancelled => ApplicationEvent::HoveredFileCancelled,
            WindowEvent::ReceivedCharacter(char) => ApplicationEvent::ReceivedCharacter(char),
            WindowEvent::Focused(focused) => ApplicationEvent::Focused(focused),
            WindowEvent::KeyboardInput { device_id, input, is_synthetic } => ApplicationEvent::KeyboardInput {device_id, input, is_synthetic},
            WindowEvent::ModifiersChanged(ms) => ApplicationEvent::ModifiersChanged(ms),
            WindowEvent::CursorMoved { device_id, position, modifiers } => ApplicationEvent::CursorMoved {device_id, position, modifiers},
            WindowEvent::CursorEntered { device_id } => ApplicationEvent::CursorEntered {device_id},
            WindowEvent::CursorLeft { device_id } => ApplicationEvent::CursorLeft {device_id},
            WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => ApplicationEvent::MouseWheel {device_id, delta, phase, modifiers},
            WindowEvent::MouseInput { device_id, state, button, modifiers } => ApplicationEvent::MouseInput {device_id, state, button, modifiers},
            WindowEvent::TouchpadPressure { device_id, pressure, stage } => ApplicationEvent::TouchpadPressure {device_id, pressure, stage},
            WindowEvent::AxisMotion { device_id, axis, value } => ApplicationEvent::AxisMotion {device_id, axis, value},
            WindowEvent::Touch(t) => ApplicationEvent::Touch(t),
            WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => ApplicationEvent::ScaleFactorChanged {scale_factor, new_inner_size: *new_inner_size},
            WindowEvent::ThemeChanged(t) => ApplicationEvent::ThemeChanged(t),
        }
    }
}

pub trait GameState{
    fn game_loop(&mut self);
    fn render_loop(&mut self, display: &mut Display) -> Result<(), GliumError>;
}

pub enum GliumError{
    DrawError(DrawError),
    SwapBuffersError(SwapBuffersError),
}