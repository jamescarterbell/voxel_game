use specs::prelude::*;
use std::sync::{Arc, Mutex};
use num_derive::*;
use num_traits::*;
use enum_display_derive::*;
use std::fmt::Display;
use std::collections::HashMap;
use glutin::event::ElementState;
use glutin::event::DeviceEvent;
pub use glutin::{dpi::PhysicalPosition};
use std::collections::hash_map::Entry;
use nalgebra as na;
use na::{Vector2};
use v_windowing::{ApplicationEvent, WindowDisplay};

pub struct Inputs{
    keys: HashMap<KeyCode, KeyState>,
    mouse_buttons: HashMap<ButtonCode, KeyState>,
    mouse_position: Vector2<f32>,
    mouse_delta: Vector2<f32>,
}

impl Default for Inputs{
    fn default() -> Self{
        let mut map = HashMap::new();
        for key in (0..54){
            map.insert(KeyCode::from_key(key), KeyState::Up);
        }
        let mut mouse_map = HashMap::new();
        mouse_map.insert(ButtonCode::MB0, KeyState::Up);
        mouse_map.insert(ButtonCode::MB1, KeyState::Up);
        mouse_map.insert(ButtonCode::MB2, KeyState::Up);
        Inputs{
            keys: map,
            mouse_buttons: mouse_map,
            mouse_position: Vector2::<f32>::new(0.0, 0.0),
            mouse_delta: Vector2::<f32>::new(0.0, 0.0),
        }
    }
}

impl Inputs{
    pub fn get_key(&self, key_code: &KeyCode) -> &KeyState{
        self.keys.get(key_code).unwrap()
    }

    pub fn get_button(&self, mouse_botton: &ButtonCode) -> &KeyState{
        match self.mouse_buttons.get(mouse_botton){
            None => &KeyState::Unpressed,
            Some(item) => item,
        }
    }

    pub fn get_mouse_position(&self) -> &Vector2<f32>{
        &self.mouse_position
    }

    pub fn get_mouse_delta(&self) -> &Vector2<f32>{
        &self.mouse_delta
    }
}

pub struct InputSystem{
    window_input_queue: Arc<Mutex<Vec<ApplicationEvent>>>,
    hardware_input_queue: Arc<Mutex<Vec<DeviceEvent>>>,
}

impl InputSystem{
    pub fn new(window_input_queue: Arc<Mutex<Vec<ApplicationEvent>>>, hardware_input_queue: Arc<Mutex<Vec<DeviceEvent>>>) -> Self{
        InputSystem{window_input_queue, hardware_input_queue}
    }
}

impl<'a> System<'a> for InputSystem{
    type SystemData = (
        Write<'a, Inputs>,
        Write<'a, CursorState>);

    fn run(&mut self, (mut inputs, mut cursor): Self::SystemData){

        //Update keyboard inputs
        for (key_code, key_state) in inputs.keys.iter_mut(){
            match key_state{
                KeyState::Pressed => *key_state = KeyState::Down,
                KeyState::Unpressed => *key_state = KeyState::Up,
                _ => {},
            }
        }

        //Update mouse inputs
        for (key_code, key_state) in inputs.mouse_buttons.iter_mut(){
            match key_state{
                KeyState::Pressed => *key_state = KeyState::Down,
                KeyState::Unpressed => *key_state = KeyState::Up,
                _ => {},
            }
        }

        //Get new inputs
        for input in self.window_input_queue.lock().unwrap().drain(..){
            match input{
                ApplicationEvent::CursorMoved {device_id, position, modifiers} =>{
                    inputs.mouse_position = Vector2::new(position.x as f32, position.y as f32);
                }
                ApplicationEvent::Focused(focused) => {
                    cursor.locked = focused;
                }
                _ => {},
            }
        }

        //Needs to be set to 0 because not every frame will have a delta
        inputs.mouse_delta = Vector2::new(0.0, 0.0);

        //Get hardware inputs
        for input in self.hardware_input_queue.lock().unwrap().drain(..){
            if cursor.locked{
                match input{
                    DeviceEvent::Key(key) =>{
                        match inputs.keys.entry(KeyCode::from_key(key.scancode)){
                            Entry::Occupied(mut e) => {
                                e.insert(if key.state == ElementState::Pressed {KeyState::Pressed}  else {KeyState::Unpressed});
                            },
                            _ => {println!("{}", key.scancode)},
                        } ;
                    },
                    DeviceEvent::Button {button, state} =>{
                        match inputs.mouse_buttons.entry(ButtonCode::from_key(button)){
                            Entry::Occupied(mut e) => {
                                e.insert(if state == ElementState::Pressed {KeyState::Pressed}  else {KeyState::Unpressed});
                            },
                            Entry::Vacant(mut e) => {
                                e.insert(if state == ElementState::Pressed {KeyState::Pressed}  else {KeyState::Unpressed});
                            }
                        }
                    },
                        DeviceEvent::MouseMotion {delta} => {
                        inputs.mouse_delta = Vector2::new(delta.0 as f32, delta.1 as f32);
                    },
                    _ => {},
                }
            }
        }
    }
}

#[derive(Display, PartialEq, Eq)]
pub enum KeyState{
    Pressed,
    Down,
    Up,
    Unpressed
}

#[derive(Display, FromPrimitive, PartialEq, Eq, Hash)]
pub enum KeyCode{
    Undefined = 0,
    ESC,
    KB1,
    KB2,
    KB3,
    KB4,
    KB5,
    KB6,
    KB7,
    KB8,
    KB9,
    KB0,
    MinusUnderscore,
    EqualPlus,
    Backspace,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    OpenBracket,
    CloseBracket,
    Enter,
    LeftControl,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Colon,
    Quote,
    Dunno,
    LeftShift,
    BackSlash,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    Slash,
    RightShift,
}

impl KeyCode{
    pub fn from_key(key: u32) -> KeyCode{
        match KeyCode::from_u32(key){
            Some(code) => code,
            None => KeyCode::Undefined
        }
    }
}

pub struct CursorState{
    pub visible: bool,
    pub locked: bool,
}

impl Default for CursorState{
    fn default() -> Self{
        CursorState{
            visible: false,
            locked: true,
        }
    }
}

pub struct CursorLockSystem{}

impl<'a> System<'a> for CursorLockSystem{
    type SystemData = (
        Read<'a, CursorState>,
        Write<'a, WindowDisplay>);

    fn run(&mut self, (cursor_state, window_display): Self::SystemData){
        let window_display = window_display.as_ref().unwrap().lock().unwrap();
        let window_size = window_display.gl_window().window().inner_size();
        if cursor_state.locked{
            window_display.gl_window().window().set_cursor_grab(true);
        }
        window_display.gl_window().window().set_cursor_visible(cursor_state.visible);
    }
}

#[derive(Display, FromPrimitive, PartialEq, Eq, Hash)]
pub enum ButtonCode{
    Undefined = 0,
    MB0 = 1,
    MB1,
    MB2,
}

impl ButtonCode{
    pub fn from_key(key: u32) -> ButtonCode{
        match ButtonCode::from_u32(key){
            Some(code) => code,
            None => ButtonCode::Undefined
        }
    }
}