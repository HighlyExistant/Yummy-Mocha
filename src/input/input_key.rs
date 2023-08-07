#![allow(unused)]
use winit::event::VirtualKeyCode;

/// Temporary File for testing user input.

#[derive(Clone, Copy)]
pub struct InputKey {
    /// bitmask
    /// 0000 0 Reserved
    /// 1 justpressed
    /// 1 justunpressed
    /// 1 currentlypressed
    state: u8,
}

impl InputKey {
    pub fn new() -> Self {
        Self { state: 0 }
    }
    pub fn key_pressed(&mut self) {
        if self.is_pressed() {
            self.state &= 0b11111101; // just pressed clear
        } else {
            self.state |= 0b00000010; // just pressed set
        }
        self.state |= 0b00000001;
    }
    pub fn key_unpressed(&mut self) {
        if !self.is_pressed() {
            self.state &= 0b11111011; // just pressed clear
        } else {
            self.state |= 0b00000100; // just pressed set
        }
        self.state &= 0b11111110;
    }
    pub fn poll(&mut self, state: winit::event::ElementState) {
        match state {
            winit::event::ElementState::Pressed => {
                self.key_pressed();
            }
            winit::event::ElementState::Released => {
                self.key_unpressed();
            }
        }
    }
    pub fn is_pressed(&self) -> bool {
        (self.state & 0b00000001) == 0b00000001
    }
    pub fn is_just_pressed(&self) -> bool {
        (self.state & 0b00000010) == 0b00000010
    }
    pub fn is_just_unpressed(&self) -> bool {
        (self.state & 0b00000100) == 0b00000100
    }
}