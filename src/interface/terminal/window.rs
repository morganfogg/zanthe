//! A basic terminal multiplexing system. Unfortunately this part had to be handrolled, as there
//! are no actively-maintained cross-platform Curses implementations/equivalents for Rust...
//!
//! The API of this module is largely based around the the GLK windowing system, of which only a
//! subset is required for the Z-Machine.

use std::collections::VecDeque;
use std::mem;

struct TextFormat {
    bold: bool,
    italic: bool,
    reverse: bool,
}

struct Chunk {
    format: TextFormat,
    value: String,
}

struct Line {
    chunks: Vec<Chunk>,
}

enum SplitDirection {
    Above,
    Below,
    Left,
    Right,
}

#[derive(Default)]
struct Window {
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    lines: VecDeque<Line>,
    invalidations: Vec<usize>,
}

impl Window {
    fn new(id: u32, x: usize, y: usize, width: usize, height: usize) -> Window {
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

#[derive(Default)]
struct Lineage {
    items: Vec<Option<WindowKind>>,
}

enum WindowKind {
    Window(Window),
    PairWindow(Direction),
}

impl Lineage {
    fn parent(&self, child: usize) -> usize {
        if child == 1 {panic!("Tried to find parent of root window.")}
        self.items[child / 2].unwrap()
    }

    fn split(&mut self, node: usize, direction: Direction) {
        if match!(self.items[node], WindowKind::PairWindow(_)) {
            panic!("Can't split a window that's already split");
        }

        let window = mem::replace(self.items[node], WindowKind::PairWindow);

    }

    fn child_left(&self, node: usize) -> Option<usize> {
        let i = node * 2;
        if i < self.items.len() {
            self.items[i]
        } else {
            None
        }
    }

    fn child_right(&self, node: usize) -> Option<usize> {
        let i = node * 2 + 1;
        if i < self.items.len() {
            self.items[i]
        } else {
            None
        }
    }

    /// Divides the provided window into two.
    fn split(&mut self, node: usize, child: WindowKind) {
        let left_index = node * 2;
        let right_index = left_index + 1;

        match self.items[node]  {
            Some(WindowKind::PairWindow(_)) => {
                panic!("Can't split a window that's already split!");
            }
            None => {
                panic!("No such window!");
            }
            _ => {}
        }

        let existing = mem::replace(self.items[node], Some(WindowKind::PairWindow));
        self.items[left_index] = existing;
        self.items[right_index] = child;
    }

    /// Drop the provided window.
    fn destroy(&mut self, node: usize) {
        let destroy_stack = vec![node];
        while !destroy_stack.is_empty() {
            if matches!(self.items[node], Some(WindowKind::PairWindow(_))) {
                destroy_stack.extend_from_slice(&[node * 2, node * 2 + 1]);
            }
            self.items[node] = None;
        }
    }
}

#[derive(Default)]
struct WindowManager {
    width: usize,
    height: usize,
    windows: Vec<Window>,
    active_window: usize,
    lineage: Lineage,
}

impl WindowManager {
    fn new(width: usize, height: usize) -> {
        WindowManager {
            width,
            height,
            windows: Vec::new(),
            active_window: 0,
            lineage: Vec::new(),
        }
    }

    /// Create a new window by splitting an existing window. `base_window`, `size` and `direction`
    /// are all ignored if there are no active windows.
    fn open_window(&mut self, base_window: usize, size: usize, direction: Direction) {
        if self.windows.is_empty() {
            self.windows.append(Window::new(0, 0, self.width, self.height));
            self.active_window = 1;
        } else {

        }
    }
}
