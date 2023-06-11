use std::io::Write;
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::{QueueableCommand, cursor};
use crossterm::style::{Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};

use crate::config::ColorScheme;
use crate::commands::COMMAND_LIST;

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

        let info_text_len = info_text.len();
        let mut cursor_pos = info_text_len;
        let mut cursor_in_string = 0;

        loop {

            //draw flush and wait for user input
            self.draw(stdout, info_text, &user_string, cursor_pos as u16, color_scheme);
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

                    //delete character left to the cursor
                    KeyEvent{ code: KeyCode::Backspace, .. } => {
                        if cursor_in_string > 0 {
                            cursor_in_string -= 1;
                            cursor_pos -= 1;
                            user_string.remove(cursor_in_string);
                        }
                    },

                    //delete character at the cursor position
                    KeyEvent{ code: KeyCode::Delete, .. } => {
                        if cursor_in_string < user_string.len() {
                            user_string.remove(cursor_in_string);
                        }
                    },

                    //move cursor to the start of the string
                    KeyEvent{ code: KeyCode::Home, .. } => {
                        cursor_in_string = 0;
                        cursor_pos = info_text_len;
                    },

                    //move cursor to the end of the string
                    KeyEvent{ code: KeyCode::End, .. } => {
                        cursor_in_string = user_string.len();
                        cursor_pos = info_text_len + cursor_in_string;
                    },

                    //get previous user input from history
                    KeyEvent{ code: KeyCode::Up, .. } => {
                        history_idx = history_idx.saturating_sub(1);
                        user_string = user_input_history.get(history_idx).unwrap_or(&"".to_string()).to_owned();

                        cursor_in_string = user_string.len();
                        cursor_pos = cursor_in_string + info_text_len;
                    },

                    //get next user input from history
                    KeyEvent{ code: KeyCode::Down, .. } => {
                        if history_idx <= user_input_history.len() {
                            history_idx += 1;
                        }
                        user_string = user_input_history.get(history_idx).unwrap_or(&"".to_string()).to_owned();

                        cursor_in_string = user_string.len();
                        cursor_pos = cursor_in_string + info_text_len;
                    },

                    //Move cursor to the left
                    KeyEvent{ code: KeyCode::Left, .. } => {
                        if cursor_in_string > 0 {
                            cursor_in_string -= 1;
                            cursor_pos -= 1;
                        }
                    },

                    //move cursor to the right
                    KeyEvent{ code: KeyCode::Right, .. } => {
                        if cursor_in_string < user_string.len() {
                            cursor_in_string += 1;
                            cursor_pos += 1;
                        }
                    },

                    //command auto completion code
                    KeyEvent{ code: KeyCode::Tab, .. } => {
                        let mut common_commands: Vec<&str> = COMMAND_LIST.into_iter().filter(|s| s.starts_with(&user_string)).collect();

                        if !common_commands.is_empty() {

                            //if there is only one command, set it and ends it with space
                            if common_commands.len() == 1 {
                                user_string = common_commands.first().unwrap().to_string();
                                user_string.push(' ');

                            //if there is a multiple results, set the max common length
                            } else {
                                let ref_string = common_commands.pop().unwrap();
                                let mut common_length = user_string.len();

                                while common_length < ref_string.len() {
                                    if !common_commands.iter().all(|command| command.starts_with(&ref_string[..common_length])) {
                                        break;
                                    }
                                    common_length += 1;
                                }
                                user_string = ref_string[..common_length.saturating_sub(1)].to_string();
                            }

                            cursor_in_string = user_string.len();
                            cursor_pos = cursor_in_string + info_text_len;
                        }
                    },

                    //Any printable character push into user input string
                    KeyEvent{ code: KeyCode::Char(c), .. } if (' '..='~').contains(&c) => {
                        user_string.insert(cursor_pos - info_text_len, c);
                        if cursor_in_string < user_string.len() {
                            cursor_in_string += 1;
                            cursor_pos += 1;
                        }
                    },
                    _ => (),
                }
            }
        }

        stdout.queue(cursor::Hide).unwrap();
        user_string
    }


    fn draw(&self, stdout: &mut std::io::Stdout, info_text: &str, user_string: &str, cursor_pos: u16, color_scheme: &ColorScheme) {

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
        stdout.queue(crossterm::cursor::MoveToColumn(cursor_pos)).unwrap();
    }
}
