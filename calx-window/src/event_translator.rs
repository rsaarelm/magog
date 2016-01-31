use time;
use glium::{self, glutin};
use key::Key;
use event::{Event, MouseButton};
use window::LogicalResolution;
use scancode;

/// Translate Glutin events to Calx events.
pub struct EventTranslator {
    layout_independent_keys: bool,
    queue: Vec<Event>,
    drag_start: [f32; 2],
    pub mouse_pos: [f32; 2],
    /// If mouse is down, timestamp for when it was pressed.
    pub mouse_pressed: [Option<f64>; 3],
}

impl EventTranslator {
    pub fn new(layout_independent_keys: bool) -> EventTranslator {
        EventTranslator {
            layout_independent_keys: layout_independent_keys,
            queue: Vec::new(),
            drag_start: [0.0, 0.0],
            mouse_pos: [0.0, 0.0],
            mouse_pressed: [None, None, None],
        }
    }

    /// Return true if the event is for suspending the window.
    fn process_event(&mut self,
                     resolution: &LogicalResolution,
                     glutin_event: glutin::Event)
                     -> bool {
        static MOUSE_DRAG_THRESHOLD_T: f64 = 0.1;

        match glutin_event {
            glutin::Event::Resized(_w, _h) => {}
            glutin::Event::Moved(_x, _y) => {}
            glutin::Event::DroppedFile(_path) => {}
            glutin::Event::Touch(_touch) => {}
            // TODO: Refresh indicates that the window needs repainting.
            // Probably should handle it somehow?
            glutin::Event::Refresh => {}

            glutin::Event::Awakened => {
                return false;
            }

            glutin::Event::Focused(b) => {
                return !b;
            }

            glutin::Event::Suspended(b) => {
                return b;
            }

            glutin::Event::Closed => {
                self.queue.push(Event::Quit);
            }

            glutin::Event::ReceivedCharacter(ch) => {
                self.queue.push(Event::Char(ch));
            }

            glutin::Event::KeyboardInput(action, scan, vko) => {
                let scancode_mapped = if self.layout_independent_keys &&
                                         (scan as usize) <
                                         scancode::MAP.len() {
                    scancode::MAP[scan as usize]
                } else {
                    None
                };

                if let Some(key) = scancode_mapped.or(vko.map(vko_to_key)
                                                         .unwrap_or(None)) {
                    self.queue
                        .push(if action == glutin::ElementState::Pressed {
                            Event::KeyPress(key)
                        } else {
                            Event::KeyRelease(key)
                        });
                }
            }

            glutin::Event::MouseMoved((x, y)) => {
                let pixel_pos = resolution.screen_to_canvas(&[x, y]);
                self.mouse_pos = [pixel_pos[0] as f32, pixel_pos[1] as f32];

                // See if there are mouse drags going on with the
                // buttons, create extra "ongoing drag" events if so.
                let current_t = time::precise_time_s();
                for &b in [MouseButton::Left,
                           MouseButton::Right,
                           MouseButton::Middle]
                              .iter() {
                    if let Some(press_t) = self.mouse_pressed[b as usize] {
                        if current_t - press_t > MOUSE_DRAG_THRESHOLD_T {
                            self.queue.push(Event::MouseDrag(b,
                                                             self.drag_start,
                                                             self.mouse_pos));
                        }
                    }
                }
                self.queue.push(Event::MouseMove(self.mouse_pos));
            }
            glutin::Event::MouseWheel(glutin::MouseScrollDelta::LineDelta(x,
                                                                          _)) => {
                {
                    self.queue.push(Event::MouseWheel(x as i32));
                }
            }
            // TODO: Handle LineDelta and PixelDelta events...
            glutin::Event::MouseWheel(_) => {}

            glutin::Event::MouseInput(state, button) => {
                if let Some(button) = match button {
                    glutin::MouseButton::Left => Some(MouseButton::Left),
                    glutin::MouseButton::Right => Some(MouseButton::Right),
                    glutin::MouseButton::Middle => Some(MouseButton::Middle),
                    glutin::MouseButton::Other(_) => None,
                } {
                    match state {
                        glutin::ElementState::Pressed => {
                            // Possibly the start of a mouse drag, make
                            // note of when the button was pressed and
                            // where the cursor was at the time.
                            self.drag_start = self.mouse_pos;
                            self.mouse_pressed[button as usize] =
                                Some(time::precise_time_s());
                            self.queue.push(Event::MousePress(button));
                        }
                        glutin::ElementState::Released => {
                            let release_t = time::precise_time_s();

                            // Interpret short mouse presses as clicks
                            // and long ones as drags.
                            if let Some(press_t) =
                                   self.mouse_pressed[button as usize] {
                                if release_t - press_t >
                                   MOUSE_DRAG_THRESHOLD_T {
                                    self.queue.push(Event::MouseDragEnd(
                                            button, self.drag_start, self.mouse_pos));
                                }
                                self.queue.push(Event::MouseClick(button));
                            }
                            self.mouse_pressed[button as usize] = None;
                            self.queue.push(Event::MouseRelease(button));
                        }
                    }
                }
            }
        }

        false
    }

    /// Read new events from Glium into the event queue. Will suspend the
    /// thread if an application suspending event appears in the event queue.
    pub fn pump(&mut self,
                display: &mut glium::Display,
                resolution: &LogicalResolution) {
        let mut window_suspended = false;

        loop {
            let glutin_event = if window_suspended {
                // If the window is suspended, use the blocking event query so
                // that the calling thread will sleep until an event wakes the
                // window up.
                display.wait_events().next()
            } else {
                display.poll_events().next()
            };

            match glutin_event {
                Some(glutin_event) => {
                    window_suspended = self.process_event(resolution,
                                                          glutin_event);
                }
                None => {
                    break;
                }
            }
        }
    }

    pub fn next(&mut self) -> Option<Event> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }
}

fn vko_to_key(vko: glutin::VirtualKeyCode) -> Option<Key> {
    use glium::glutin::VirtualKeyCode::*;

    match vko {
        A => Some(Key::A),
        B => Some(Key::B),
        C => Some(Key::C),
        D => Some(Key::D),
        E => Some(Key::E),
        F => Some(Key::F),
        G => Some(Key::G),
        H => Some(Key::H),
        I => Some(Key::I),
        J => Some(Key::J),
        K => Some(Key::K),
        L => Some(Key::L),
        M => Some(Key::M),
        N => Some(Key::N),
        O => Some(Key::O),
        P => Some(Key::P),
        Q => Some(Key::Q),
        R => Some(Key::R),
        S => Some(Key::S),
        T => Some(Key::T),
        U => Some(Key::U),
        V => Some(Key::V),
        W => Some(Key::W),
        X => Some(Key::X),
        Y => Some(Key::Y),
        Z => Some(Key::Z),
        Escape => Some(Key::Escape),
        F1 => Some(Key::F1),
        F2 => Some(Key::F2),
        F3 => Some(Key::F3),
        F4 => Some(Key::F4),
        F5 => Some(Key::F5),
        F6 => Some(Key::F6),
        F7 => Some(Key::F7),
        F8 => Some(Key::F8),
        F9 => Some(Key::F9),
        F10 => Some(Key::F10),
        F11 => Some(Key::F11),
        F12 => Some(Key::F12),
        Scroll => Some(Key::ScrollLock),
        Pause => Some(Key::Pause),
        Insert => Some(Key::Insert),
        Home => Some(Key::Home),
        Delete => Some(Key::Delete),
        End => Some(Key::End),
        PageDown => Some(Key::PageDown),
        PageUp => Some(Key::PageUp),
        Left => Some(Key::Left),
        Up => Some(Key::Up),
        Right => Some(Key::Right),
        Down => Some(Key::Down),
        Return => Some(Key::Enter),
        Space => Some(Key::Space),
        Numlock => Some(Key::NumLock),
        Numpad0 => Some(Key::Pad0),
        Numpad1 => Some(Key::Pad1),
        Numpad2 => Some(Key::Pad2),
        Numpad3 => Some(Key::Pad3),
        Numpad4 => Some(Key::Pad4),
        Numpad5 => Some(Key::Pad5),
        Numpad6 => Some(Key::Pad6),
        Numpad7 => Some(Key::Pad7),
        Numpad8 => Some(Key::Pad8),
        Numpad9 => Some(Key::Pad9),
        Add => Some(Key::PadPlus),
        Apostrophe => Some(Key::Apostrophe),
        Backslash => Some(Key::Backslash),
        Comma => Some(Key::Comma),
        Decimal => Some(Key::PadDecimal),
        Divide => Some(Key::PadDivide),
        Equals => Some(Key::PadEquals),
        Grave => Some(Key::Grave),
        LAlt => Some(Key::LeftAlt),
        LBracket => Some(Key::LeftBracket),
        LControl => Some(Key::LeftControl),
        LMenu => Some(Key::LeftSuper),
        LShift => Some(Key::LeftShift),
        LWin => Some(Key::LeftSuper),
        Minus => Some(Key::Minus),
        Multiply => Some(Key::PadMultiply),
        NumpadComma => Some(Key::PadDecimal),
        NumpadEnter => Some(Key::PadEnter),
        NumpadEquals => Some(Key::PadEquals),
        Period => Some(Key::Period),
        RAlt => Some(Key::RightAlt),
        RBracket => Some(Key::RightBracket),
        RControl => Some(Key::RightControl),
        RMenu => Some(Key::RightSuper),
        RShift => Some(Key::RightShift),
        RWin => Some(Key::RightSuper),
        Semicolon => Some(Key::Semicolon),
        Slash => Some(Key::Slash),
        Subtract => Some(Key::PadMinus),
        Tab => Some(Key::Tab),
        _ => None,
    }
}
