use std::{env, fs};
use std::io::{Read, Write};
use std::time::Instant;
use crossterm::style::{Color, Print, ResetColor};
use crossterm::terminal::{Clear, ClearType, size};
use crossterm::QueueableCommand;
use crossterm::event::{Event, KeyEvent, KeyCode, read, KeyModifiers};
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture};

mod config;
use config::{Config, ColorScheme, HighlightStyle, ScreenPagingSize};

mod cursor;
use cursor::{Cursor, CursorState};

mod command_functions;
mod commands;
use commands::Command;

mod ui;
use ui::screens::Screen;
use ui::screens::text_screen::TextScreen;
use ui::screens::byte_screen::ByteScreen;
use ui::screens::word_screen::WordScreen;
use ui::elements::user_input::UserInput;
use ui::elements::message_box::*;

mod file_buffer;
use file_buffer::FileBuffer;

mod location_list;
mod signatures;

fn create_screens(cols: u16, rows: u16, config: &Config) -> Vec<Box<dyn Screen>> {
    let mut screens = Vec::<Box<dyn Screen>>::new();

    if let Some(s) = config.screen_settings("text_screen") {
        if s.enabled {
            screens.push(Box::new(TextScreen::new(cols, rows, s)));
        }
    } else {
        println!("Can't found setting for '{}' in 'config.toml' file.", "text_screen");
    }

    if let Some(s) = config.screen_settings("byte_screen") {
        if s.enabled {
            screens.push(Box::new( ByteScreen::new(cols, rows, s)));
        }
    } else {
        println!("Can't found setting for '{}' in 'config.toml' file.", "byte_screen");
    }

    if let Some(s) = config.screen_settings("word_screen") {
        if s.enabled {
            screens.push(Box::new(WordScreen::new(cols, rows, s)));
        }
    } else {
        println!("Can't found setting for '{}' in 'config.toml' file.", "word_screen");
    }

    screens
}

fn generate_highlight_color(rnd_seed: &mut u32, highlight_style: HighlightStyle, color_scheme: &ColorScheme) -> Color {
    match highlight_style {
        HighlightStyle::None => color_scheme.bg_color,
        HighlightStyle::Solid => color_scheme.highlight_bg_color,
        HighlightStyle::RandomDark => {
            let min = 30;
            let len = 130 - min;
            let r = (xor_shift_rng(rnd_seed) % len + min) as u8;
            let g = (xor_shift_rng(rnd_seed) % len + min) as u8;
            let b = (xor_shift_rng(rnd_seed) % len + min) as u8;
            Color::Rgb{r, g, b}
        },
        HighlightStyle::RandomLight => {
            let min = 120;
            let len = 220 - min;
            let r = (xor_shift_rng(rnd_seed) % len + min) as u8;
            let g = (xor_shift_rng(rnd_seed) % len + min) as u8;
            let b = (xor_shift_rng(rnd_seed) % len + min) as u8;
            Color::Rgb{r, g, b}
        },
        HighlightStyle::RandomAnsi => {
            Color::AnsiValue((xor_shift_rng(rnd_seed) & 0xFF) as u8)
        }
    }
}

fn xor_shift_rng(rnd_seed: &mut u32) -> u32 {
    *rnd_seed = *rnd_seed ^ (*rnd_seed << 13);
    *rnd_seed = *rnd_seed ^ (*rnd_seed >> 17);
    *rnd_seed = *rnd_seed ^ (*rnd_seed << 5);
    *rnd_seed
}


fn main() {
    //read config file
    let cfg_string = match fs::read_to_string("config.toml") {
        Ok(s) => s,
        Err(s) => { println!("Reading 'config.toml' error: {}", s); return; },
    };

    //parse toml structure
    let mut config: Config = match toml::from_str(&cfg_string) {
        Ok(c) => c,
        Err(s) => { println!("Parsing 'config.toml' error: {}", s); return; },
    };
    
    //load colorscheme
    let mut color_scheme: ColorScheme = match config.color_scheme(&config.active_color_scheme) {
        Some(cs) => cs.clone(),
        None => { println!("Can't find active color scheme. Please set 'active_color_scheme' in 'config.toml' to correct value"); return; },
    };

    let mut yank_buffer = Vec::<u8>::new();
    let mut file_buffers: Vec<FileBuffer> = Vec::new();
    let mut active_fb_index = 0;

    //parse cmdline and load every file into separate file buffer
    for arg in env::args().skip(1) {
        match command_functions::read_file(&arg) {
            Ok(file_data) => {
                let mut fb = FileBuffer::from_vec(file_data);
                fb.set_filename(&arg);
                file_buffers.push(fb);
            },
            Err(s) => { 
                println!("Can't open file '{}'. {}\nPress enter to continue...", arg, s);
                std::io::stdin().read_exact(&mut [0; 1]).expect("Failed to read stdin!");
            },
        }
    }

    //if there is nothing to open, print message and quit
    if file_buffers.is_empty() {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("Copyright 2023 Michal Kopera\n\
            This software is licensed under the terms of Apache 2.0 license. http://www.apache.org/licenses/LICENSE-2.0\n\
            Is distributed on an \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\n\n\
            hexegg version {}\nusage: hexegg <file1> [file2] [file3] ...\n", VERSION);
        return;
    }

    //init terminal
    let mut stdout = std::io::stdout();
    let (mut cols, mut rows) = size().unwrap();
    let mut cursor = Cursor::new(0, CursorState::Hidden);

    //load and create screens
    let mut screens = create_screens(cols, rows, &config);
    if screens.is_empty() {
        println!("All screens are disabled in 'config.toml' file!");
        return;
    }

    //set up terminal and mouse
    match crossterm::terminal::enable_raw_mode() {
        Ok(_) => {
            stdout.queue(Print(Clear(ClearType::All))).unwrap();
            stdout.queue(crossterm::cursor::Hide).unwrap();
            if config.mouse_enabled {
                stdout.queue(EnableMouseCapture).unwrap();
            }
        },
        Err(s) => { println!("{}", s); return; },
    }

    //find default screen. If not found, load the first one.
    let mut active_screen_index = screens.iter()
                                    .enumerate()
                                    .find_map(|(i,scr)| (scr.screen_name() == config.default_screen).then_some(i))
                                    .unwrap_or(0);

    screens[active_screen_index].draw(&mut stdout, &file_buffers, active_fb_index, &cursor, &color_scheme, &config);
    stdout.flush().unwrap();
   
    let mut random_seed = 0x5EED;
    let mut in_selection_mode = false;
    let mut selection_start = 0;
    let mut cmd_history = Vec::<String>::new();
    let (mut last_mouse_col, mut last_mouse_row) = (0, 0);
    let mut last_click_time = Instant::now();

    //main program loop
    loop {

        //recreate screens if terminal size change
        let (new_cols, new_rows) = size().unwrap();
        if new_cols != cols || new_rows != rows {
            cols = new_cols;
            rows = new_rows;
            screens = create_screens(cols, rows, &config);
        }

        let mut command: Option<Command> = None;
        let event = read().unwrap();

        //process input events
        if let Event::Key(key_event) = event {
            let file_size = file_buffers[active_fb_index].len();
            let file_view_offset = file_buffers[active_fb_index].position();
            let row_size = screens[active_screen_index].row_size() as usize;
            let page_size = screens[active_screen_index].page_size();

            //keyboard handling 
            match key_event {
                KeyEvent{ code: KeyCode::Esc, .. } => {
                    if in_selection_mode {
                        file_buffers[active_fb_index].set_selection(None);
                        in_selection_mode = false;
                        
                    } else if cursor.is_visible() {
                        cursor.set_state(CursorState::Hidden);

                    } else {
                        file_buffers[active_fb_index].set_selection(None);
                        in_selection_mode = false;
                    }
                },
                KeyEvent{ code: KeyCode::Up, modifiers: km, .. } => {
                    let mv = if km == KeyModifiers::SHIFT { 8 * row_size } else { row_size };
                    command = Some(Command::GotoRelative(-(mv as isize)));
                },
                KeyEvent{ code: KeyCode::Down, modifiers: km, .. } => {
                    let mv = if km == KeyModifiers::SHIFT { 8 * row_size } else { row_size };
                    command = Some(Command::GotoRelative(mv as isize));
                },
                KeyEvent{ code: KeyCode::Left, modifiers: km, .. } => { 
                    let mv = if km == KeyModifiers::SHIFT { 8 } else { 1 };
                    command = Some(Command::GotoRelative(-(mv as isize)));
                },
                KeyEvent{ code: KeyCode::Right, modifiers: km, .. } => {
                    let mv = if km == KeyModifiers::SHIFT { 8 } else { 1 };
                    command = Some(Command::GotoRelative(mv as isize));
                },
                KeyEvent{ code: KeyCode::PageUp, .. } => {
                    command = Some(Command::GotoRelative(-(page_size as isize)));
                },
                KeyEvent{ code: KeyCode::PageDown, .. } => {
                    command = Some(Command::GotoRelative( page_size as isize));
                },
                KeyEvent{ code: KeyCode::Home, .. } => {
                    command = Some(Command::Goto(0));
                },
                KeyEvent{ code: KeyCode::End, .. } => {
                    let new_pos = if cursor.is_visible() {
                        if file_size > 0 { file_size - 1 } else { 0 }
                    } else {
                        if file_size < page_size { 0 } else { file_size - page_size }
                    };
                    command = Some(Command::Goto(new_pos));
                },
                KeyEvent{ code: KeyCode::Enter, .. } => {
                    active_screen_index = (active_screen_index + 1) % screens.len();

                    if cursor.is_visible() {
                        command = Some(Command::Goto(cursor.position()));
                    }
                },
                KeyEvent{ code: KeyCode::Tab, .. } => { 
                    active_fb_index = (active_fb_index + 1) % file_buffers.len();
                },
                KeyEvent{ code: KeyCode::Delete, .. } if cursor.is_visible() => {
                    file_buffers[active_fb_index].unpatch_offset(cursor.position());
                    cursor.set_ho_byte_part(true);
                },
                KeyEvent{ code: KeyCode::Backspace, .. } if cursor.is_visible() => {
                    cursor -= 1;
                    file_buffers[active_fb_index].unpatch_offset(cursor.position());
                    command = Some(Command::GotoRelative(0));
                },
                KeyEvent{ code: KeyCode::Char('q'), .. } if !cursor.is_visible() => command = Some(Command::Quit(true)), 
                KeyEvent{ code: KeyCode::Char('h'), .. } if cursor.is_byte() || !cursor.is_visible() => config.highlight_diff = !config.highlight_diff,  
                KeyEvent{ code: KeyCode::Char('i'), .. } if cursor.is_byte() || !cursor.is_visible() => screens[active_screen_index].toggle_info_bar(),
                KeyEvent{ code: KeyCode::Char('l'), .. } if cursor.is_byte() || !cursor.is_visible() => screens[active_screen_index].toggle_location_bar(),
                KeyEvent{ code: KeyCode::Char('o'), .. } if cursor.is_byte() || !cursor.is_visible() => screens[active_screen_index].toggle_offset_bar(),
                KeyEvent{ code: KeyCode::Char('k'), .. } if cursor.is_byte() || !cursor.is_visible() => config.lock_file_buffers = !config.lock_file_buffers,
                KeyEvent{ code: KeyCode::Char('p'), .. } if cursor.is_byte() || !cursor.is_visible() => config.only_printable = ! config.only_printable,
                KeyEvent{ code: KeyCode::Char('.'), .. } if cursor.is_byte() || !cursor.is_visible() => command = Some(Command::FindDiff),
                KeyEvent{ code: KeyCode::Char(','), .. } if cursor.is_byte() || !cursor.is_visible() => command = Some(Command::FindPatch),
                KeyEvent{ code: KeyCode::Char('['), .. } if cursor.is_byte() || !cursor.is_visible() => {
                    if let Some((o,_)) = file_buffers[active_fb_index].location_list_mut().previous() {
                        command = Some(Command::Goto(o)) 
                    } 
                },
                KeyEvent{ code: KeyCode::Char(']'), .. } if cursor.is_byte() || !cursor.is_visible() => {
                    if let Some((o,_)) = file_buffers[active_fb_index].location_list_mut().next() {
                        command = Some(Command::Goto(o)) 
                    }
                },
                KeyEvent{ code: KeyCode::Char('='), .. } if cursor.is_byte() || !cursor.is_visible() => {
                    if let Some((o,_)) = file_buffers[active_fb_index].location_list().current() {
                        command = Some(Command::Goto(o))
                    }
                },
                KeyEvent{ code: KeyCode::Char('r'), .. } if cursor.is_byte() || !cursor.is_visible() => {
                    //remove currently selected locattion and jump to new current
                    file_buffers[active_fb_index].location_list_mut().remove_current_location();
                    if let Some((o,_)) = file_buffers[active_fb_index].location_list().current() {
                        command = Some(Command::Goto(o))
                    }
                },
                KeyEvent{ code: KeyCode::Char('-'), .. } if cursor.is_byte() || !cursor.is_visible() => screens[active_screen_index].dec_row_size(),
                KeyEvent{ code: KeyCode::Char('+'), .. } if cursor.is_byte() || !cursor.is_visible() => screens[active_screen_index].inc_row_size(),
                KeyEvent{ code: KeyCode::Char('/'), .. } if cursor.is_byte() || !cursor.is_visible() => {
                    let user_string = UserInput::new(0, rows-2, cols).input(&mut stdout, ">", &mut cmd_history, &color_scheme);
                    if !user_string.is_empty() {
                        match Command::from_str(&user_string) {
                            Ok(c) => command = Some(c),
                            Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s, MessageBoxType::Error, &color_scheme); },
                        }
                    }
                },
                KeyEvent{ code: KeyCode::Char('t'), .. } if cursor.is_byte() || !cursor.is_visible() => {
                    cursor.set_state(CursorState::Text);

                    //move cursor to the screen range if needed
                    if cursor.position() < file_view_offset || cursor.position() >= file_view_offset + page_size {
                        cursor.set_position(file_view_offset);
                    }
                },
                KeyEvent{ code: KeyCode::Char('b'), .. } if !cursor.is_visible() => {
                    cursor.set_state(CursorState::Byte);

                    //move cursor to the screen range if needed
                    if cursor.position() < file_view_offset || cursor.position() >= file_view_offset + page_size {
                        cursor.set_position(file_view_offset);
                    }
                },
                KeyEvent{ code: KeyCode::Char('S'), .. } if cursor.is_byte() => {
                    let fb = &mut file_buffers[active_fb_index];

                    //select highlighted block and if not possible, select string under the cursor
                    //fb.set_selection(if let Some((s,e,_)) = fb.get_highlight(cursor.position()) {
                    //        Some((s,e))
                    //    } else {
                    //        command_functions::find_string_at_position(fb, cursor.position())
                    //    });
                    fb.set_selection(command_functions::find_string_at_position(fb, cursor.position()));
                },
                KeyEvent{ code: KeyCode::Char('s'), .. } if cursor.is_byte() => {
                    if !in_selection_mode && cursor.position() < file_buffers[active_fb_index].len() {
                        selection_start = cursor.position();
                        in_selection_mode = true;
                    } else {
                        in_selection_mode = false;
                    }
                },
                KeyEvent{ code: KeyCode::Char('y'), .. } if !cursor.is_text() => {
                    command = Some(Command::YankBlock);
                },
                KeyEvent{ code: KeyCode::Char('m'), .. } if !cursor.is_text() => {
                    if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                        let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                        file_buffers[active_fb_index].add_highlight(s, e, color);
                        file_buffers[active_fb_index].set_selection(None);
                    }
                },
                KeyEvent{ code: KeyCode::Char('M'), .. } if !cursor.is_text() => {
                    if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                        file_buffers[active_fb_index].remove_highlight(s, e);
                        file_buffers[active_fb_index].set_selection(None);
                    }
                },
                //all other keys
                KeyEvent{ code, .. } => {
                    match code {
                        //jump to bookmark
                        KeyCode::Char(ch) if !cursor.is_visible() && ('0'..='9').contains(&ch) => {
                            let ch = ch.to_digit(10).unwrap() as usize;
                            command = Some(Command::GotoBookmark(ch));
                        },

                        //edit in text mode
                        KeyCode::Char(ch) if cursor.is_text() && (' '..='~').contains(&ch) => {
                            file_buffers[active_fb_index].set(cursor.position(), ch as u8);
                            command = Some(Command::GotoRelative(1));
                        },
                        
                        //edit in byte mode
                        KeyCode::Char(ch) if cursor.is_byte() && (ch >= '0' && ch <= '9' || ch >= 'a' && ch <= 'f'|| ch >= 'A' && ch <= 'F') => {

                            let ch = ch.to_digit(16).unwrap() as u8;
                            let b = file_buffers[active_fb_index].get(cursor.position()).unwrap_or(0);

                            //modify HO/LO part of byte
                            let b = if cursor.ho_byte_part() {
                                b & 0x0F | (ch << 4)
                            } else {
                                b & 0xF0 | ch
                            };

                            file_buffers[active_fb_index].set(cursor.position(), b);
                            
                            //move cursor to the right only if LO part is modified
                            if !cursor.ho_byte_part() {
                                command = Some(Command::GotoRelative(1));
                            } else {
                                cursor.set_ho_byte_part(false);
                            }
                        },
                        _ => (),
                    }
                },
            }
        } else if let Event::Mouse(mouse_event) = event {
            let file_view_offset = file_buffers[active_fb_index].position();
            let row_size = screens[active_screen_index].row_size() as usize;
            let page_size = screens[active_screen_index].page_size();

            let scroll_size = config.mouse_scroll_size * match config.mouse_scroll_type {
                ScreenPagingSize::Byte => 1,
                ScreenPagingSize::Row => row_size,
                ScreenPagingSize::Page => page_size,
            };

            match mouse_event {
                MouseEvent{ kind: MouseEventKind::ScrollUp, column, row, .. } => {
                    if screens[active_screen_index].is_over_data_area(column, row) {
                        command = Some(Command::GotoRelative(-(scroll_size as isize)));

                    } else if screens[active_screen_index].is_over_location_bar(column, row) {
                        if let Some((o,_)) = file_buffers[active_fb_index].location_list_mut().previous() {
                            command = Some(Command::Goto(o))
                        }
                    }
                },
                MouseEvent{ kind: MouseEventKind::ScrollDown, column, row, .. } => {
                    if screens[active_screen_index].is_over_data_area(column, row) {
                        command = Some(Command::GotoRelative(scroll_size as isize));

                    } else if screens[active_screen_index].is_over_location_bar(column, row) {
                        if let Some((o,_)) = file_buffers[active_fb_index].location_list_mut().next() {
                            command = Some(Command::Goto(o))
                        }
                    }
                },
                MouseEvent{ kind: MouseEventKind::Down(MouseButton::Left), column, row, .. } => {
                    let screen = &screens[active_screen_index];
                    let is_double_click = column == last_mouse_col && row == last_mouse_row && last_click_time.elapsed().as_millis() < 500;

                    if screen.is_over_data_area(column, row) {
                        if let Some(fo) = screen.screen_coord_to_file_offset(file_view_offset, column, row) {
                            if is_double_click {
                                let fb = &mut file_buffers[active_fb_index];

                                //select highlighted block and if not possible, select string under the cursor
                                //fb.set_selection(if let Some((s,e,_)) = fb.get_highlight(fo) {
                                //        Some((s,e))
                                //    } else {
                                //        command_functions::find_string_at_position(fb, fo)
                                //    });
                                fb.set_selection(command_functions::find_string_at_position(fb, cursor.position()));
                            } else {
                                cursor.set_position(fo);
                            }
                        }

                    } else if screen.is_over_location_bar(column, row) {
                        let fb = &mut file_buffers[active_screen_index];

                        if let Some(loc_list_idx) = screen.location_list_index(column, row, fb.location_list()) {
                            if let Some((o,_)) = fb.location_list().get(loc_list_idx) {
                                fb.location_list_mut().set_current_index(loc_list_idx);
                                command = Some(Command::Goto(o));
                            }
                        }
                    }

                    last_click_time = Instant::now();
                    last_mouse_col = column;
                    last_mouse_row = row;
                }
                MouseEvent{ kind: MouseEventKind::Down(MouseButton::Right), column, row, .. } => {
                    if screens[active_screen_index].is_over_data_area(column, row) {
                        if let Some(fo) = screens[active_screen_index].screen_coord_to_file_offset(file_view_offset, column, row) {
                            file_buffers[active_fb_index].set_selection(Some((cursor.position(), fo)));
                        }
                    }
                }
                MouseEvent{ kind: MouseEventKind::Drag(MouseButton::Left), column, row, .. } => {
                    if screens[active_screen_index].is_over_data_area(column, row) {
                        if let Some(fo) = screens[active_screen_index].screen_coord_to_file_offset(file_view_offset, column, row) {
                            file_buffers[active_fb_index].set_selection(Some((cursor.position(), fo)));
                        }
                    }
                }
                _ => (),
            }
        }

        //commands evaluation
        if command.is_some() {
            let file_view_offset = file_buffers[active_fb_index].position();
            let row_size = screens[active_screen_index].row_size() as usize;
            let page_size = screens[active_screen_index].page_size();

            match command {
                Some(Command::Quit(save)) => {
                    if !save {
                        break;
                    }

                    while !file_buffers.is_empty() {
                        let fb = file_buffers.first().unwrap();
                        if fb.is_modified() {
                            let s = format!("Do you want to write changes to the file '{}'?", fb.filename());
                            match MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Confirmation, &color_scheme) {
                                MessageBoxResponse::Cancel => break,
                                MessageBoxResponse::No => (),
                                MessageBoxResponse::Yes => {
                                    if let Err(s) = command_functions::save_file(fb.filename(), fb.as_slice()) {
                                       MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme);
                                    }
                                },
                            }
                        }
                        file_buffers.remove(0);
                    }

                    //if there is any remaining filebuffer keep the program going.
                    if file_buffers.is_empty() {
                        break;
                    } else {
                        active_fb_index = 0;
                    }
                },
                Some(Command::Goto(o)) => {
                    if cursor.is_visible() {
                        cursor.set_position(o);

                        //if cursor is out of screen, adjust file_buffer position
                        if o < file_view_offset || o >= file_view_offset + page_size {
                            command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers);
                        }
                    } else {
                        command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers);
                    }
                },
                Some(Command::GotoRelative(delta_offset)) => {
                    let pos = if cursor.is_visible() { cursor.position() } else { file_view_offset };
                    let do_abs = delta_offset.unsigned_abs();

                    //calculate new offset for cursor or file_buffer
                    let new_pos = if delta_offset.is_negative() {
                        if do_abs >= pos { 0 } else { pos - do_abs }
                    } else {
                        pos + do_abs
                    };

                    if cursor.is_visible() {
                        cursor.set_position(new_pos);

                        let size = match config.screen_paging_size {
                            ScreenPagingSize::Byte => 1,
                            ScreenPagingSize::Row => row_size,
                            ScreenPagingSize::Page => page_size,
                        };

                        //if cursor is out of screen, adjust file_buffer position
                        //cursor / screen paging "upward"
                        if new_pos < file_view_offset { 
                            let count = (file_view_offset - new_pos + size - 1) / size;
                            let new_file_offset = if file_view_offset <= count * size { 0 } else { file_view_offset - count * size };
                            command_functions::set_position(&mut file_buffers, active_fb_index, new_file_offset, config.lock_file_buffers);
                        }
                        
                        //cursor / screen paging "downward"
                        if new_pos >= file_view_offset + page_size {
                            let count = (new_pos - file_view_offset - page_size + size) / size;
                            let new_file_offset = file_view_offset + count*size;
                            command_functions::set_position(&mut file_buffers, active_fb_index, new_file_offset, config.lock_file_buffers);
                        }
                    } else {
                        command_functions::set_position(&mut file_buffers, active_fb_index, new_pos, config.lock_file_buffers);
                    }
                },
                Some(Command::GotoBookmark(idx)) => {
                    if let Some(b_offset) = file_buffers[active_fb_index].bookmark(idx) {
                        if cursor.is_visible() {
                            cursor.set_position(b_offset);

                            if b_offset < file_view_offset || b_offset >= file_view_offset + page_size {
                                command_functions::set_position(&mut file_buffers, active_fb_index, b_offset, config.lock_file_buffers);
                            }
                        } else {
                            command_functions::set_position(&mut file_buffers, active_fb_index, b_offset, config.lock_file_buffers);
                        }
                    } else if idx > 9 {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Please specify 'bookmark_index' from 0 to 9!", MessageBoxType::Error, &color_scheme);
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, format!("Bookmark '{}' not set.", idx as u8).as_str(), MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::Bookmark(bookmark_idx, offset)) => {
                    if bookmark_idx > 9 {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Please specify 'bookmark_index' from 0 to 9!", MessageBoxType::Error, &color_scheme);
                    } else {
                        let o = match offset {
                            Some(_) => offset,
                            None if cursor.is_visible() => Some(cursor.position()),
                            None => Some(file_view_offset),
                        };
                        file_buffers[active_fb_index].set_bookmark(bookmark_idx, o);
                    }
                },
                Some(Command::FindPatch) => {
                    match command_functions::find_patch(&file_buffers[active_fb_index]) {
                        Ok(o) => command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers),
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindAllPatches) => {
                    match command_functions::find_all_patches(&file_buffers[active_fb_index]) {
                        Ok(ll) => {
                            file_buffers[active_fb_index].set_location_list(ll);
                            screens.iter_mut().for_each(|s| s.show_location_bar(true));
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::Find(mut b)) => {
                    //if pattern is empty try to use selection
                    if b.is_empty() {
                        if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                            b = file_buffers[active_fb_index].as_slice()[s..=e].to_vec();
                        }
                    }

                    //if it is still empty no block was selected. Display error message
                    if b.is_empty() {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "No pattern or block specified!", MessageBoxType::Error, &color_scheme);
                    } else {
                        match command_functions::find(&file_buffers[active_fb_index], &b) {
                            Ok(o) => command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers),
                            Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                        }
                    }
                },
                Some(Command::FindAll(mut b)) => {
                    if b.is_empty() {
                        if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                            b = file_buffers[active_fb_index].as_slice()[s..=e].to_vec();
                        }
                    }

                    if b.is_empty() {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "No pattern or block specified!", MessageBoxType::Error, &color_scheme);

                    } else {
                        match command_functions::find_all(&file_buffers[active_fb_index], &b) {
                            Ok(ll) => {
                                if let Some((o,_)) = ll.get(0) {
                                    command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers);
                                    file_buffers[active_fb_index].clear_highlights();
                                    ll.iter().for_each(|(o,_)| {
                                        let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                                        file_buffers[active_fb_index].add_highlight(*o, *o + b.len() - 1, color);
                                    });
                                    file_buffers[active_fb_index].set_location_list(ll);
                                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
                                }
                            },
                            Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                        }
                    }
                },
                Some(Command::FindString(min_size, substring)) => {
                    match command_functions::find_string(&file_buffers[active_fb_index], min_size, &substring) {
                        Ok(o) => command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers),
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindAllStrings(min_size, substring)) => {
                    match command_functions::find_all_strings(&file_buffers[active_fb_index], min_size, &substring) {
                        Ok(ll) => {
                            file_buffers[active_fb_index].clear_highlights();
                            ll.iter().for_each(|(o,s)| {
                                let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                                file_buffers[active_fb_index].add_highlight(*o, *o + s.len() - 1, color);
                            });
                            file_buffers[active_fb_index].set_location_list(ll);
                            screens.iter_mut().for_each(|s| s.show_location_bar(true));
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindDiff) => {
                    match command_functions::find_diff(&file_buffers, active_fb_index) {
                        Some(o) => command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers),
                        None => { MessageBox::new(0, rows-2, cols).show(&mut stdout, "No more diffs.", MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindAllDiffs) => {
                    match command_functions::find_all_diffs(&file_buffers, active_fb_index) {
                        Ok(ll) => {
                            file_buffers[active_fb_index].set_location_list(ll);
                            screens.iter_mut().for_each(|s| s.show_location_bar(true));
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindAllSignatures(signature_list, ignored)) => {
                    match command_functions::find_all_signatures(&file_buffers, active_fb_index, signature_list, ignored) {
                        Ok(ll) => {
                            file_buffers[active_fb_index].set_location_list(ll);
                            screens.iter_mut().for_each(|s| s.show_location_bar(true));
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindAllBookmarks) => {
                    match command_functions::find_all_bookmarks(&file_buffers, active_fb_index) {
                        Ok(ll) => {
                            file_buffers[active_fb_index].set_location_list(ll);
                            screens.iter_mut().for_each(|s| s.show_location_bar(true));
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::Entropy(block_size, margin)) => {
                    let ll = command_functions::calculate_entropy(&file_buffers[active_fb_index], block_size, margin);
                    file_buffers[active_fb_index].set_location_list(ll);
                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
                },
                Some(Command::OpenFile(file_name)) => {
                    match command_functions::read_file(&file_name) {
                        Ok(file_data) => {
                            let mut fb = FileBuffer::from_vec(file_data);
                            fb.set_filename(&file_name);
                            file_buffers.push(fb); 
                            active_fb_index = file_buffers.len() - 1;
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::CloseFile) => {
                    if file_buffers[active_fb_index].is_modified() {  
                        let filename = file_buffers[active_fb_index].filename(); 
                        let s = format!("File '{}' seems to be modified. Do you want to save it?", filename);
                        match MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Confirmation, &color_scheme) {
                            MessageBoxResponse::Cancel => (),
                            MessageBoxResponse::No => { file_buffers.remove(active_fb_index); },
                            MessageBoxResponse::Yes => {
                                if let Err(s) = command_functions::save_file(filename, file_buffers[active_fb_index].as_slice()) {
                                   MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme);
                                }
                                file_buffers.remove(active_fb_index);
                            },
                        }
                    } else {
                        file_buffers.remove(active_fb_index);
                    }

                    //if it was last file, quit program
                    if file_buffers.is_empty() {
                        break;
                    }

                    if active_fb_index == file_buffers.len() {
                        active_fb_index -= 1;
                    }
                },
                Some(Command::SaveFile(file_name)) => {
                    let file_name = file_name.unwrap_or(file_buffers[active_fb_index].filename().to_owned());
                    match command_functions::save_file(&file_name, file_buffers[active_fb_index].as_slice()) {
                        Ok(count) => {
                            MessageBox::new(0, rows-2, cols).show(&mut stdout, format!("written {} bytes to '{}'.", count, file_name).as_str(), MessageBoxType::Informative, &color_scheme);
                            file_buffers[active_fb_index].set_filename(&file_name);
                            file_buffers[active_fb_index].clear_patches();
                            file_buffers[active_fb_index].reset_hash();
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::SaveBlock(file_name)) => {
                    match command_functions::save_block(&file_buffers, active_fb_index, &file_name) {
                        Ok(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Informative, &color_scheme); },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FillBlock(pattern_bytes)) => {
                    let fb = &mut file_buffers[active_fb_index];
                    match fb.selection() {
                        Some((start, end)) => {
                            pattern_bytes.iter().cycle().enumerate()
                                        .take_while(|(i,_)| *i < (end - start + 1))
                                        .for_each(|(i,&b)| fb.set(start + i, b));
                            fb.set_selection(None);
                        },
                        None => { MessageBox::new(0, rows-2, cols).show(&mut stdout, "Please select the block first.", MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::YankBlock) => {
                    //move bytes from selection into yank_buffer or clear it doesn't exist
                    if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                        yank_buffer = file_buffers[active_fb_index].as_slice()[s..=e].to_vec();
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, format!("Yanked {} bytes.", yank_buffer.len()).as_str(), MessageBoxType::Informative, &color_scheme);
                    } else {
                        yank_buffer.clear();
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Nothing to yank. Buffer cleared.", MessageBoxType::Informative, &color_scheme);
                    }
                },
                Some(Command::OpenBlock) => {
                    match command_functions::open_block(&mut file_buffers, active_fb_index, &yank_buffer) {
                        Ok(_) => active_fb_index = file_buffers.len() - 1,
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::InsertBlock) => {
                    if cursor.is_visible() {
                        //insert bytes to the file from selected or yanked block. 
                        if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                            let bytes = file_buffers[active_fb_index].as_slice()[s..=e].to_vec();
                            file_buffers[active_fb_index].insert_block(cursor.position(), bytes);
                        } else if !yank_buffer.is_empty() {
                            file_buffers[active_fb_index].insert_block(cursor.position(), yank_buffer.clone());
                        } else {
                            MessageBox::new(0, rows-2, cols).show(&mut stdout, "Please select or yank the block first.", MessageBoxType::Error, &color_scheme);
                        }
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Move cursor to the position where block should be inserted.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::AppendBlock) => {
                    if cursor.is_visible() {
                        if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                            let bytes = file_buffers[active_fb_index].as_slice()[s..=e].to_vec();
                            file_buffers[active_fb_index].insert_block(cursor.position()+1, bytes);
                        } else if !yank_buffer.is_empty() {
                            file_buffers[active_fb_index].insert_block(cursor.position()+1, yank_buffer.clone());
                        } else {
                            MessageBox::new(0, rows-2, cols).show(&mut stdout, "Please select or yank the block first.", MessageBoxType::Error, &color_scheme);
                        }
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Move cursor to the position where block should be appended.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::DeleteBlock) => {
                    let fb = &mut file_buffers[active_fb_index];
                    if fb.selection().is_some() {
                        fb.remove_block();
                        fb.set_selection(None);
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Please select the block first.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::InsertFile(file_name)) => {
                    if cursor.is_visible() {
                        match command_functions::read_file(&file_name) {
                            Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                            Ok(data) => { file_buffers[active_fb_index].insert_block(cursor.position(), data); },
                        }
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Move cursor to the position where block should be inserted.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::AppendFile(file_name)) => {
                    if cursor.is_visible() {
                        match command_functions::read_file(&file_name) {
                            Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                            Ok(data) => { file_buffers[active_fb_index].insert_block(cursor.position() + 1, data); },
                        }
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Move cursor to the position where block should be appended.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::InsertFilledBlock(bytes)) => {
                    if cursor.is_visible() {
                        file_buffers[active_fb_index].insert_block(cursor.position(), bytes);
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Move cursor to the position where block should be inserted.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::AppendFilledBlock(bytes)) => {
                    if cursor.is_visible() {
                        file_buffers[active_fb_index].insert_block(cursor.position()+1, bytes);
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Move cursor to the position where block should be appended.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::ClearLocationBar) => {
                    file_buffers[active_fb_index].set_location_list(location_list::LocationList::new());
                    file_buffers[active_fb_index].clear_highlights();
                },
                Some(Command::Set(name, value)) => {
                    if let Err(s) = command_functions::set_variable(&name, &value, &mut config, &mut color_scheme) {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme);
                    }
                },
                None => (),
            }
        }

        //set selection to current filebuffer
        if in_selection_mode {
            file_buffers[active_fb_index].set_selection(Some((selection_start, cursor.position())));
        }

        //redraw screen
        screens[active_screen_index].draw(&mut stdout, &file_buffers, active_fb_index, &cursor, &color_scheme, &config); 
        stdout.flush().unwrap();
    }

    //deinit mouse
    if config.mouse_enabled {
        stdout.queue(DisableMouseCapture).unwrap();
    }

    //deint terminal
    crossterm::terminal::disable_raw_mode().unwrap();
    stdout.queue(crossterm::cursor::Show).unwrap();
    stdout.queue(ResetColor).unwrap();
    if config.clear_screen_on_exit {
        stdout.queue(crossterm::cursor::MoveTo(0,0)).unwrap();
        stdout.queue(Print(Clear(ClearType::All))).unwrap();
    }
}
