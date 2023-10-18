use std::io::Write;
use crossterm::event::{read, Event, KeyEvent, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::{QueueableCommand, cursor};
use crossterm::style::{Print, SetForegroundColor, SetBackgroundColor};
use crossterm::terminal::{Clear, ClearType};

use crate::config::ColorScheme;

pub enum MessageBoxType {
    Error,
    Informative,
    Confirmation
}

pub enum MessageBoxResponse {
    Yes,
    No,
    Cancel
}

pub struct MessageBox {
    x: u16,
    y: u16,
    w: u16
}

impl MessageBox {

    pub fn new(x: u16, y: u16, w: u16) -> MessageBox {
        MessageBox{ x, y, w }
    }

    pub fn show(&self, stdout: &mut std::io::Stdout, text: &str, msgbox_type: MessageBoxType, color_scheme: &ColorScheme) -> MessageBoxResponse {

        stdout.queue(SetForegroundColor(color_scheme.fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(color_scheme.bg_color)).unwrap();
        stdout.queue(cursor::MoveTo(self.x, self.y)).unwrap();
        stdout.queue(Print("-".repeat(self.w as usize))).unwrap();

        //set color
        let (fg_color, bg_color) = match msgbox_type {
            MessageBoxType::Error => (color_scheme.error_fg_color, color_scheme.error_bg_color),
            _ => (color_scheme.fg_color, color_scheme.bg_color),
        };

        stdout.queue(SetForegroundColor(fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(bg_color)).unwrap();
        stdout.queue(cursor::MoveTo(self.x, self.y+1)).unwrap();
        stdout.queue(Print(Clear(ClearType::CurrentLine))).unwrap();
        stdout.queue(Print(text)).unwrap();

        if let MessageBoxType::Confirmation = msgbox_type {
            stdout.queue(Print(" [y/n/c]:")).unwrap();
        }
        stdout.flush().unwrap();

        //wait for user input. Y/N/C are required only for confirmation type. Others types return cancel on almost any event.
        loop {
            let event = read().unwrap();

            if let MessageBoxType::Confirmation = msgbox_type {
                if let Event::Key(key_event) = event {
                    match key_event {
                        KeyEvent{ code: KeyCode::Char('y'), kind: KeyEventKind::Press, .. } => return MessageBoxResponse::Yes,
                        KeyEvent{ code: KeyCode::Char('Y'), kind: KeyEventKind::Press, .. } => return MessageBoxResponse::Yes,
                        KeyEvent{ code: KeyCode::Char('n'), kind: KeyEventKind::Press, .. } => return MessageBoxResponse::No,
                        KeyEvent{ code: KeyCode::Char('N'), kind: KeyEventKind::Press, .. } => return MessageBoxResponse::No,
                        KeyEvent{ code: KeyCode::Char('c'), kind: KeyEventKind::Press, .. } => return MessageBoxResponse::Cancel,
                        KeyEvent{ code: KeyCode::Char('C'), kind: KeyEventKind::Press, .. } => return MessageBoxResponse::Cancel,
                        KeyEvent{ code: KeyCode::Esc, kind: KeyEventKind::Press, .. } => return MessageBoxResponse::Cancel,
                        _ => (),
                    }
                }
            } else {
                match event {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        return MessageBoxResponse::Cancel;
                    },
                    Event::Mouse(mouse_event) if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) => {
                        return MessageBoxResponse::Cancel;
                    },
                    _ => (),
                }
            }
        }
    }
}
