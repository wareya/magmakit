use std::collections::HashMap;

use glium::glutin;

pub (crate) struct InputHandler {
    pub (crate) keys_down_previous: HashMap<String, bool>,
    pub (crate) keys_down: HashMap<String, bool>,
    pub (crate) mouse_pos: (f64, f64),
    pub (crate) mouse_delta: (f64, f64),
    pub (crate) mouse_buttons: [bool; 5],
}

impl InputHandler {
    pub (crate) fn new() -> InputHandler
    {
        InputHandler{keys_down : HashMap::new(), keys_down_previous : HashMap::new(), mouse_pos: (0.0, 0.0), mouse_delta: (0.0, 0.0), mouse_buttons: [false, false, false, false, false]}
    }
    pub (crate) fn keyevent(&mut self, event : glutin::KeyboardInput)
    {
        if let Some(key) = event.virtual_keycode
        {
            use glium::glutin::VirtualKeyCode::*;
            let keystr = match key
            {
                // esc
                Escape => "Esc",
                
                // number row
                Key0 => "0",
                Key1 => "1",
                Key2 => "2",
                Key3 => "3",
                Key4 => "4",
                Key5 => "5",
                Key6 => "6",
                Key7 => "7",
                Key8 => "8",
                Key9 => "9",
                
                // letters
                A => "A",
                B => "B",
                C => "C",
                D => "D",
                E => "E",
                F => "F",
                G => "G",
                H => "H",
                I => "I",
                J => "J",
                K => "K",
                L => "L",
                M => "M",
                N => "N",
                O => "O",
                P => "P",
                Q => "Q",
                R => "R",
                S => "S",
                T => "T",
                U => "U",
                V => "V",
                W => "W",
                X => "X",
                Y => "Y",
                Z => "Z",
                
                // navigation block (bottom)
                Left => "Left",
                Up => "Up",
                Right => "Right",
                Down => "Down",
                
                // typographic control
                Back => "Backspace",
                Return => "Enter",
                Space => "Space",
                Tab => "Tab",
                
                // punctuation
                Grave => "`",
                
                Minus => "-",
                Equals => "=",
                
                LBracket => "[",
                RBracket => "]",
                Backslash => "\\",
                
                Semicolon => ";",
                Apostrophe => "'",
                
                Comma => ",",
                Period => ".",
                Slash => "/",
                
                // navigation block (top)
                Scroll => "ScollLock",
                Pause => "Pause",
                Insert => "Insert",
                Delete => "Delete",
                Home => "Home",
                End => "End",
                PageUp => "PageUp",
                PageDown => "PageDown",
                
                // modifiers
                LAlt => "LAlt",
                LShift => "LShift",
                LControl => "LControl",
                RAlt => "RAlt",
                RShift => "RShift",
                RControl => "RControl",
                
                // numpad
                Numpad0 => "Numpad0",
                Numpad1 => "Numpad1",
                Numpad2 => "Numpad2",
                Numpad3 => "Numpad3",
                Numpad4 => "Numpad4",
                Numpad5 => "Numpad5",
                Numpad6 => "Numpad6",
                Numpad7 => "Numpad7",
                Numpad8 => "Numpad8",
                Numpad9 => "Numpad9",
                
                Divide => "Numpad/",
                Multiply => "Numpad*",
                Subtract => "Numpad-",
                Add => "Numpad+",
                Decimal => "Numpad.",
                
                // functions
                F1 => "F1",
                F2 => "F2",
                F3 => "F3",
                F4 => "F4",
                F5 => "F5",
                F6 => "F6",
                F7 => "F7",
                F8 => "F8",
                F9 => "F9",
                F10 => "F10",
                F11 => "F11",
                F12 => "F12",
                
                // unsupported
                _ => ""
            };
            if keystr != ""
            {
                self.keys_down.insert(keystr.to_string(), event.state == glutin::ElementState::Pressed);
            }
        }
    }
    pub (crate) fn cycle(&mut self)
    {
        self.keys_down_previous = self.keys_down.clone();
    }
}
