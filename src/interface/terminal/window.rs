//! A basic terminal multiplexing system. Unfortunately this part had to be handrolled, as there
//! are no actively-maintained cross-platform Curses implementations/equivalents for Rust...
//!
//! The API of this module is largely based around the the GLK windowing system, of which only a
//! subset is required for the Z-Machine.

use std::collections::VecDeque;
use std::io;
use std::mem;

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

#[derive(Clone, Debug)]
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

#[derive(Default, Clone, Debug)]
pub struct Window {
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    lines: VecDeque<Line>,
    invalidations: Vec<usize>,
}

impl Window {
    fn new(x: usize, y: usize, width: usize, height: usize) -> Window {
        Window {
            x,
            y,
            width,
            height,
            lines: VecDeque::with_capacity(height),
            invalidations: Vec::new(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct WindowManager {
    items: Vec<Option<WindowNode>>,
}

#[derive(Clone, Debug)]
pub enum WindowNode {
    Window(Window),
    PairWindow(SplitDirection),
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

fn parent_id(id: usize) -> usize {
    (id - 1) / 2
}

fn child_left_id(id: usize) -> usize {
    id * 2 + 1
}

fn child_right_id(id: usize) -> usize {
    id * 2 + 2
}

impl WindowManager {
    pub fn init() -> Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, MoveTo(0, 0))?;
        enable_raw_mode()?;
        Ok(())
    }

    /// Retrieve the parent of the provided window, as well as its ID.
    fn parent(&self, child: usize) -> Option<(usize, &WindowNode)> {
        if child == 1 {
            panic!("Tried to find parent of root window.")
        } else if child == 0 {
            panic!("Invalid window ID");
        }
        self.items[parent_id(child)].as_ref().map(|x| (parent_id(child), x))
    }

    /// Retrieve the left-side child of the provided window, as well as its ID.
    fn child_left(&self, node: usize) -> Option<(usize, &WindowNode)> {
        let i = child_left_id(node);
        if i < self.items.len() {
            self.items[i].as_ref().map(|x| (i, x))
        } else {
            None
        }
    }

    /// Retrieve the right-side child of the provided window, as well as its ID.
    fn child_right(&self, node: usize) -> Option<(usize, &WindowNode)> {
        let i = child_right_id(node);
        if i < self.items.len() {
            self.items[i].as_ref().map(|x| (i, x))
        } else {
            None
        }
    }

    /// Divides the provided window into two.
    pub fn split(&mut self, node: usize, direction: SplitDirection, size: usize) -> Result<usize> {
        if self.items.is_empty() {
            let (cols, rows) = term_size()?;
            let child = Window::new(0, 0, cols.into(), rows.into());
            self.items.push(Some(WindowNode::Window(child)));
            return Ok(0)
        }

        let left_index = child_left_id(node);
        let right_index = child_right_id(node);

        if right_index > 64 {
            panic!("Maximum split depth exceeded");
        }

        if right_index >= self.items.len() {
            self.items.resize(right_index + 1, None);
        }

        let existing = mem::replace(
            &mut self.items[node],
            Some(WindowNode::PairWindow(direction)),
        );

        let mut existing = match existing {
            Some(WindowNode::PairWindow(_)) => {
                panic!("Can't split a window that's already split!");
            }
            None => {
                panic!("No such window!");
            }
            Some(WindowNode::Window(x)) => x,
        };

        let child = match direction {
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

        self.items[left_index] = Some(WindowNode::Window(existing));
        self.items[right_index] = Some(WindowNode::Window(child));
        Ok(right_index)
    }

    /// Drop the provided window id.
    fn destroy(&mut self, node: usize) {
        let mut destroy_stack = vec![node];
        while !destroy_stack.is_empty() {
            if matches!(self.items[node], Some(WindowNode::PairWindow(_))) {
                destroy_stack.extend_from_slice(&[child_left_id(node), child_right_id(node)]);
            }
            self.items[node] = None;
        }
    }
}
