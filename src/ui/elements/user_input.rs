use std::io::Write;
use crossterm::event::{read, Event, KeyEvent, KeyCode, KeyEventKind};
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
        let mut command_list = Vec::<&str>::new();

        loop {

            //draw flush and wait for user input
            self.draw(stdout, info_text, &user_string, cursor_pos as u16, &command_list, color_scheme);
            stdout.flush().unwrap();

            let event = read().unwrap();

            //process only keyboard events
            if let Event::Key(key_event) = event {
                match key_event {

                    //ESC to cancel user input and return empty string
                    KeyEvent{ code: KeyCode::Esc, kind: KeyEventKind::Press, .. } => { user_string.clear(); break; },

                    //Enter to confirm user input and store it in command history. If is not the same as the last command
                    KeyEvent{ code: KeyCode::Enter, kind: KeyEventKind::Press, .. } => {
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
                    KeyEvent{ code: KeyCode::Backspace, kind: KeyEventKind::Press, .. } => {
                        if cursor_in_string > 0 {
                            cursor_in_string -= 1;
                            cursor_pos -= 1;
                            user_string.remove(cursor_in_string);
                        }

                        //update list with aviable commands
                        if user_string.is_empty() {
                            command_list.clear();
                        } else {
                            command_list = COMMAND_LIST.into_iter().filter(|s| s.starts_with(&user_string)).collect();
                        }
                    },

                    //delete character at the cursor position
                    KeyEvent{ code: KeyCode::Delete, kind: KeyEventKind::Press, .. } => {
                        if cursor_in_string < user_string.len() {
                            user_string.remove(cursor_in_string);
                        }

                        //update list with aviable commands
                        if user_string.is_empty() {
                            command_list.clear();
                        } else {
                            command_list = COMMAND_LIST.into_iter().filter(|s| s.starts_with(&user_string)).collect();
                        }
                    },

                    //move cursor to the start of the string
                    KeyEvent{ code: KeyCode::Home, kind: KeyEventKind::Press, .. } => {
                        cursor_in_string = 0;
                        cursor_pos = info_text_len;
                    },

                    //move cursor to the end of the string
                    KeyEvent{ code: KeyCode::End, kind: KeyEventKind::Press, .. } => {
                        cursor_in_string = user_string.len();
                        cursor_pos = info_text_len + cursor_in_string;
                    },

                    //get previous user input from history
                    KeyEvent{ code: KeyCode::Up, kind: KeyEventKind::Press, .. } => {
                        history_idx = history_idx.saturating_sub(1);
                        user_string = user_input_history.get(history_idx).unwrap_or(&"".to_string()).to_owned();

                        cursor_in_string = user_string.len();
                        cursor_pos = cursor_in_string + info_text_len;
                        command_list.clear();
                    },

                    //get next user input from history
                    KeyEvent{ code: KeyCode::Down, kind: KeyEventKind::Press, .. } => {
                        if history_idx <= user_input_history.len() {
                            history_idx += 1;
                        }
                        user_string = user_input_history.get(history_idx).unwrap_or(&"".to_string()).to_owned();

                        cursor_in_string = user_string.len();
                        cursor_pos = cursor_in_string + info_text_len;
                        command_list.clear();
                    },

                    //Move cursor to the left
                    KeyEvent{ code: KeyCode::Left, kind: KeyEventKind::Press, .. } => {
                        if cursor_in_string > 0 {
                            cursor_in_string -= 1;
                            cursor_pos -= 1;
                        }
                    },

                    //move cursor to the right
                    KeyEvent{ code: KeyCode::Right, kind: KeyEventKind::Press, .. } => {
                        if cursor_in_string < user_string.len() {
                            cursor_in_string += 1;
                            cursor_pos += 1;
                        }
                    },

                    //command auto completion code
                    KeyEvent{ code: KeyCode::Tab, kind: KeyEventKind::Press, .. } => {
                        command_list = COMMAND_LIST.into_iter().filter(|s| s.starts_with(&user_string)).collect();

                        if !command_list.is_empty() {

                            //if there is only one command, set it and ends it with space
                            if command_list.len() == 1 {
                                user_string = command_list.first().unwrap().to_string();
                                user_string.push(' ');
                                command_list.clear();

                            //if there is a multiple results, find and set max common length
                            } else {
                                let ref_string = command_list.last().unwrap();
                                let mut common_length = user_string.len();

                                while common_length < ref_string.len() {
                                    if !command_list.iter().all(|command| command.starts_with(&ref_string[..common_length])) {
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
                    KeyEvent{ code: KeyCode::Char(c), kind: KeyEventKind::Press, .. } if (' '..='~').contains(&c) => {
                        user_string.insert(cursor_pos - info_text_len, c);
                        if cursor_in_string < user_string.len() {
                            cursor_in_string += 1;
                            cursor_pos += 1;
                        }

                        //update list of aviable commands
                        command_list = COMMAND_LIST.into_iter().filter(|s| s.starts_with(&user_string)).collect();
                    },
                    _ => (),
                }
            }
        }

        stdout.queue(cursor::Hide).unwrap();
        user_string
    }


    fn draw(&self, stdout: &mut std::io::Stdout, info_text: &str, user_string: &str, cursor_pos: u16, command_list: &[&str], color_scheme: &ColorScheme) {

        //draw border line with aviable commands if any
        stdout.queue(SetForegroundColor(color_scheme.fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(color_scheme.bg_color)).unwrap();
        stdout.queue(cursor::MoveTo(self.x, self.y)).unwrap();
        let mut free_space = self.w as usize;

        if !command_list.is_empty() {
            let mut cl_string = "-[ ".to_string();

            for command in command_list.iter() {
                if cl_string.len() + command.len() < (self.w as usize).saturating_sub(7) {
                    cl_string.push_str(command);
                    cl_string.push_str(", ");
                } else {
                    cl_string.push_str(".. ");
                    break;
                }
            }
            cl_string.push_str("]-");
            free_space -= cl_string.len();
            stdout.queue(Print(cl_string)).unwrap();
        }
        stdout.queue(Print("-".repeat(free_space))).unwrap();

        //clear input line and draw strings
        stdout.queue(cursor::MoveTo(self.x, self.y+1)).unwrap();
        stdout.queue(Print(Clear(ClearType::CurrentLine))).unwrap();
        stdout.queue(Print(info_text)).unwrap();
        stdout.queue(Print(user_string)).unwrap();
        stdout.queue(crossterm::cursor::MoveToColumn(cursor_pos)).unwrap();
    }
}
