use std::fmt::Display;
use std::io::{self, Stdout, Write};
use std::iter::{from_fn, Iterator};

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
use itertools::unfold;
use log::warn;

use crate::game::InputCode;
use crate::helper::split_exhaustive;
use crate::ui::interface::Interface;
use crate::ui::Screen;
use crate::ui::TextStyle;

struct TextBlob {
    text: String,
    style: TextStyle,
    break_points: Vec<BreakPoint>,
}

#[derive(Clone)]
struct BreakPoint {
    byte_index: usize,
    include_char: bool,
}

impl TextBlob {
    fn len(&self) -> usize {
        self.text.len()
    }

    fn from_string(text: &str, style: TextStyle) -> Vec<TextBlob> {
        split_exhaustive(&text, '\n')
            .map(|v| TextBlob {
                text: v.to_owned(),
                style: style.clone(),
                break_points: Vec::new(),
            })
            .collect()
    }
}

fn wrap_blobs(blobs: &mut [TextBlob], width: usize, mut offset: usize) {
    let mut last_possible_breakpoint: Option<(usize, BreakPoint)> = None;
    for blob in blobs.iter_mut() {
        blob.break_points.clear();
    }
    for i in 0..blobs.len() {
        let blob = &blobs[i];
        if blob.text == "\n" {
            offset = 0;
            last_possible_breakpoint = None;
            continue;
        }
        let break_points = blob
            .text
            .match_indices(' ')
            .map(|x| x.0)
            .collect::<Vec<_>>();
        warn!("{:?}", &break_points);
        let text_len = blob.text.chars().count();
        if text_len + offset <= width {
            offset += text_len;
        } else {
            if let Some((blob_index, breakpoint)) = &last_possible_breakpoint {
                let mut len = blobs[*blob_index].text[breakpoint.byte_index..]
                    .chars()
                    .count()
                    + blobs[*blob_index..i]
                        .iter()
                        .fold(0, |acc, cur| acc + cur.text.chars().count());
                if let Some(i) = break_points.last() {
                    len += blob.text[*i + 1..].chars().count();
                }
                warn!("Len is {}", len);
                if len <= width {
                    blobs[*blob_index].break_points.push(breakpoint.clone());
                    offset = len;
                }
            }
        }
        if let Some(byte_index) = break_points.last() {
            last_possible_breakpoint = Some((
                i,
                BreakPoint {
                    byte_index: *byte_index,
                    include_char: false,
                },
            ));
        }
    }
}

struct Point {
    x: usize,
    y: usize,
}

/// A traditional terminal-based user interface.
pub struct TerminalInterface {
    text_style: TextStyle,
    active_screen: Screen,
    screen_buffer: Vec<TextBlob>,
    old_cursor_pos: Point,
}

impl TerminalInterface {
    pub fn new() -> Result<TerminalInterface> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, MoveTo(0, 0))?;
        enable_raw_mode()?;
        Ok(TerminalInterface {
            screen_buffer: Vec::new(),
            active_screen: Screen::Lower,
            text_style: TextStyle::default(),
            old_cursor_pos: Point { x: 0, y: 0 },
        })
    }

    fn str_to_blobs(&mut self, text: &str) -> Vec<TextBlob> {
        let mut blobs = TextBlob::from_string(text, self.text_style.clone());
        wrap_blobs(
            &mut blobs,
            term_size().unwrap().0 as usize,
            cursor_pos().unwrap().0 as usize,
        );
        blobs
    }

    fn store_blobs(&mut self, blobs: &mut Vec<TextBlob>) {
        self.screen_buffer.append(blobs);
    }

    fn backspace_screenbuffer(&mut self) {
        if let Some(c) = self.screen_buffer.last_mut() {
            if c.text.len() > 1 {
                c.text.pop();
            } else {
                self.screen_buffer.pop();
            }
        }
    }

    fn print_blobs(&mut self, blobs: &[TextBlob]) -> Result<()> {
        for blob in blobs.iter() {
            self.print_blob(blob)?;
        }
        Ok(())
    }

    fn reflow_screen(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, MoveTo(0, 0));
        wrap_blobs(&mut self.screen_buffer, term_size().unwrap().0 as usize, 0);
        for blob in self.screen_buffer.iter() {
            self.print_blob(blob)?;
        }
        Ok(())
    }

    // Here be dragons.
    fn print_blob(&self, blob: &TextBlob) -> Result<()> {
        let mut stdout = io::stdout();
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
        stdout.flush()?;
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
        let mut blobs = self.str_to_blobs(text);
        self.print_blobs(&blobs)?;
        self.store_blobs(&mut blobs);
        Ok(())
    }

    fn print_char(&mut self, text: char) -> Result<()> {
        self.print(&text.to_string())
    }

    fn clear(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        queue!(stdout, Clear(ClearType::All))?;
        stdout.flush()?;
        Ok(())
    }

    fn read_char(&mut self) -> Result<InputCode> {
        loop {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => return Ok(InputCode::Newline),
                    KeyCode::Char(c) => return Ok(InputCode::Character(c)),
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
        let mut line = String::new();
        let mut stdout = io::stdout();
        loop {
            match event::read()? {
                Event::Resize(..) => {
                    self.clear()?;
                    self.reflow_screen()?;
                    //self.reflow_screen();
                }
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => {
                        //self.write(&"\n")?;
                        break;
                    }
                    KeyCode::Esc => {
                        panic!("Yes");
                    }
                    KeyCode::Char(c) => {
                        if line.len() < max_chars {
                            //self.write_screenbuffer(&c.to_string());
                            //self.write(c)?;
                            //self.stdout.flush()?;
                            line.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if !line.is_empty() {
                            queue!(stdout, MoveLeft(1), Print(" "), MoveLeft(1))?;
                            self.backspace_screenbuffer();
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
