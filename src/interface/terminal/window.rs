//! A basic terminal multiplexing system. Unfortunately this part had to be handrolled, as there
//! are no actively-maintained cross-platform Curses implementations/equivalents for Rust...
//!
//! The API of this module is largely based around the the GLK windowing system, of which only a
//! subset is required for the Z-Machine.

use std::collections::VecDeque;
use std::io::{self, Write};
use std::mem;

use unicode_width::UnicodeWidthChar;
use crossterm::{
    self,
    cursor::{position as cursor_pos, MoveLeft, MoveTo},
    event::{self, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{
        disable_raw_mode, enable_raw_mode, size as term_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use crate::game::Result;

#[derive(Clone, Debug, Default)]
struct TextFormat {
    bold: bool,
    italic: bool,
    reverse: bool,
}

#[derive(Clone, Debug)]
struct Chunk {
    format: TextFormat,
    value: String,
}

#[derive(Clone, Debug)]
struct Line {
    chunks: Vec<Chunk>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SplitDirection {
    Above,
    Below,
    Left,
    Right,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct Cursor {
    x: u16,
    y: u16,
}

#[derive(Default, Clone, Debug)]
pub struct Window {
    width: u16,
    height: u16,
    x: u16,
    y: u16,
    lines: VecDeque<Line>,
    invalidations: Vec<usize>,
    active_format: TextFormat,
    cursor: Cursor,
}

impl Window {
    fn new(x: u16, y: u16, width: u16, height: u16) -> Window {
        Window {
            x,
            y,
            width,
            height,
            lines: VecDeque::with_capacity(height as usize),
            invalidations: Vec::new(),
            active_format: TextFormat::default(),
            cursor: Cursor {x, y},
        }
    }

    pub fn write<T: Into<String>>(&mut self, text: T) {
        let text: String = text.into();
        let mut stdout = io::stdout();
        queue!(stdout, MoveTo(self.x + self.cursor.x, self.y + self.cursor.y));
        for c in text.chars() {
            let mut char_buffer = [0u8; 4];
            let available_space = (self.width - self.cursor.x).saturating_sub(1);
            let char_width = c.width().unwrap_or(0) as u16;
            if char_width > available_space {
                self.cursor.y += 1;
                self.cursor.x = 0;
                queue!(stdout, MoveTo(self.x + self.cursor.x, self.y + self.cursor.y));
            } else {
                self.cursor.x += (char_width as u16);
            }
            c.encode_utf8(&mut char_buffer);
            stdout.write(&char_buffer);
        }
        stdout.flush();
    }
}

#[derive(Default, Clone, Debug)]
pub struct WindowManager {
    items: Vec<Option<WindowNode>>,
}

#[derive(Clone, Debug)]
pub enum WindowNode {
    Window{ window: Window, parent: Option<usize>},
    PairWindow{ direction: SplitDirection, parent: Option<usize>, child_left: usize, child_right: usize },
}

impl WindowNode {
    fn set_parent(&mut self, new_parent: Option<usize>) {
        match self {
            WindowNode::Window { ref mut parent, ..} => *parent = new_parent,
            WindowNode::PairWindow { ref mut parent, ..} => *parent = new_parent,
        }
    }
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

impl WindowManager {
    pub fn init() -> Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, MoveTo(0, 0))?;
        enable_raw_mode()?;
        Ok(())
    }

    pub fn get_mut_window_node(&mut self, id: usize) -> Option<&mut WindowNode> {
        if id >= self.items.len() {
            None
        } else {
            self.items[id].as_mut()
        }
    }

    pub fn get_mut_window(&mut self, id: usize) -> Option<&mut Window> {
        match self.get_mut_window_node(id) {
            None | Some(WindowNode::PairWindow{..}) => None,
            Some(WindowNode::Window{window, ..}) => Some(window)
        }
    }

    /// Retrieve the parent of the provided window, as well as its ID.
    fn parent(&self, child: usize) -> Option<(usize, &WindowNode)> {
        todo!()
    }

    /// Retrieve the left-side child of the provided window, as well as its ID.
    fn child_left(&self, node: usize) -> Option<(usize, &WindowNode)> {
        todo!()
    }

    /// Retrieve the right-side child of the provided window, as well as its ID.
    fn child_right(&self, node: usize) -> Option<(usize, &WindowNode)> {
        todo!()
    }

    /// Divides the provided window into two.
    pub fn split(&mut self, node: usize, direction: SplitDirection, size: u16) -> Result<usize> {
        if self.items.is_empty() {
            let (cols, rows) = term_size()?;
            let child = Window::new(0, 0, cols.into(), rows.into());
            self.items.push(Some(WindowNode::Window{window: child, parent: None}));
            return Ok(0)
        }

        let (mut existing, parent) = match &mut self.items[node] {
            Some(WindowNode::PairWindow{..}) => {
                panic!("Can't split a window that's already split!");
            }
            None => {
                panic!("No such window!");
            }
            Some(WindowNode::Window{window, parent}) => (window, parent.to_owned()),
        };


        let new_window = match direction {
            SplitDirection::Above => {
                existing.y += size;
                existing.height -= size;
                Window::new(existing.x, existing.y - size, existing.width, size)
            }
            SplitDirection::Below => {
                existing.height -= size;
                Window::new(existing.x, existing.y + existing.height, existing.width, size)
            }
            SplitDirection::Left => {
                existing.x += size;
                existing.width -= size;
                Window::new(existing.x - size, existing.y, size, existing.height)
            }
            SplitDirection::Right => {
                existing.width -= size;
                Window::new(existing.x + existing.width, existing.y, size, existing.height)
            }
        };

        let new_window_id = self.insert_node(WindowNode::Window{window: new_window, parent: None});

        let split_node = WindowNode::PairWindow {
            direction,
            parent,
            child_left: node,
            child_right: new_window_id,
        };

        let split_node_id = self.insert_node(split_node);
        self.items[node].as_mut().unwrap().set_parent(Some(split_node_id));
        self.items[new_window_id].as_mut().unwrap().set_parent(Some(split_node_id));

        Ok(new_window_id)
    }

    fn insert_node(&mut self, node: WindowNode) -> usize {
        for i in 0..self.items.len() {
            if matches!(self.items[i], None) {
                self.items[i] = Some(node);
                return i;
            }
        }
        self.items.push(Some(node));
        self.items.len() - 1
    }

    /// Drop the provided window id.
    fn destroy(&mut self, node: usize) {
        todo!();
    }
}
