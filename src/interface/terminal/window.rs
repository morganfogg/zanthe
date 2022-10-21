//! A basic terminal multiplexing system. Unfortunately this part had to be handrolled, as there
//! are no actively-maintained cross-platform Curses implementations/equivalents for Rust...
//!
//! The API of this module is largely based around the the GLK windowing system, of which only a
//! subset is required for the Z-Machine.

use std::collections::VecDeque;

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
    items: Vec<Option<usize>>,
}

enum LineageKind {
    Window(Window),
    Split,
}

impl Lineage {
    fn parent(&self, child: usize) -> usize {
        self.items[i / 2].unwrap()
    }

    fn child_left(&self, parent: usize) -> Option<usize> {
        let i = i * 2;
        if i < self.items.len() {
            self.items[i]
        } else {
            None
        }
    }

    fn child_right(&self, parent: usize) -> Option<usize> {
        let i = i * 2 + 1;
        if i < self.items.len() {
            self.items[i]
        } else {
            None
        }
    }

    fn drop(&self, leaf: usize) {
        self.items[leaf] = None;
        let a = leaf;
        let b = leaf;
        loop {
            a *= 2;
            b = b * 2 + 1;
            if x >= self.items.len() {
                break;
            }
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
