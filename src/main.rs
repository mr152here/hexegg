use std::{env, fs};
use std::io::{Read, Write};
use crossterm::style::{Color, Print, ResetColor};
use crossterm::terminal::{Clear, ClearType, size};
use crossterm::QueueableCommand;
use crossterm::event::{Event, KeyEvent, KeyCode, read, KeyModifiers};

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
use ui::elements::user_input::UserInput;
use ui::elements::message_box::*;

mod file_buffer;
use file_buffer::FileBuffer;

mod location_list;
use location_list::LocationList;
mod signatures;

fn create_screens(cols: u16, rows: u16) -> Vec<Box<dyn Screen>> {
    vec![
        Box::new(TextScreen::new(cols, rows)),
        Box::new(ByteScreen::new(cols, rows))
    ]
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
            hexegg version {}\nusage: hexegg [file1] <file2> <file3> ...\n", VERSION);
        return;
    }

    //init terminal
    let mut stdout = std::io::stdout();
    match crossterm::terminal::enable_raw_mode() {
        Ok(_) => {
            stdout.queue(Print(Clear(ClearType::All))).unwrap();
            stdout.queue(crossterm::cursor::Hide).unwrap();
        },
        Err(s) => { println!("{}", s); return; },
    }

    let (mut cols, mut rows) = size().unwrap();
    let mut cursor = Cursor::new(0, CursorState::Hidden);
    let mut screens = create_screens(cols, rows);
    screens[0].draw(&mut stdout, &file_buffers, active_fb_index, &cursor, &color_scheme, &config); 
    stdout.flush().unwrap();
   
    let mut random_seed = 0x5EED;
    let mut active_screen_index = 0;
    let mut in_selection_mode = false;
    let mut selection_start = 0;
    let mut cmd_history = Vec::<String>::new();

    //main program loop
    loop {

        //recreate screens if terminal size change
        let (new_cols, new_rows) = size().unwrap();
        if new_cols != cols || new_rows != rows {
            cols = new_cols;
            rows = new_rows;
            screens = create_screens(cols, rows);
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
                KeyEvent{ code: KeyCode::Char('i'), .. } if cursor.is_byte() || !cursor.is_visible() => screens.iter_mut().for_each(|s| s.toggle_info_bar()),
                KeyEvent{ code: KeyCode::Char('l'), .. } if cursor.is_byte() || !cursor.is_visible() => screens.iter_mut().for_each(|s| s.toggle_location_bar()),
                KeyEvent{ code: KeyCode::Char('o'), .. } if cursor.is_byte() || !cursor.is_visible() => screens.iter_mut().for_each(|s| s.toggle_offset_bar()),
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
                    let fb = &mut file_buffers[active_screen_index];

                    //get highlighted block, if not possible select string under the cursor
                    if let Some((s,e,_)) = fb.get_highlight(cursor.position()) {
                        fb.set_selection(Some((s,e)));

                    } else if let Some(b) = fb.get(cursor.position()) {
                        if (0x20..=0x7E).contains(&b) {
                            let (mut s, mut e) = (cursor.position(), cursor.position());

                            //find start/end of the string
                            while let Some(b) = fb.get(e + 1) {
                                if !(0x20..=0x7E).contains(&b) {
                                    break;
                                } 
                                e += 1;
                            }
                            while let Some(b) = fb.get(s.saturating_sub(1)) {
                                if !(0x20..=0x7E).contains(&b) {
                                    break;
                                } 
                                s = s.saturating_sub(1);
                            }
                            fb.set_selection(Some((s,e)));
                        } else {
                            fb.set_selection(None);
                        }
                    } 
                },
                KeyEvent{ code: KeyCode::Char('s'), .. } if cursor.is_byte() => {
                    if !in_selection_mode && cursor.position() < file_buffers[active_fb_index].len() {
                        selection_start = cursor.position();
                        in_selection_mode = true;
                    } else {
                        in_selection_mode = false;
                    }
                },
                KeyEvent{ code: KeyCode::Char('y'), .. } if cursor.is_byte() => {
                    command = Some(Command::YankBlock);
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
                    let ll = command_functions::find_all_strings(&file_buffers[active_fb_index], min_size, &substring);
                    file_buffers[active_fb_index].clear_highlights();
                    ll.iter().for_each(|(o,s)| {
                        let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                        file_buffers[active_fb_index].add_highlight(*o, *o + s.len() - 1, color);
                    });
                    file_buffers[active_fb_index].set_location_list(ll);
                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
                },
                Some(Command::FindDiff) => {
                    match command_functions::find_diff(&file_buffers, active_fb_index) {
                        Some(o) => command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers),
                        None => { MessageBox::new(0, rows-2, cols).show(&mut stdout, "No more diffs.", MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindAllDiffs) => {
                    let ll = command_functions::find_all_diffs(&file_buffers, active_fb_index);
                    file_buffers[active_fb_index].set_location_list(ll);
                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
                },
                Some(Command::FindAllSignatures) => {
                    let ll = command_functions::find_all_headers(&file_buffers, active_fb_index);
                    file_buffers[active_fb_index].set_location_list(ll);
                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
                },
                Some(Command::FindAllBookmarks) => {
                    let fb = &mut file_buffers[active_fb_index];
                    let ll = (0..10).into_iter()
                        .filter_map(|idx| fb.bookmark(idx).map(|o| (o,format!("bm_{}",idx))))
                        .collect::<LocationList>();

                    fb.set_location_list(ll);
                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
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
                    let file_name = if file_name.is_empty() { file_buffers[active_fb_index].filename().to_owned() } else { file_name };
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
                    match file_buffers[active_fb_index].selection() {
                        Some((start, end)) => {
                            match command_functions::save_file(&file_name, &file_buffers[active_fb_index].as_slice()[start..=end]) {
                                Ok(count) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, format!("written {} bytes to '{}'.", count, file_name).as_str(), MessageBoxType::Informative, &color_scheme); },
                                Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                            }
                        },
                        None => { MessageBox::new(0, rows-2, cols).show(&mut stdout, "Please select the block first.", MessageBoxType::Error, &color_scheme); },
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

    //deint terminal
    crossterm::terminal::disable_raw_mode().unwrap();
    stdout.queue( crossterm::cursor::Show ).unwrap();
    stdout.queue( ResetColor ).unwrap();
    if config.clear_screen_on_exit {
        stdout.queue(crossterm::cursor::MoveTo(0,0)).unwrap();
        stdout.queue(Print(Clear(ClearType::All))).unwrap();
    }
}
