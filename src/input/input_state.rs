#![allow(unused)]
use std::sync::{Arc, Mutex};
use winit::event::{VirtualKeyCode, KeyboardInput};

use super::input_key::InputKey;

pub struct GlobalInputState {
    inputs: Vec<InputKey>
}
impl GlobalInputState {
    pub fn new() -> Arc<Mutex<GlobalInputState>> {
        Arc::new(Mutex::new(Self { inputs: vec![InputKey::new(); 162] }))
    }
    pub fn poll(&mut self, input: KeyboardInput) {
        if let Some(key) = input.virtual_keycode {
            self.inputs[key as usize].poll(input.state);
        }
    }
    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
            self.inputs[key as usize].is_pressed()
    }
    pub fn is_just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.inputs[key as usize].is_just_pressed()

    }
    pub fn is_just_unpressed(&self, key: VirtualKeyCode) -> bool {
        self.inputs[key as usize].is_just_unpressed()

    }
}