use std::io;

use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use tracing::warn;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::game::Result;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SplitDirection {
    Above,
    Below,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SplitSize {
    Fixed(u16),
    Unlimited,
}

impl SplitSize {
    fn as_constraint(&self) -> Constraint {
        match self {
            Self::Fixed(size) => Constraint::Length(*size),
            Self::Unlimited => Constraint::Min(100),
        }
    }
}

#[derive(Clone, Debug)]
struct Chunk {
    text: String,
    style: Style,
}

#[derive(Clone, Debug, Default)]
pub struct TextStream {
    lines: Vec<Vec<Chunk>>,
}

impl TextStream {
    pub fn print(&mut self, text: &str, style: Style) {
        let lines: Vec<_> = text.split('\n').collect();
        for (i, mut line) in lines.iter().enumerate() {
            if i != 0 || self.lines.is_empty() {
                self.lines.push(Vec::new());
            }
            let last_line = self.lines.last_mut().unwrap();
            let matching_chunk = last_line.last_mut().filter(|x| x.style == style);
            match matching_chunk {
                None => {
                    last_line.push(Chunk {
                        text: line.to_string(),
                        style,
                    });
                }
                Some(x) => {
                    x.text.push_str(line);
                }
            }
        }
    }

    pub fn print_char(&mut self, text: char, style: Style) {
        if text == '\n' {
            self.lines.push(vec![Chunk {
                text: "".to_owned(),
                style,
            }]);
        } else {
            if self.lines.is_empty() {
                self.lines.push(Vec::new());
            }
            let last_line = self.lines.last_mut().unwrap();
            let matching_chunk = last_line.last_mut().filter(|x| x.style == style);
            match matching_chunk {
                None => {
                    last_line.push(Chunk {
                        text: text.to_string(),
                        style,
                    });
                }
                Some(x) => {
                    x.text.push(text);
                }
            }
        }
    }

    pub fn backspace(&mut self) {
        for line in self.lines.iter_mut().rev() {
            while let Some(chunk) = line.last_mut() {
                if chunk.text.is_empty() {
                    line.pop();
                } else {
                    chunk.text.pop();
                    return;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum WindowKind {
    TextStream(TextStream),
    TextGrid,
}

#[derive(Clone, Debug)]
struct Window {
    kind: WindowKind,
    active_style: Style,
}

impl Window {
    pub fn render(&mut self, frame: &mut Frame<CrosstermBackend<io::Stdout>>, rect: Rect) {
        match &self.kind {
            WindowKind::TextStream(text) => {
                let lines: Vec<_> = text
                    .lines
                    .iter()
                    .map(|line| {
                        Spans::from(
                            line.iter()
                                .map(|x| Span::from(x.text.as_ref()))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect();
                let text = Text::from(lines);
                let para = Paragraph::new(text).wrap(Wrap { trim: false });
                frame.render_widget(para, rect);
            }
            _ => {
                todo!();
            }
        }
    }

    pub fn print(&mut self, text: &str) {
        match &mut self.kind {
            WindowKind::TextStream(stream) => {
                stream.print(text, self.active_style);
            }
            _ => todo!(),
        }
    }

    pub fn print_char(&mut self, text: char) {
        match &mut self.kind {
            WindowKind::TextStream(stream) => {
                stream.print_char(text, self.active_style);
            }
            _ => todo!(),
        }
    }

    pub fn backspace(&mut self) {
        match &mut self.kind {
            WindowKind::TextStream(stream) => {
                stream.backspace();
            }
            _ => todo!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum WindowNode {
    Window {
        window: Window,
        parent: Option<usize>,
        size: SplitSize,
    },
    PairWindow {
        direction: SplitDirection,
        parent: Option<usize>,
        child_left: usize,
        child_right: usize,
        size: SplitSize,
    },
}

impl WindowNode {
    fn set_parent(&mut self, new_parent: Option<usize>) {
        match self {
            Self::Window { ref mut parent, .. } => *parent = new_parent,
            Self::PairWindow { ref mut parent, .. } => *parent = new_parent,
        }
    }
    fn size(&self) -> SplitSize {
        match self {
            Self::PairWindow { size, .. } => *size,
            Self::Window { size, .. } => *size,
        }
    }
}

pub struct WindowManager {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    items: Vec<Option<WindowNode>>,
    active_window: usize,
    root_window: usize,
    active: bool,
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

impl WindowManager {
    pub fn new() -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Some(Terminal::new(backend)?);

        Ok(WindowManager {
            terminal,
            items: Vec::default(),
            active_window: 0,
            root_window: 0,
            active: false,
        })
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

    pub fn set_active(&mut self, id: usize) {
        self.active_window = id;
    }

    pub fn print(&mut self, text: &str) {
        match &mut self.items[self.active_window] {
            Some(WindowNode::Window { window, .. }) => {
                window.print(text);
            }
            _ => todo!(), // TODO
        }
    }

    pub fn print_char(&mut self, text: char) {
        match &mut self.items[self.active_window] {
            Some(WindowNode::Window { window, .. }) => {
                window.print_char(text);
            }
            _ => todo!(), // TODO
        }
    }

    pub fn backspace(&mut self) {
        match &mut self.items[self.active_window] {
            Some(WindowNode::Window { window, .. }) => {
                window.backspace();
            }
            _ => todo!(), // TODO
        }
    }

    pub fn size(&self) -> (u16, u16) {
        let result = self.terminal.as_ref().unwrap().size().unwrap();
        (result.width, result.height)
    }

    pub fn render(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let mut terminal = self.terminal.take().unwrap();
        terminal
            .draw(|frame| {
                self.render_node(self.root_window, frame, frame.size());
            })
            .unwrap();

        self.terminal.replace(terminal);
    }

    pub fn render_node(
        &mut self,
        node: usize,
        frame: &mut Frame<CrosstermBackend<io::Stdout>>,
        rect: Rect,
    ) {
        match &mut self.items[node] {
            None => panic!("Tried to render non-existant node"),
            Some(WindowNode::Window { window, .. }) => {
                window.render(frame, rect);
            }
            Some(WindowNode::PairWindow {
                child_left,
                child_right,
                size,
                direction,
                ..
            }) => {
                let (render_first, render_second) = match direction {
                    SplitDirection::Below | SplitDirection::Right => (*child_left, *child_right),
                    SplitDirection::Above | SplitDirection::Left => (*child_right, *child_left),
                };

                let direction = match direction {
                    SplitDirection::Above | SplitDirection::Below => Direction::Vertical,
                    SplitDirection::Left | SplitDirection::Right => Direction::Horizontal,
                };

                let first_node = self.items[render_first].as_ref().unwrap();
                let second_node = self.items[render_second].as_ref().unwrap();

                let constraints = [
                    first_node.size().as_constraint(),
                    second_node.size().as_constraint(),
                ];
                let layout = Layout::default()
                    .direction(direction)
                    .constraints(constraints.as_ref())
                    .split(rect);
                self.render_node(render_first, frame, layout[0]);
                self.render_node(render_second, frame, layout[1]);
            }
        }
    }

    pub fn split(
        &mut self,
        node: usize,
        direction: SplitDirection,
        size: SplitSize,
        kind: WindowKind,
    ) -> Result<usize> {
        if self.items.is_empty() {
            let window = Window {
                kind,
                active_style: Style::default(),
            };
            self.items.push(Some(WindowNode::Window {
                window,
                parent: None,
                size: SplitSize::Unlimited,
            }));
            return Ok(0);
        }

        let (mut existing, parent, existing_size) = match &mut self.items[node] {
            Some(WindowNode::PairWindow { .. }) => {
                panic!("Can't split a window that's already split!");
            }
            None => {
                panic!("No such window!");
            }
            Some(WindowNode::Window {
                window,
                parent,
                size: size_ref,
            }) => {
                let existing_size = size_ref.to_owned();
                *size_ref = SplitSize::Unlimited;
                (window, parent.clone(), existing_size)
            }
        };

        let new_window = Window {
            kind,
            active_style: Style::default(),
        };

        let new_window_id = self.insert_node(WindowNode::Window {
            window: new_window,
            parent: None,
            size,
        });

        let split_node = WindowNode::PairWindow {
            direction,
            parent,
            child_left: node,
            child_right: new_window_id,
            size: existing_size,
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
