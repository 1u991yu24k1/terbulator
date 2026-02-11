use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

pub struct KeyboardHandler {
    modifiers: ModifiersState,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            modifiers: ModifiersState::empty(),
        }
    }

    pub fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    pub fn handle_key(&self, key: &PhysicalKey) -> Option<Vec<u8>> {
        match key {
            PhysicalKey::Code(code) => self.handle_keycode(*code),
            _ => None,
        }
    }

    fn handle_keycode(&self, code: KeyCode) -> Option<Vec<u8>> {
        let ctrl = self.modifiers.control_key();
        let shift = self.modifiers.shift_key();

        match code {
            // Control characters
            KeyCode::KeyA if ctrl => Some(vec![0x01]),
            KeyCode::KeyB if ctrl => Some(vec![0x02]),
            KeyCode::KeyC if ctrl => Some(vec![0x03]),
            KeyCode::KeyD if ctrl => Some(vec![0x04]),
            KeyCode::KeyE if ctrl => Some(vec![0x05]),
            KeyCode::KeyF if ctrl => Some(vec![0x06]),
            KeyCode::KeyG if ctrl => Some(vec![0x07]),
            KeyCode::KeyH if ctrl => Some(vec![0x08]),
            KeyCode::KeyI if ctrl => Some(vec![0x09]),
            KeyCode::KeyJ if ctrl => Some(vec![0x0A]),
            KeyCode::KeyK if ctrl => Some(vec![0x0B]),
            KeyCode::KeyL if ctrl => Some(vec![0x0C]),
            KeyCode::KeyM if ctrl => Some(vec![0x0D]),
            KeyCode::KeyN if ctrl => Some(vec![0x0E]),
            KeyCode::KeyO if ctrl => Some(vec![0x0F]),
            KeyCode::KeyP if ctrl => Some(vec![0x10]),
            KeyCode::KeyQ if ctrl => Some(vec![0x11]),
            KeyCode::KeyR if ctrl => Some(vec![0x12]),
            KeyCode::KeyS if ctrl => Some(vec![0x13]),
            KeyCode::KeyT if ctrl => Some(vec![0x14]),
            KeyCode::KeyU if ctrl => Some(vec![0x15]),
            KeyCode::KeyV if ctrl => Some(vec![0x16]),
            KeyCode::KeyW if ctrl => Some(vec![0x17]),
            KeyCode::KeyX if ctrl => Some(vec![0x18]),
            KeyCode::KeyY if ctrl => Some(vec![0x19]),
            KeyCode::KeyZ if ctrl => Some(vec![0x1A]),

            // Backspace
            KeyCode::Backspace => Some(vec![0x7F]),

            // Enter
            KeyCode::Enter => Some(vec![b'\r']),

            // Tab
            KeyCode::Tab => Some(vec![b'\t']),

            // Escape
            KeyCode::Escape => Some(vec![0x1B]),

            // Arrow keys
            KeyCode::ArrowUp => Some(b"\x1b[A".to_vec()),
            KeyCode::ArrowDown => Some(b"\x1b[B".to_vec()),
            KeyCode::ArrowRight => Some(b"\x1b[C".to_vec()),
            KeyCode::ArrowLeft => Some(b"\x1b[D".to_vec()),

            // Home/End
            KeyCode::Home => Some(b"\x1b[H".to_vec()),
            KeyCode::End => Some(b"\x1b[F".to_vec()),

            // Page Up/Down
            KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
            KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),

            // Insert/Delete
            KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
            KeyCode::Delete => Some(b"\x1b[3~".to_vec()),

            // Function keys
            KeyCode::F1 => Some(b"\x1bOP".to_vec()),
            KeyCode::F2 => Some(b"\x1bOQ".to_vec()),
            KeyCode::F3 => Some(b"\x1bOR".to_vec()),
            KeyCode::F4 => Some(b"\x1bOS".to_vec()),
            KeyCode::F5 => Some(b"\x1b[15~".to_vec()),
            KeyCode::F6 => Some(b"\x1b[17~".to_vec()),
            KeyCode::F7 => Some(b"\x1b[18~".to_vec()),
            KeyCode::F8 => Some(b"\x1b[19~".to_vec()),
            KeyCode::F9 => Some(b"\x1b[20~".to_vec()),
            KeyCode::F10 => Some(b"\x1b[21~".to_vec()),
            KeyCode::F11 => Some(b"\x1b[23~".to_vec()),
            KeyCode::F12 => Some(b"\x1b[24~".to_vec()),

            // Space
            KeyCode::Space => {
                if ctrl {
                    Some(vec![0x00]) // Ctrl-Space = NUL
                } else {
                    Some(vec![b' '])
                }
            }

            // Alphanumeric keys
            KeyCode::Digit0 if !ctrl && !shift => Some(vec![b'0']),
            KeyCode::Digit1 if !ctrl && !shift => Some(vec![b'1']),
            KeyCode::Digit2 if !ctrl && !shift => Some(vec![b'2']),
            KeyCode::Digit3 if !ctrl && !shift => Some(vec![b'3']),
            KeyCode::Digit4 if !ctrl && !shift => Some(vec![b'4']),
            KeyCode::Digit5 if !ctrl && !shift => Some(vec![b'5']),
            KeyCode::Digit6 if !ctrl && !shift => Some(vec![b'6']),
            KeyCode::Digit7 if !ctrl && !shift => Some(vec![b'7']),
            KeyCode::Digit8 if !ctrl && !shift => Some(vec![b'8']),
            KeyCode::Digit9 if !ctrl && !shift => Some(vec![b'9']),

            KeyCode::Digit0 if !ctrl && shift => Some(vec![b')']),
            KeyCode::Digit1 if !ctrl && shift => Some(vec![b'!']),
            KeyCode::Digit2 if !ctrl && shift => Some(vec![b'@']),
            KeyCode::Digit3 if !ctrl && shift => Some(vec![b'#']),
            KeyCode::Digit4 if !ctrl && shift => Some(vec![b'$']),
            KeyCode::Digit5 if !ctrl && shift => Some(vec![b'%']),
            KeyCode::Digit6 if !ctrl && shift => Some(vec![b'^']),
            KeyCode::Digit7 if !ctrl && shift => Some(vec![b'&']),
            KeyCode::Digit8 if !ctrl && shift => Some(vec![b'*']),
            KeyCode::Digit9 if !ctrl && shift => Some(vec![b'(']),

            KeyCode::KeyA if !ctrl => Some(vec![if shift { b'A' } else { b'a' }]),
            KeyCode::KeyB if !ctrl => Some(vec![if shift { b'B' } else { b'b' }]),
            KeyCode::KeyC if !ctrl => Some(vec![if shift { b'C' } else { b'c' }]),
            KeyCode::KeyD if !ctrl => Some(vec![if shift { b'D' } else { b'd' }]),
            KeyCode::KeyE if !ctrl => Some(vec![if shift { b'E' } else { b'e' }]),
            KeyCode::KeyF if !ctrl => Some(vec![if shift { b'F' } else { b'f' }]),
            KeyCode::KeyG if !ctrl => Some(vec![if shift { b'G' } else { b'g' }]),
            KeyCode::KeyH if !ctrl => Some(vec![if shift { b'H' } else { b'h' }]),
            KeyCode::KeyI if !ctrl => Some(vec![if shift { b'I' } else { b'i' }]),
            KeyCode::KeyJ if !ctrl => Some(vec![if shift { b'J' } else { b'j' }]),
            KeyCode::KeyK if !ctrl => Some(vec![if shift { b'K' } else { b'k' }]),
            KeyCode::KeyL if !ctrl => Some(vec![if shift { b'L' } else { b'l' }]),
            KeyCode::KeyM if !ctrl => Some(vec![if shift { b'M' } else { b'm' }]),
            KeyCode::KeyN if !ctrl => Some(vec![if shift { b'N' } else { b'n' }]),
            KeyCode::KeyO if !ctrl => Some(vec![if shift { b'O' } else { b'o' }]),
            KeyCode::KeyP if !ctrl => Some(vec![if shift { b'P' } else { b'p' }]),
            KeyCode::KeyQ if !ctrl => Some(vec![if shift { b'Q' } else { b'q' }]),
            KeyCode::KeyR if !ctrl => Some(vec![if shift { b'R' } else { b'r' }]),
            KeyCode::KeyS if !ctrl => Some(vec![if shift { b'S' } else { b's' }]),
            KeyCode::KeyT if !ctrl => Some(vec![if shift { b'T' } else { b't' }]),
            KeyCode::KeyU if !ctrl => Some(vec![if shift { b'U' } else { b'u' }]),
            KeyCode::KeyV if !ctrl => Some(vec![if shift { b'V' } else { b'v' }]),
            KeyCode::KeyW if !ctrl => Some(vec![if shift { b'W' } else { b'w' }]),
            KeyCode::KeyX if !ctrl => Some(vec![if shift { b'X' } else { b'x' }]),
            KeyCode::KeyY if !ctrl => Some(vec![if shift { b'Y' } else { b'y' }]),
            KeyCode::KeyZ if !ctrl => Some(vec![if shift { b'Z' } else { b'z' }]),

            // Punctuation
            KeyCode::Minus if !shift => Some(vec![b'-']),
            KeyCode::Minus if shift => Some(vec![b'_']),
            KeyCode::Equal if !shift => Some(vec![b'=']),
            KeyCode::Equal if shift => Some(vec![b'+']),
            KeyCode::BracketLeft if !shift => Some(vec![b'[']),
            KeyCode::BracketLeft if shift => Some(vec![b'{']),
            KeyCode::BracketRight if !shift => Some(vec![b']']),
            KeyCode::BracketRight if shift => Some(vec![b'}']),
            KeyCode::Backslash if !shift => Some(vec![b'\\']),
            KeyCode::Backslash if shift => Some(vec![b'|']),
            KeyCode::Semicolon if !shift => Some(vec![b';']),
            KeyCode::Semicolon if shift => Some(vec![b':']),
            KeyCode::Quote if !shift => Some(vec![b'\'']),
            KeyCode::Quote if shift => Some(vec![b'"']),
            KeyCode::Comma if !shift => Some(vec![b',']),
            KeyCode::Comma if shift => Some(vec![b'<']),
            KeyCode::Period if !shift => Some(vec![b'.']),
            KeyCode::Period if shift => Some(vec![b'>']),
            KeyCode::Slash if !shift => Some(vec![b'/']),
            KeyCode::Slash if shift => Some(vec![b'?']),
            KeyCode::Backquote if !shift => Some(vec![b'`']),
            KeyCode::Backquote if shift => Some(vec![b'~']),

            _ => None,
        }
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}
