//! A basic terminal multiplexing system. Unfortunately this part had to be handrolled, as there
//! are no actively-maintained cross-platform Curses implementations/equivalents for Rust...
//!
//! The API of this module is largely based around the the GLK windowing system, of which only a
//! subset is required for the Z-Machine.

use std::collections::VecDeque;
use std::io::{self, Write};
use std::ops::RangeBounds;
use std::mem;

use itertools::Itertools;
use itertools::FoldWhile;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crossterm::{
    self,
    cursor::MoveTo,
    execute, queue,
    style::{Attribute, SetAttribute},
    terminal::{
        disable_raw_mode, enable_raw_mode, size as term_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use crate::game::Result;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextFormat {
    bold: bool,
    italic: bool,
    reverse: bool,
}

impl TextFormat {
    pub fn update_terminal(&self, old: TextFormat) {
        if self == &old {
            return;
        }

        let mut stdout = io::stdout();
        let mut intensity_cleared = false;

        if (!self.bold && old.bold) || (!self.italic && old.italic) {
            queue!(stdout, SetAttribute(Attribute::NormalIntensity)).unwrap();
            intensity_cleared = true;
        }
        if self.bold && (!old.bold || intensity_cleared) {
            queue!(stdout, SetAttribute(Attribute::Bold)).unwrap();
        }
        if self.italic && (!old.italic || intensity_cleared) {
            queue!(stdout, SetAttribute(Attribute::Italic)).unwrap();
        }

        if self.reverse && !old.reverse {
            queue!(stdout, SetAttribute(Attribute::Reverse)).unwrap();
        } else if !self.reverse && old.reverse {
            queue!(stdout, SetAttribute(Attribute::NoReverse)).unwrap();
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    format: TextFormat,
    value: String,
}

impl Chunk {
    pub fn new<T: Into<String>>(format: TextFormat, value: T) -> Chunk {
        return Chunk {
            value: value.into(),
            format,
        };
    }

    pub fn put(&mut self, value: &str) {
        self.value.push_str(value);
    }

    pub fn wrap(&mut self, split: usize) -> Option<Chunk> {
        let mut split_point = 0;
        for (i, c) in self.value.char_indices() {
            let char_width = c.width().unwrap_or(0);
            if char_width + split_point > split {
                break;
            }
            split_point += char_width;
        }

        if split_point >= self.value.len() {
            None
        } else {
            let remaining = Chunk {
                value: self.value[split_point..].to_owned(),
                format: self.format,
            };
            self.value.truncate(split_point);
            Some(remaining)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Line {
    pub chunks: Vec<Chunk>,
}

impl Line {
    fn with_initial_chunk<S: Into<String>>(value: S, style: TextFormat) -> Line {
        Line {
            chunks: vec![Chunk {
                value: value.into(),
                format: style,
            }],
        }
    }

    fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }
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
    buffer: Vec<Chunk>,
    buffered: bool,
    lines: VecDeque<Line>,
    active_format: TextFormat,
    cursor: Cursor,
}

trait Printable {}

impl Window {
    fn new(x: u16, y: u16, width: u16, height: u16) -> Window {
        Window {
            x,
            y,
            width,
            height,
            buffered: false,
            buffer: Vec::default(),
            lines: VecDeque::with_capacity(height as usize),
            active_format: TextFormat::default(),
            cursor: Cursor { x, y },
        }
    }


    pub fn bold(&mut self) {
        self.active_format.bold = true;
    }

    pub fn emphasis(&mut self) {
        self.active_format.italic = true;
    }

    pub fn reverse(&mut self) {
        self.active_format.reverse = true;
    }

    pub fn reset_style(&mut self) {
        self.active_format = TextFormat::default();
    }

    pub fn line_break(&mut self) {
        self.lines.push_back(Line::default());
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.lines
            .get_mut(self.lines.len() - 1)
            .unwrap()
            .add_chunk(chunk);
    }

    fn print_char(&mut self, text: char) -> Result<()> {
        match self.buffer.last_mut() {
            Some(x) if x.format == self.active_format => {
                x.value.push(text);
            }
            _ => {
                self.buffer.push(Chunk::new(self.active_format, text));
            }
        }
        if !self.buffered {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn print(&mut self, text: &str) -> Result<()> {
        match self.buffer.last_mut() {
            Some(x) if x.format == self.active_format => {
                x.value.push_str(text);
            }
            _ => {
                self.buffer.push(Chunk::new(self.active_format, text));
            }
        }
        if !self.buffered {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<()> {
        let mut line_remaining = self.cursor.x;
        let mut buffer = mem::take(&mut self.buffer);

        if self.width == 0 || self.height == 0 {
            return Ok(())
        }

        let from_line = self.lines.len();
        let from_chunk = self.lines.back()
                .map(|x| x.chunks.len()).unwrap_or(0);
        for mut chunk in buffer.drain(..) {
            while let Some(other) = chunk.wrap(line_remaining as usize) {
                line_remaining = 0;
                self.add_chunk(chunk);
                self.line_break();
                chunk = other;
            }
        }
        let mut offset = 0;
        let mut stdout = io::stdout();
        for line in self.lines.range(from_line..) {
            for chunk in line.chunks[offset..].iter() {
                chunk.format.update_terminal(self.active_format);
                self.active_format = chunk.format;
                stdout.write(chunk.value.as_bytes())?;
            }
            offset = 0;
        }
        stdout.flush()?;
        Ok(())
    }

    // fn flush_buffer(&mut self) {
    //     for chunk in self.buffer() {
    //         chunk.format.update_terminal(active_format);
    //         active_format = chunk.format;
    //         for c in chunk.value() {
    //             let mut char_buffer = [0u8; 4];
    //             let available_width = self.width - self.cursor.x;
    //             let available_height = self.height.saturating_sub(self.cursor.y);
    //             let char_width = c.width().unwrap_or(0) as u16;
    //             if c == '\n' || char_width > available_width {
    //                 if available_height <= 1 {
    //                     self.more_prompt();
    //                 }
    //                 self.cursor.x = 0;
    //                 self.cursor.y += 1;
    //                 if !need_full_write {
    //                     queue!(
    //                         stdout,
    //                         MoveTo(self.x + self.cursor.x, self.y + self.cursor.y)
    //                     )
    //                     .unwrap();
    //                 }
    //                 let chunk = Chunk::new(self.active_format, &self.buffer[last_line_break..i]);
    //                 last_line_break = i + c.width().unwrap_or(1);
    //                 self.add_chunk(chunk);
    //                 self.line_break();
    //             } else {
    //                 self.cursor.x += char_width as u16;
    //             }
    //             if !need_full_write {
    //                 c.encode_utf8(&mut char_buffer);
    //                 stdout.write(&char_buffer).unwrap();
    //             }
    //         }
    //     }
    //     self.buffer.truncate(0);
    // }

    fn full_write(&mut self) {
        let mut stdout = io::stdout();
        let mut active_format = TextFormat::default();
        queue!(
            stdout,
            SetAttribute(Attribute::Reset),
            SetAttribute(Attribute::NoReverse),
        )
        .unwrap();

        let start = if self.lines.len() > self.height as usize {
            self.lines.len() - self.height as usize
        } else {
            0
        };
        let mut consumed_width = 0;
        for (i, line) in self.lines.range(start..self.lines.len()).enumerate() {
            queue!(stdout, MoveTo(self.x, self.y + (i as u16)),).unwrap();
            consumed_width = 0;
            for chunk in line.chunks.iter() {
                chunk.format.update_terminal(active_format);
                active_format = chunk.format;
                stdout.write(chunk.value.as_bytes()).unwrap();
                consumed_width += chunk.value.width();
            }
            if consumed_width < self.width as usize {
                for _ in 0..(self.width as usize - consumed_width) {
                    stdout.write(b" ").unwrap();
                }
            }
        }
        stdout.flush().unwrap();
        self.cursor.x = consumed_width as u16;
        self.cursor.y = if self.lines.len() > self.height as usize {
            self.height - 1
        } else {
            self.lines.len() as u16
        };
    }

    // pub fn _flush_buffer(&mut self) {
    //     if self.width == 0 {
    //         return;
    //     }

    //     if self.lines.is_empty() {
    //         self.lines.push_back(Line::default());
    //     }

    //     let mut stdout = io::stdout();
    //     let mut last_line_break = 0;
    //     let mut need_full_write = false;

    //     queue!(
    //         stdout,
    //         MoveTo(self.x + self.cursor.x, self.y + self.cursor.y)
    //     )
    //     .unwrap();

    //     for (i, c) in self.buffer.char_indices() {
    //         let mut char_buffer = [0u8; 4];
    //         let available_width = self.width - self.cursor.x;
    //         let available_height = self.height.saturating_sub(self.cursor.y);
    //         let char_width = c.width().unwrap_or(1) as u16;
    //         if c == '\n' || char_width > available_width {
    //             if available_height <= 1 {
    //                 need_full_write = true;
    //             }
    //             self.cursor.x = 0;
    //             self.cursor.y += 1;
    //             if !need_full_write {
    //                 queue!(
    //                     stdout,
    //                     MoveTo(self.x + self.cursor.x, self.y + self.cursor.y)
    //                 )
    //                 .unwrap();
    //             }
    //             let chunk = Chunk::new(self.active_format, &self.buffer[last_line_break..i]);
    //             last_line_break = i + c.width().unwrap_or(1);
    //             self.add_chunk(chunk);
    //             self.line_break();
    //         } else {
    //             self.cursor.x += char_width as u16;
    //         }
    //         if !need_full_write {
    //             c.encode_utf8(&mut char_buffer);
    //             stdout.write(&char_buffer).unwrap();
    //         }
    //     }
    //     if last_line_break < self.buffer.len() - 1 {
    //         let chunk = Chunk::new(
    //             self.active_format,
    //             &self.buffer[last_line_break..self.buffer.len()],
    //         );
    //         self.add_chunk(chunk);
    //     }

    //     if &&need_full_write {
    //         self.full_write();
    //     }

    //     stdout.flush().unwrap();
    //     self.buffer.truncate(0);
    // }
}

#[derive(Default, Clone, Debug)]
pub struct WindowManager {
    items: Vec<Option<WindowNode>>,
    active_window: usize,
}

#[derive(Clone, Debug)]
pub enum WindowNode {
    Window {
        window: Window,
        parent: Option<usize>,
    },
    PairWindow {
        direction: SplitDirection,
        parent: Option<usize>,
        child_left: usize,
        child_right: usize,
    },
}

impl WindowNode {
    fn set_parent(&mut self, new_parent: Option<usize>) {
        match self {
            WindowNode::Window { ref mut parent, .. } => *parent = new_parent,
            WindowNode::PairWindow { ref mut parent, .. } => *parent = new_parent,
        }
    }
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Clear(ClearType::All), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

impl WindowManager {
    pub fn init() -> Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            MoveTo(0, 0),
            Clear(ClearType::All)
        )?;
        enable_raw_mode()?;
        Ok(())
    }

    pub fn print(&mut self, text: &str) -> Result<()> {
        let window = self
            .get_mut_window(self.active_window)
            .expect("Tried to write to non-existant window");
        window.print(text)?;
        Ok(())
    }

    pub fn print_char(&mut self, text: char) -> Result<()> {
        let window = self
            .get_mut_window(self.active_window)
            .expect("Tried to write to non-existant window");
        window.print_char(text)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        for item in self.items {
            match item {
                Some(WindowNode::Window{window, ..}) => {
                    window.flush()?;
                },
                _ => {}
            }
        }
    }

    pub fn bold(&mut self) -> Result<()> {
        let window = self
            .get_mut_window(self.active_window)
            .expect("Tried to write to non-existant window");
        window.bold();
        Ok(())
    }

    pub fn emphasis(&mut self) -> Result<()> {
        let window = self
            .get_mut_window(self.active_window)
            .expect("Tried to write to non-existant window");
        window.emphasis();
        Ok(())
    }
    pub fn reverse(&mut self) -> Result<()> {
        let window = self
            .get_mut_window(self.active_window)
            .expect("Tried to write to non-existant window");
        window.reverse();
        Ok(())
    }

    pub fn reset_style(&mut self) -> Result<()> {
        let window = self
            .get_mut_window(self.active_window)
            .expect("Tried to write to non-existant window");
        window.reset_style();
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
            None | Some(WindowNode::PairWindow { .. }) => None,
            Some(WindowNode::Window { window, .. }) => Some(window),
        }
    }

    /// Retrieve the parent of the provided window, as well as its ID.
    fn parent(&self, _child: usize) -> Option<(usize, &WindowNode)> {
        todo!()
    }

    /// Retrieve the left-side child of the provided window, as well as its ID.
    fn child_left(&self, _node: usize) -> Option<(usize, &WindowNode)> {
        todo!()
    }

    /// Retrieve the right-side child of the provided window, as well as its ID.
    fn child_right(&self, _node: usize) -> Option<(usize, &WindowNode)> {
        todo!()
    }

    /// Divides the provided window into two.
    pub fn split(&mut self, node: usize, direction: SplitDirection, size: u16) -> Result<usize> {
        if self.items.is_empty() {
            let (cols, rows) = term_size()?;
            let child = Window::new(0, 0, cols.into(), rows.into());
            self.items.push(Some(WindowNode::Window {
                window: child,
                parent: None,
            }));
            return Ok(0);
        }

        let (mut existing, parent) = match &mut self.items[node] {
            Some(WindowNode::PairWindow { .. }) => {
                panic!("Can't split a window that's already split!");
            }
            None => {
                panic!("No such window!");
            }
            Some(WindowNode::Window { window, parent }) => (window, parent.to_owned()),
        };

        let new_window = match direction {
            SplitDirection::Above => {
                existing.y += size;
                existing.height -= size;
                Window::new(existing.x, existing.y - size, existing.width, size)
            }
            SplitDirection::Below => {
                existing.height -= size;
                Window::new(
                    existing.x,
                    existing.y + existing.height,
                    existing.width,
                    size,
                )
            }
            SplitDirection::Left => {
                existing.x += size;
                existing.width -= size;
                Window::new(existing.x - size, existing.y, size, existing.height)
            }
            SplitDirection::Right => {
                existing.width -= size;
                Window::new(
                    existing.x + existing.width,
                    existing.y,
                    size,
                    existing.height,
                )
            }
        };

        let new_window_id = self.insert_node(WindowNode::Window {
            window: new_window,
            parent: None,
        });

        let split_node = WindowNode::PairWindow {
            direction,
            parent,
            child_left: node,
            child_right: new_window_id,
        };

        let split_node_id = self.insert_node(split_node);
        self.items[node]
            .as_mut()
            .unwrap()
            .set_parent(Some(split_node_id));
        self.items[new_window_id]
            .as_mut()
            .unwrap()
            .set_parent(Some(split_node_id));

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
    fn destroy(&mut self, _node: usize) {
        todo!();
    }
}
