use std::collections::VecDeque;
use std::io::{self, prelude::*};
use std::mem;

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
use tracing::warn;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::game::Result;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Above,
    Below,
    Left,
    Right,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Style {
    bold: bool,
    italic: bool,
    reverse: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Constraint {
    RightFixed(u16),
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Rectangle {
    pub fn split_fixed(&self, direction: Direction, size: u16) -> (Self, Self) {
        let mut sized = self.clone();
        let mut unconstrained = self.clone();

        match direction {
            Direction::Above => {
                sized.height = size;
                unconstrained.height -= size;
                unconstrained.y += size;
            }
            Direction::Below => {
                sized.height = size;
                unconstrained.height -= size;
                sized.y += unconstrained.height;
            }
            Direction::Left => {
                sized.width = size;
                unconstrained.width -= size;
                unconstrained.x += size;
            }
            Direction::Right => {
                sized.width = size;
                unconstrained.width -= size;
                sized.y += unconstrained.width;
            }
        };
        (sized, unconstrained)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Cursor {
    x: u16,
    y: u16,
}

impl Cursor {
    fn default_from(rect: Rectangle) -> Cursor {
        Cursor {
            x: rect.x,
            y: rect.y,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Window {
    kind: WindowKind,
    active_style: Style,
    screen_model: ScreenModel,
}

#[derive(Clone, Copy, Debug)]
struct ScreenModel {
    area: Rectangle,
    cursor: Cursor,
}

impl ScreenModel {
    fn sync_cursor(&self) -> Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            MoveTo(self.area.x + self.cursor.x, self.area.y + self.cursor.y)
        )?;
        Ok(())
    }
}

impl Window {
    fn redraw(&mut self) -> Result<()> {
        self.screen_model.cursor.x = 0;
        self.screen_model.cursor.y = 0;

        self.screen_model.sync_cursor()?;
        match &mut self.kind {
            WindowKind::TextStream(stream) => {
                stream.redraw(&mut self.screen_model)?;
            }
            _ => {
                todo!();
            }
        }
        Ok(())
    }

    fn print(&mut self, text: &str, immediate: bool) -> Result<()> {
        match &mut self.kind {
            WindowKind::TextStream(stream) => {
                match stream
                    .buffer
                    .last_mut()
                    .filter(|chunk| chunk.style == self.active_style)
                {
                    Some(chunk) => {
                        chunk.value.push_str(text.into());
                    }
                    None => stream.buffer.push(Chunk {
                        value: text.into(),
                        style: self.active_style,
                    }),
                }
                if immediate {
                    stream.flush_buffer(&mut self.screen_model)?;
                }
            }
            _ => {
                todo!();
            }
        }
        Ok(())
    }

    fn print_char(&mut self, text: char, immediate: bool) -> Result<()> {
        match &mut self.kind {
            WindowKind::TextStream(stream) => {
                match stream
                    .buffer
                    .last_mut()
                    .filter(|chunk| chunk.style == self.active_style)
                {
                    Some(chunk) => {
                        chunk.value.push(text);
                    }
                    None => stream.buffer.push(Chunk {
                        value: text.into(),
                        style: self.active_style,
                    }),
                }
                if immediate {
                    stream.flush_buffer(&mut self.screen_model)?;
                }
            }
            _ => {
                todo!();
            }
        }
        Ok(())
    }

    fn backspace(&mut self) -> Result<()> {
        match &mut self.kind {
            WindowKind::TextStream(stream) => {
                stream.flush_buffer(&mut self.screen_model)?;
                while let Some(line) = stream.lines.back_mut() {
                    while let Some(chunk) = line.last_mut() {
                        if !chunk.value.is_empty() {
                            let mut stdout = io::stdout();
                            chunk.value.pop();
                            self.screen_model.cursor.x -= 1;
                            self.screen_model.sync_cursor()?;
                            stdout.write(b" ")?;
                            stdout.flush()?;
                            self.screen_model.sync_cursor()?;
                            return Ok(());
                        } else {
                            line.pop();
                        }
                    }
                    stream.lines.pop_back();
                    self.screen_model.cursor.y -= 1;
                    self.screen_model.cursor.x = stream.last_line_width();
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn flush_buffer(&mut self) -> Result<()> {
        match &mut self.kind {
            WindowKind::TextStream(stream) => {
                stream.flush_buffer(&mut self.screen_model)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum WindowKind {
    TextStream(TextStream),
    TextGrid(TextGrid),
}

#[derive(Debug, Default, Clone)]
pub struct TextStream {
    lines: VecDeque<Vec<Chunk>>,
    buffer: Vec<Chunk>,
}

impl TextStream {
    fn last_line_width(&self) -> u16 {
        if let Some(line) = self.lines.back() {
            line.iter()
                .map(|chunk| chunk.value.width())
                .sum::<usize>()
                .try_into()
                .unwrap()
        } else {
            0
        }
    }

    fn flush_buffer(&mut self, screen_model: &mut ScreenModel) -> Result<()> {
        let mut y = screen_model.cursor.y;
        let mut buffer = mem::take(&mut self.buffer);
        let mut line_remaining = (screen_model.area.width - screen_model.cursor.x) as usize;
        if self.lines.is_empty() {
            self.lines.push_back(Vec::new());
        }

        let line_from = self.lines.len() - 1;
        let mut chunk_from = self.lines.back().unwrap().len();

        for mut chunk in buffer.drain(..) {
            while let Some(split) = chunk.split(line_remaining) {
                self.lines.back_mut().unwrap().push(chunk);
                self.lines.push_back(Vec::new());
                line_remaining = screen_model.area.width as usize;
                chunk = split;
                y += 1;
            }
            self.lines.back_mut().unwrap().push(chunk);
        }

        if y < screen_model.area.height {
            let mut stdout = io::stdout();
            let mut first = true;
            screen_model.sync_cursor()?;
            for line in self.lines.iter().skip(line_from) {
                if !first {
                    screen_model.cursor.y += 1;
                    screen_model.cursor.x = 0;
                    screen_model.sync_cursor()?;
                }
                first = false;
                for chunk in &line[chunk_from..] {
                    stdout.write(chunk.value.as_bytes());
                }
                chunk_from = 0;
            }
            screen_model.cursor.x = self.last_line_width();
            stdout.flush()?;
        } else {
            self.redraw(screen_model);
        }
        Ok(())
    }

    fn redraw(&mut self, screen_model: &mut ScreenModel) -> Result<()> {
        let mut stdout = io::stdout();

        screen_model.cursor.x = 0;
        screen_model.cursor.y = 0;
        screen_model.sync_cursor()?;
        let mut first = true;
        let lines = if self.lines.len() > screen_model.area.height as usize {
            self.lines
                .range(self.lines.len() - screen_model.area.height as usize..)
        } else {
            self.lines.iter()
        };
        for line in lines {
            let mut line_consumed = 0;
            if !first {
                screen_model.cursor.x = 0;
                screen_model.cursor.y += 1;
                screen_model.sync_cursor()?;
            }
            first = false;
            for chunk in line {
                line_consumed += chunk.value.width();
                stdout.write(chunk.value.as_bytes())?;
            }
            for c in 0..(screen_model.area.width as usize - line_consumed) {
                stdout.write(b" ")?;
            }
        }
        stdout.flush()?;
        screen_model.cursor.x = self.last_line_width();
        screen_model.sync_cursor()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TextGrid {
    lines: Vec<Vec<Chunk>>,
    screen_model: ScreenModel,
}

impl TextGrid {
    fn new(screen_model: ScreenModel) -> Self {
        Self {
            screen_model,
            lines: Vec::with_capacity(screen_model.area.height as usize),
        }
    }

    fn cursor_to(&mut self, x: u16, y: u16) {
        if x >= self.screen_model.area.width {
            self.screen_model.cursor.x = self.screen_model.area.width.saturating_sub(1);
        } else {
            self.screen_model.cursor.x = x;
        }
        if y >= self.screen_model.area.height {
            self.screen_model.cursor.y  = self.screen_model.area.height.saturating_sub(1);
        } else {
            self.screen_model.cursor.y = y;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    value: String,
    style: Style,
}

impl Chunk {
    /// Truncate the chunk at the provided width (based on unicode character width, not index), and
    /// returns the trailing chunk, if there is more text after the split point, or None otherwise.
    fn split(&mut self, at: usize) -> Option<Self> {
        let mut width = 0;

        for (mut i, c) in self.value.char_indices() {
            if c == '\n' {
                let other = Self {
                    value: self.value[i + 1..].to_owned(),
                    style: self.style,
                };
                self.value.truncate(i);
                return Some(other);
            }

            width += c.width().unwrap_or(0);
            if width > at {
                let other = Self {
                    value: self.value[i..].to_owned(),
                    style: self.style,
                };
                self.value.truncate(i);
                return Some(other);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub enum WindowNode {
    Window {
        window: Window,
        parent: Option<usize>,
    },
    PairWindow {
        direction: Direction,
        parent: Option<usize>,
        child_left: usize,
        child_right: usize,
        constraint: Constraint,
        area: Rectangle,
    },
}

impl WindowNode {
    fn set_parent(&mut self, new_parent: Option<usize>) {
        match self {
            Self::Window { ref mut parent, .. } => *parent = new_parent,
            Self::PairWindow { ref mut parent, .. } => *parent = new_parent,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WindowManager {
    items: Vec<Option<WindowNode>>,
    active_window: usize,
    root_window: usize,
    active: bool,
}

impl WindowManager {
    pub fn new() -> WindowManager {
        Self::default()
    }

    pub fn init(&mut self) -> Result<()> {
        self.active = true;
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let mut stdout = io::stdout();
        disable_raw_mode()?;
        execute!(stdout, Clear(ClearType::All), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn split(
        &mut self,
        node: usize,
        direction: Direction,
        constraint: Constraint,
        kind: WindowKind,
    ) -> Result<usize> {
        if self.items.is_empty() {
            let window = Window {
                kind,
                active_style: Style::default(),
                screen_model: ScreenModel {
                    area: Self::available_space(),
                    cursor: Cursor::default(),
                },
            };
            self.items.push(Some(WindowNode::Window {
                window,
                parent: None,
            }));
            return Ok(0);
        }

        let (parent, area) = match &self.items[node] {
            Some(WindowNode::PairWindow { .. }) => {
                panic!("Can't split a window that's already split!");
            }
            None => {
                panic!("No such window!");
            }
            Some(WindowNode::Window { parent, window, .. }) => {
                (parent.clone(), window.screen_model.area.clone())
            }
        };

        let new_window = Window {
            kind,
            active_style: Style::default(),
            screen_model: ScreenModel {
                area,
                cursor: Cursor::default(),
            },
        };

        let new_window_id = self.insert_node(WindowNode::Window {
            window: new_window,
            parent: None,
        });

        let split_node = WindowNode::PairWindow {
            direction,
            constraint,
            parent,
            child_left: node,
            child_right: new_window_id,
            area,
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

        if node == self.root_window {
            self.root_window = split_node_id;
        }
        self.reflow()?;
        Ok(new_window_id)
    }

    pub fn redraw_all(&mut self) -> Result<()> {
        for node in self.items.iter_mut() {
            if let Some(WindowNode::Window { window, .. }) = node {
                window.redraw()?;
            }
        }
        Ok(())
    }

    pub fn print(&mut self, text: &str, immediate: bool) -> Result<()> {
        match &mut self.items[self.active_window] {
            Some(WindowNode::Window { window, .. }) => {
                warn!("PWINDOW {} = {:?}", self.active_window, text);
                window.print(text, immediate)?;
            }
            _ => panic!(),
        }
        Ok(())
    }

    pub fn print_char(&mut self, text: char, immediate: bool) -> Result<()> {
        match &mut self.items[self.active_window] {
            Some(WindowNode::Window { window, .. }) => {
                warn!("CPWINDOW {} = {:?}", self.active_window, text);
                window.print_char(text, immediate)?;
            }
            _ => panic!(),
        }
        Ok(())
    }

    pub fn flush_buffer(&mut self) -> Result<()> {
        for node in self.items.iter_mut() {
            if let Some(WindowNode::Window { window, .. }) = node {
                window.flush_buffer()?;
            }
        }
        match &mut self.items[self.active_window] {
            Some(WindowNode::Window { window, .. }) => {
                window.screen_model.sync_cursor();
            }
            _ => panic!(),
        }
        Ok(())
    }

    pub fn set_active(&mut self, active: usize) -> Result<()> {
        self.active_window = active;
        match &mut self.items[self.active_window] {
            Some(WindowNode::Window { window, .. }) => {
                window.screen_model.sync_cursor();
            }
            _ => panic!(),
        }
        Ok(())
    }

    pub fn backspace(&mut self) -> Result<()> {
        match &mut self.items[self.active_window] {
            Some(WindowNode::Window { window, .. }) => {
                window.backspace()?;
            }
            _ => panic!(),
        }
        Ok(())
    }

    fn reflow(&mut self) -> Result<()> {
        let rect = Self::available_space();
        if rect.width == 0 || rect.height == 0 {
            return Ok(());
        }
        self.reflow_window(self.root_window, rect);
        self.redraw_all()?;
        Ok(())
    }

    fn available_space() -> Rectangle {
        let (width, height) = term_size().unwrap();
        Rectangle {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    pub fn size(&self) -> (u16, u16) {
        let space = Self::available_space();
        (space.width, space.height)
    }

    fn reflow_window(&mut self, id: usize, rect: Rectangle) {
        match &mut self.items[id] {
            Some(WindowNode::Window { window, .. }) => {
                window.screen_model.area = rect;
            }
            Some(WindowNode::PairWindow {
                area,
                child_left,
                child_right,
                direction,
                constraint,
                ..
            }) => {
                *area = rect;
                match constraint {
                    Constraint::RightFixed(size) => {
                        let child_left = child_left.clone();
                        let child_right = child_right.clone();
                        let (sized, unconstrained) = rect.split_fixed(*direction, *size);
                        self.reflow_window(child_left, unconstrained);
                        self.reflow_window(child_right, sized);
                    }
                }
            }
            None => unreachable!(),
        }
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

    pub fn child_left(&self, parent: usize) -> Option<(usize, &WindowNode)> {
        match self.items[parent] {
            None => panic!("Cannot get child of non-existant window"),
            Some(WindowNode::Window { .. }) => None,
            Some(WindowNode::PairWindow { child_left, .. }) => {
                Some((child_left, &self.items[child_left].as_ref().unwrap()))
            }
        }
    }

    pub fn child_right<'a>(&'a self, parent: usize) -> Option<(usize, &'a WindowNode)> {
        match self.items[parent] {
            None => panic!("Cannot get child of non-existant window"),
            Some(WindowNode::Window { .. }) => None,
            Some(WindowNode::PairWindow { child_right, .. }) => {
                Some((child_right, &self.items[child_right].as_ref().unwrap()))
            }
        }
    }

    pub fn parent<'a>(&'a self, child: usize) -> Option<(usize, &'a WindowNode)> {
        match self.items[child] {
            None => panic!("Cannot get child of non-existant window"),
            Some(WindowNode::Window { parent, .. } | WindowNode::PairWindow { parent, .. }) => {
                parent.map(|p| (p, self.items[p].as_ref().unwrap()))
            }
        }
    }
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
