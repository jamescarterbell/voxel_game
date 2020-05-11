use specs::prelude::*;
use std::sync::{Arc, Mutex};
use num_derive::*;
use num_traits::*;
use enum_display_derive::*;
use std::fmt::Display;
use std::collections::HashMap;
use glutin::event::ElementState;
pub use glutin::event::MouseButton;
use std::collections::hash_map::Entry;
use nalgebra as na;
use na::{Vector2};
use v_windowing::ApplicationEvent;

pub struct Inputs{
    keys: HashMap<KeyCode, KeyState>,
    mouse_buttons: HashMap<MouseButton, KeyState>,
    mouse_position: Vector2<f32>,
    mouse_delta: Vector2<f32>,
    first_frame: bool,
}

impl Default for Inputs{
    fn default() -> Self{
        let mut map = HashMap::new();
        for key in (0..54){
            map.insert(KeyCode::from_key(key), KeyState::Up);
        }
        let mut mouse_map = HashMap::new();
        mouse_map.insert(MouseButton::Left, KeyState::Up);
        mouse_map.insert(MouseButton::Right, KeyState::Up);
        mouse_map.insert(MouseButton::Middle, KeyState::Up);
        Inputs{
            keys: map,
            mouse_buttons: mouse_map,
            mouse_position: Vector2::<f32>::new(0.0, 0.0),
            mouse_delta: Vector2::<f32>::new(0.0, 0.0),
            first_frame: true
        }
    }
}

impl Inputs{
    pub fn get_key(&self, key_code: &KeyCode) -> &KeyState{
        self.keys.get(key_code).unwrap()
    }

    pub fn get_mouse_button(&self, mouse_botton: &MouseButton) -> &KeyState{
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
    input_queue: Arc<Mutex<Vec<ApplicationEvent>>>,
}

impl InputSystem{
    pub fn new(input_queue: Arc<Mutex<Vec<ApplicationEvent>>>) -> Self{
        InputSystem{input_queue}
    }
}

impl<'a> System<'a> for InputSystem{
    type SystemData = (Write<'a, Inputs>);

    fn run(&mut self, mut inputs: Self::SystemData){

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
        for input in self.input_queue.lock().unwrap().drain(..){
            match input{
                ApplicationEvent::KeyboardInput {device_id, input, is_synthetic} => {
                    match inputs.keys.entry(KeyCode::from_key(input.scancode)){
                        Entry::Occupied(mut e) => {
                            e.insert(if input.state == ElementState::Pressed {KeyState::Pressed}  else {KeyState::Unpressed});
                        },
                        _ => {println!("{}", input.scancode)},
                    } ;
                },
                ApplicationEvent::MouseInput {device_id, state, button, modifiers} =>{
                    match inputs.mouse_buttons.entry(button){
                        Entry::Occupied(mut e) => {
                            e.insert(if state == ElementState::Pressed {KeyState::Pressed}  else {KeyState::Unpressed});
                        },
                        Entry::Vacant(mut e) => {
                            e.insert(if state == ElementState::Pressed {KeyState::Pressed}  else {KeyState::Unpressed});
                        }
                    }
                },
                ApplicationEvent::CursorMoved {device_id, position, modifiers} =>{
                    let position_hold = inputs.mouse_position.clone();
                    inputs.mouse_position = Vector2::new(position.x as f32, position.y as f32);
                    if inputs.first_frame{
                        inputs.mouse_delta = Vector2::new(0.0, 0.0);
                        inputs.first_frame = false;
                    } else {
                        inputs.mouse_delta = &inputs.mouse_position - position_hold;
                    }
                }
                _ => {},
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
