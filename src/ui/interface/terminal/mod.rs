mod text_blob;

use std::io::{self, Stdout, Write};

use anyhow::Result;
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

use crate::game::InputCode;
use crate::ui::interface::Interface;
use crate::ui::Screen;
use crate::ui::TextStyle;

use text_blob::{wrap_blobs, TextBlob};

struct Point {
    x: usize,
    y: usize,
}

/// A traditional terminal-based user interface.
pub struct TerminalInterface {
    text_style: TextStyle,
    active_screen: Screen,
    history: Vec<TextBlob>,
    buffer_point: usize,
    old_cursor_pos: Point,
}

impl TerminalInterface {
    pub fn new() -> Result<TerminalInterface> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, MoveTo(0, 0))?;
        enable_raw_mode()?;
        Ok(TerminalInterface {
            history: Vec::new(),
            active_screen: Screen::Lower,
            text_style: TextStyle::default(),
            old_cursor_pos: Point { x: 0, y: 0 },
            buffer_point: 0,
        })
    }

    fn str_to_blobs(&mut self, text: &str) -> Vec<TextBlob> {
        TextBlob::from_string(text, self.text_style.clone())
    }

    fn print_blobs(&mut self, blobs: &mut Vec<TextBlob>) -> Result<()> {
        self.history.append(blobs);
        Ok(())
    }

    fn backspace(&mut self) {
        if let Some(c) = self.history.last_mut() {
            if c.text.len() > 1 {
                c.text.pop();
            } else {
                self.history.pop();
            }
        }
    }

    fn flush_buffer(&mut self) -> Result<()> {
        wrap_blobs(
            &mut self.history[self.buffer_point..],
            term_size().unwrap().0 as usize,
            cursor_pos().unwrap().0 as usize,
        );
        if self.buffer_point >= self.history.len() {
            return Ok(());
        }
        let mut stdout = io::stdout();
        for blob in self.history[self.buffer_point..].iter() {
            self.print_blob(blob, &mut stdout)?;
        }
        stdout.flush()?;
        self.buffer_point = self.history.len();
        Ok(())
    }

    fn reflow_screen(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        wrap_blobs(&mut self.history, term_size().unwrap().0 as usize, 0);
        for blob in self.history.iter() {
            self.print_blob(blob, &mut stdout)?;
        }
        stdout.flush()?;
        Ok(())
    }

    fn print_bufferable(&mut self, text: &str, immediate: bool) -> Result<()> {
        let mut blobs = self.str_to_blobs(text);
        self.print_blobs(&mut blobs)?;
        if immediate {
            self.flush_buffer();
        }
        Ok(())
    }

    fn print_blob(&self, blob: &TextBlob, stdout: &mut Stdout) -> Result<()> {
        if blob.style.bold {
            queue!(stdout, SetAttribute(Attribute::Bold))?;
        }
        if blob.style.emphasis {
            queue!(stdout, SetAttribute(Attribute::Underlined))?;
        }
        if blob.style.reverse_video {
            queue!(stdout, SetAttribute(Attribute::Reverse))?;
        }
        if blob.break_points.is_empty() {
            queue!(
                stdout,
                Print(format!("{}", blob.text).replace("\n", "\n\r")),
                SetAttribute(Attribute::Reset)
            )?;
        } else {
            queue!(stdout, Print(&blob.text[..blob.break_points[0].byte_index]),)?;
            for i in 1..blob.break_points.len() {
                queue!(
                    stdout,
                    Print("\n\r"),
                    Print(
                        &blob.text[blob.break_points[i - 1].byte_index + 1
                            ..blob.break_points[i].byte_index]
                    ),
                )?;
            }
            queue!(
                stdout,
                Print("\n\r"),
                Print(&blob.text[blob.break_points[blob.break_points.len() - 1].byte_index + 1..]),
            )?;
        }
        queue!(stdout, SetAttribute(Attribute::Reset))?;
        Ok(())
    }
}

impl Drop for TerminalInterface {
    /// Restore the terminal to its previous state when exiting.
    fn drop(&mut self) {
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}

impl Interface for TerminalInterface {
    fn print(&mut self, text: &str) -> Result<()> {
        self.print_bufferable(text, false)
    }

    fn print_char(&mut self, text: char) -> Result<()> {
        self.print(&text.to_string())
    }

    fn clear(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        queue!(stdout, Clear(ClearType::All))?;
        self.history.clear();
        self.buffer_point = 0;
        stdout.flush()?;
        Ok(())
    }

    fn read_char(&mut self) -> Result<InputCode> {
        self.flush_buffer()?;
        loop {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => return Ok(InputCode::Newline),
                    KeyCode::Char(c) => {
                        self.print_bufferable(&c.to_string(), true)?;
                        return Ok(InputCode::Character(c));
                    }
                    KeyCode::Up => return Ok(InputCode::CursorUp),
                    KeyCode::Down => return Ok(InputCode::CursorDown),
                    KeyCode::Left => return Ok(InputCode::CursorLeft),
                    KeyCode::Right => return Ok(InputCode::CursorRight),
                    KeyCode::Backspace | KeyCode::Delete => return Ok(InputCode::Delete),
                    KeyCode::Esc => return Ok(InputCode::Escape),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn read_line(&mut self, max_chars: usize) -> Result<String> {
        self.flush_buffer()?;
        let mut line = String::new();
        let mut stdout = io::stdout();
        loop {
            match event::read()? {
                Event::Resize(..) => {
                    self.reflow_screen()?;
                }
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => {
                        self.print_bufferable(&"\n", true)?;
                        break;
                    }
                    KeyCode::Esc => {
                        panic!("Yes");
                    }
                    KeyCode::Char(c) => {
                        if line.len() < max_chars {
                            self.print_bufferable(&c.to_string(), true)?;
                            line.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if !line.is_empty() {
                            queue!(stdout, MoveLeft(1), Print(" "), MoveLeft(1))?;
                            self.backspace();
                            //self.stdout.flush()?;
                            line.pop();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(line)
    }

    fn done(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        queue!(stdout, Print("\n\r[Hit any key to exit...]"))?;
        stdout.flush()?;
        read()?;
        Ok(())
    }

    fn text_style_bold(&mut self) {
        self.text_style.bold = true;
    }

    fn text_style_emphasis(&mut self) {
        self.text_style.emphasis = true;
    }

    fn text_style_reverse(&mut self) {
        self.text_style.reverse_video = true;
    }

    fn text_style_fixed(&mut self) {
        self.text_style.fixed_width = true;
    }

    fn text_style_clear(&mut self) {
        self.text_style.bold = false;
        self.text_style.emphasis = false;
        self.text_style.reverse_video = false;
        self.text_style.fixed_width = false;
    }

    fn quit(&mut self) {}
}
