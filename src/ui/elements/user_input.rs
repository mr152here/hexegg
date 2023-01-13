use std::io::Write;
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::{QueueableCommand, cursor};
use crossterm::style::{Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};

use crate::config::ColorScheme;

pub struct UserInput {
    x: u16,
    y: u16,
    w: u16
}

//draws a simple input line at the bottom of the screen with border line above. 
//process/block keyboard events until user submit or cancel its input
impl UserInput {

    pub fn new(x: u16, y:u16, w:u16) -> UserInput {
        UserInput {x, y, w}
    }

    pub fn input(&self, stdout: &mut std::io::Stdout, info_text: &str, user_input_history: &mut Vec<String>, color_scheme: &ColorScheme) -> String {

        let mut history_idx = user_input_history.len();
        let mut user_string = String::new();
        stdout.queue(cursor::Show).unwrap();

        loop {

            //draw flush and wait for user input
            self.draw(stdout, info_text, &user_string, color_scheme);
            stdout.flush().unwrap();
            let event = read().unwrap();

            //process only keyboard events
            if let Event::Key(key_event) = event {
                match key_event {

                    //ESC to cancel user input and return empty string
                    KeyEvent{ code: KeyCode::Esc, .. } => { user_string.clear(); break; },

                    //Enter to confirm user input and store it in command history. If is not the same as the last command
                    KeyEvent{ code: KeyCode::Enter, .. } => { 
                        user_string = user_string.trim().to_string();

                        if !user_string.is_empty() {
                            match user_input_history.last() {
                                None => user_input_history.push(user_string.clone()),
                                Some(s) if *s != user_string => user_input_history.push(user_string.clone()), 
                                _ => (),
                            }
                        }
                        break;
                    },

                    //Backspace to delete last character from user input
                    KeyEvent{ code: KeyCode::Backspace, .. } => { user_string.pop(); },

                    //Up to get previous user input from history
                    KeyEvent{ code: KeyCode::Up, .. } => {
                        history_idx = history_idx.saturating_sub(1);
                        user_string = user_input_history.get(history_idx).unwrap_or(&"".to_string()).to_owned();
                    },

                    //Down to get next user input from history
                    KeyEvent{ code: KeyCode::Down, .. } => {
                        if history_idx <= user_input_history.len() {
                            history_idx += 1;
                        }
                        user_string = user_input_history.get(history_idx).unwrap_or(&"".to_string()).to_owned();
                    },

                    //Any printable character push into user input
                    KeyEvent{ code: KeyCode::Char(c), .. } if (' '..='~').contains(&c) => { user_string.push(c); },
                    _ => (),
                }
            }
        }

        stdout.queue(cursor::Hide).unwrap();
        user_string
    }


    fn draw(&self, stdout: &mut std::io::Stdout, info_text: &str, user_string: &str, color_scheme: &ColorScheme) {

        //draw border line
        stdout.queue(SetForegroundColor(color_scheme.fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(color_scheme.bg_color)).unwrap();
        stdout.queue(cursor::MoveTo(self.x, self.y)).unwrap();
        stdout.queue(Print("-".repeat(self.w as usize))).unwrap();

        //clear input line and draw strings
        stdout.queue(cursor::MoveTo(self.x, self.y+1)).unwrap();
        stdout.queue(Print(Clear(ClearType::CurrentLine))).unwrap();
        stdout.queue(Print(info_text)).unwrap();
        stdout.queue(Print(user_string)).unwrap();
    }
}
