use std::{env, fs};
use std::path::{Path, PathBuf};
use std::io::{Read, Write, IsTerminal};
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_family = "unix")]
use signal_hook::low_level;
use signal_hook::consts::signal::*;

use crossterm::style::{Color, Print, ResetColor};
use crossterm::terminal::{Clear, ClearType, size};
use crossterm::QueueableCommand;
use crossterm::event::{Event, KeyEvent, KeyCode, read, KeyModifiers, poll};
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture};

mod config;
use config::{Config, ColorScheme, HighlightStyle, ScreenPagingSize, StdinInput};

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
use location_list::LocationList;

mod highlight_list;
use highlight_list::HighlightList;

mod signatures;
mod struct_parsers;

fn parse_args() -> Result<(Option<u64>, Vec::<String>), String> {

    let mut file_names = Vec::<String>::new();
    let mut file_end_offset: Option<u64> = None;
    let mut arg_iter = env::args().skip(1);

    while let Some(arg) = arg_iter.next() {
        match arg.as_str() {
            "-t" => {
                file_end_offset = match arg_iter.next() {
                    Some(offset) => match offset.parse::<u64>() {
                        Ok(eo) => Some(eo),
                        Err(_) => return Err(format!("Unable to convert '{}' to integer!", offset)),
                    },
                    None => return Err("Expecting size limit after the '-t' parameter!".to_string()),
                };
            },
            s => file_names.push(s.to_string()),
        }
    }
    Ok((file_end_offset, file_names))
}

//returns path to the configuration file
fn find_config_file() -> Result<PathBuf, String> {

    //first try to find config in its own directory
    let prog_path = env::args().next().unwrap_or("".to_owned());
    let p = Path::new(&prog_path).parent().unwrap_or(Path::new("./")).join("config.toml");
    if p.exists() {
        return Ok(p);
    }

    //if not found try to use local user folder
    #[cfg(target_family = "unix")]
    {
        if let Ok(path) = std::env::var("HOME") {
            let mut pb = PathBuf::from(path);
            pb.push(".config/hexegg/config.toml");

            if pb.exists() {
                return Ok(pb);
            }
        }
        Err("Can't find 'config.toml'!\nPlease copy it to the '$HOME/.config/hexegg/' or to the current program location.".to_string())
    }

    #[cfg(target_family = "windows")]
    {
        if let Ok(path) = std::env::var("APPDATA") {
            let mut pb = PathBuf::from(path);
            pb.push("hexegg\\config.toml");

            if pb.exists() {
                return Ok(pb);
            }
        }
        Err("Can't find 'config.toml'!\nPlease copy it to the '%APPDATA%\\hexegg\\' or to the current program location.".to_string())
    }
}

fn create_screens(cols: u16, rows: u16, config: &Config) -> Vec<Box<dyn Screen>> {
    let mut screens = Vec::<Box<dyn Screen>>::new();

    if let Some(s) = config.screen_settings("text_screen") {
        if s.enabled {
            screens.push(Box::new(TextScreen::new(cols, rows, s)));
        }
    } else {
        println!("Can't found setting for 'text_screen' in 'config.toml' file.");
    }

    if let Some(s) = config.screen_settings("byte_screen") {
        if s.enabled {
            screens.push(Box::new( ByteScreen::new(cols, rows, s)));
        }
    } else {
        println!("Can't found setting for 'byte_screen' in 'config.toml' file.");
    }

    if let Some(s) = config.screen_settings("word_screen") {
        if s.enabled {
            screens.push(Box::new(WordScreen::new(cols, rows, s)));
        }
    } else {
        println!("Can't found setting for 'word_screen' in 'config.toml' file.");
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
    //get path to the configuration file
    let cfg_path = match find_config_file() {
        Ok(s) => s,
        Err(s) => { println!("{}", s); return; },
    };

    //read file with configuration
    let cfg_string = match fs::read_to_string(&cfg_path) {
        Ok(s) => s,
        Err(s) => { println!("Reading '{}' error: {}", cfg_path.to_str().unwrap(), s); return; },
    };

    //parse toml structure
    let mut config: Config = match toml::from_str(&cfg_string) {
        Ok(c) => c,
        Err(s) => { println!("Parsing '{}' error: {}", cfg_path.to_str().unwrap(), s); return; },
    };
    
    //load colorscheme
    let mut color_scheme: ColorScheme = match config.color_scheme(&config.active_color_scheme) {
        Some(cs) => cs.clone(),
        None => { println!("Can't find active color scheme. Please set 'active_color_scheme' in 'config.toml' to correct value"); return; },
    };

    let mut yank_buffer = Vec::<u8>::new();
    let mut file_buffers = Vec::<FileBuffer>::new();
    let mut active_fb_index = 0;

    //parse cmdline and load every file into separate file buffer
    match parse_args() {
        Ok((file_size_limit, file_names)) => {

            //open stdin for input data (if expected)
            let open_stdin = match config.stdin_input{
                StdinInput::Pipe => !std::io::stdin().is_terminal(),
                StdinInput::Always => true,
                StdinInput::Never => false,
            };

            if open_stdin {
                match command_functions::read_stdin(file_size_limit) {
                    Ok(file_data) => {
                        let mut fb = FileBuffer::from_vec(file_data);
                        fb.set_filename("stdin");
                        fb.set_truncate_on_save(true);
                        file_buffers.push(fb);
                    },
                    Err(s) => {
                        println!("{}\nPress enter to continue...", s);
                        std::io::stdin().read_exact(&mut [0; 1]).expect("Failed to read stdin!");
                    },
                }
            }

            //try to read each input file
            for file_name in file_names {
                match command_functions::read_file(&file_name, file_size_limit) {
                    Ok(file_data) => {
                        let mut fb = FileBuffer::from_vec(file_data);
                        fb.set_filename(&file_name);
                        fb.set_truncate_on_save(file_size_limit.is_none());
                        file_buffers.push(fb);
                    },
                    Err(s) => {
                        println!("Can't open file '{}'. {}\nPress enter to continue...", file_name, s);
                        std::io::stdin().read_exact(&mut [0; 1]).expect("Failed to read stdin!");
                    },
                }
            }
        },
        Err(s) => { println!("{}", s); return; },
    }

    //if there is nothing to open, print message and quit
    if file_buffers.is_empty() {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("Copyright 2023 Michal Kopera\n\
            This software is licensed under the terms of Apache 2.0 license. http://www.apache.org/licenses/LICENSE-2.0\n\
            Is distributed on an \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\n\n\
            hexegg version {}\nusage: hexegg [-t size_limit] <file1> [file2] [file3] ...\n", VERSION);
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

    //register for signals
    #[cfg(target_family = "unix")]
    let (signal_tstp, signal_cont) = (Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));

    #[cfg(target_family = "unix")] {
        signal_hook::flag::register(SIGTSTP, Arc::clone(&signal_tstp)).unwrap();
        signal_hook::flag::register(SIGCONT, Arc::clone(&signal_cont)).unwrap();
    }

    let signal_int= Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGINT, Arc::clone(&signal_int)).unwrap();

    //main program loop
    loop {

        //recreate screens if terminal size change
        let (new_cols, new_rows) = size().unwrap();
        if new_cols != cols || new_rows != rows {
            cols = new_cols;
            rows = new_rows;
            screens = create_screens(cols, rows, &config);
        }

        //active command
        let mut command: Option<Command> = None;

        match poll(Duration::from_millis(300)) {
            Err(s) => { println!("Can't poll events! {}", s); break; },

            //check for signals
            Ok(false) => {
                //process SIGINT
                if signal_int.load(Ordering::Relaxed) {
                    signal_int.store(false, Ordering::Relaxed);
                    command = Some(Command::Quit(false));
                }

                #[cfg(target_family = "unix")] {
                    //process SIGTSTP
                    if signal_tstp.load(Ordering::Relaxed) {
                        signal_tstp.store(false, Ordering::Relaxed);
                        command = Some(Command::Suspend);
                    }

                    //process SIGCONT
                    if signal_cont.load(Ordering::Relaxed) {
                        signal_cont.store(false, Ordering::Relaxed);

                        //set up terminal and mouse
                        match crossterm::terminal::enable_raw_mode() {
                            Ok(_) => {
                                stdout.queue(Print(Clear(ClearType::All))).unwrap();
                                stdout.queue(crossterm::cursor::Hide).unwrap();
                                if config.mouse_enabled {
                                    stdout.queue(EnableMouseCapture).unwrap();
                                }
                                stdout.flush().unwrap();
                            },
                            Err(s) => { println!("{}", s); break; },
                        }
                    }
                }
            },

            //process input events
            Ok(true) => {
                let event = read().unwrap();
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

                            } else if file_buffers[active_fb_index].selection().is_some() {
                                file_buffers[active_fb_index].set_selection(None);
                                in_selection_mode = false;

                            } else if config.esc_to_quit {
                                command = Some(Command::Quit(true));
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
                            let new_pos = file_size.saturating_sub(if cursor.is_visible() { 1 } else { page_size });
                            command = Some(Command::Goto(new_pos));
                        },
                        KeyEvent{ code: KeyCode::Enter, .. } => {
                            active_screen_index = (active_screen_index + 1) % screens.len();
                            command = cursor.is_visible().then_some(Command::Goto(cursor.position()));
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
                        KeyEvent{ code: KeyCode::Char('q'), .. } if !cursor.is_edit() => command = Some(Command::Quit(true)),
                        KeyEvent{ code: KeyCode::Char('h'), .. } if !cursor.is_edit() => config.highlight_diff = !config.highlight_diff,
                        KeyEvent{ code: KeyCode::Char('i'), .. } if !cursor.is_edit() => screens[active_screen_index].toggle_info_bar(),
                        KeyEvent{ code: KeyCode::Char('l'), .. } if !cursor.is_edit() => screens[active_screen_index].toggle_location_bar(),
                        KeyEvent{ code: KeyCode::Char('o'), .. } if !cursor.is_edit() => screens[active_screen_index].toggle_offset_bar(),
                        KeyEvent{ code: KeyCode::Char('k'), .. } if !cursor.is_edit() => config.lock_file_buffers = !config.lock_file_buffers,
                        KeyEvent{ code: KeyCode::Char('p'), .. } if !cursor.is_edit() => config.only_printable = ! config.only_printable,
                        KeyEvent{ code: KeyCode::Char('.'), .. } if !cursor.is_edit() => command = Some(Command::FindDiff),
                        KeyEvent{ code: KeyCode::Char(','), .. } if !cursor.is_edit() => command = Some(Command::FindPatch),
                        KeyEvent{ code: KeyCode::Char('['), .. } if !cursor.is_edit() => {
                            if let Some(loc) = file_buffers[active_fb_index].location_list_mut().previous() {
                                command = Some(Command::Goto(loc.offset))
                            }
                        },
                        KeyEvent{ code: KeyCode::Char(']'), .. } if !cursor.is_edit() => {
                            if let Some(loc) = file_buffers[active_fb_index].location_list_mut().next() {
                                command = Some(Command::Goto(loc.offset))
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('{'), .. } if !cursor.is_edit() => {
                            let lines = screens[active_screen_index].num_of_rows() as usize;
                            let ll = file_buffers[active_fb_index].location_list_mut();
                            ll.set_current_index(ll.current_index().saturating_sub(lines));

                            if let Some(loc) = ll.current() {
                                command = Some(Command::Goto(loc.offset))
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('}'), .. } if !cursor.is_edit() => {
                            let lines = screens[active_screen_index].num_of_rows() as usize;
                            let ll = file_buffers[active_fb_index].location_list_mut();
                            ll.set_current_index(ll.current_index() + lines);

                            if let Some(loc) = ll.current() {
                                command = Some(Command::Goto(loc.offset))
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('<'), .. } if !cursor.is_edit() => {
                            if let Some(loc) = file_buffers[active_fb_index].location_list().current() {
                                command = Some(Command::Goto(loc.offset))
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('>'), .. } if !cursor.is_edit() => {
                            let offset = if cursor.is_visible() { cursor.position() } else { file_buffers[active_fb_index].position() };
                            let ll = file_buffers[active_fb_index].location_list_mut();

                            if let Some(idx) = ll.find_idx(offset) {
                                ll.set_current_index(idx);
                                screens.iter_mut().for_each(|s| s.show_location_bar(true));
                            } else {
                                MessageBox::new(0, rows-2, cols).show(&mut stdout, format!("Offset {:08X} not found in location_bar.", offset).as_str(), MessageBoxType::Error, &color_scheme);
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('R'), .. } if !cursor.is_edit() => {
                            let fb = &mut file_buffers[active_fb_index];

                            if fb.location_list().current().is_some() {
                                let loc_offset = fb.location_list().current().unwrap().offset;
                                fb.location_list_mut().remove_current_location();
                                fb.highlight_list_mut().remove(loc_offset);
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('r'), .. } if !cursor.is_edit() => {
                            let ll = file_buffers[active_fb_index].location_list_mut();
                            if let Some(loc) = &mut ll.get_mut(ll.current_index()) {
                                let user_string = UserInput::new(0, rows-2, cols).input(&mut stdout, format!("rename '{}' to:", loc.name).as_str(), &mut cmd_history, &color_scheme);
                                if !user_string.is_empty() && *loc.name != user_string {
                                    loc.name = user_string;
                                }
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('-'), .. } if !cursor.is_edit() => screens[active_screen_index].dec_row_size(),
                        KeyEvent{ code: KeyCode::Char('+'), .. } if !cursor.is_edit() => screens[active_screen_index].inc_row_size(),
                        KeyEvent{ code: KeyCode::Char('/'), .. } if !cursor.is_edit() => {
                            let user_string = UserInput::new(0, rows-2, cols).input(&mut stdout, ">", &mut cmd_history, &color_scheme);
                            if !user_string.is_empty() {
                                match Command::from_str(&user_string) {
                                    Ok(c) => command = Some(c),
                                    Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s, MessageBoxType::Error, &color_scheme); },
                                }
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('n'), .. } if !cursor.is_visible() => {
                            cursor.set_state(CursorState::Normal);

                            //move cursor to the screen range if needed
                            if cursor.position() < file_view_offset || cursor.position() >= file_view_offset + page_size {
                                cursor.set_position(file_view_offset);
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('t'), .. } if !cursor.is_edit() => {
                            cursor.set_state(CursorState::Text);

                            //move cursor to the screen range if needed
                            if cursor.position() < file_view_offset || cursor.position() >= file_view_offset + page_size {
                                cursor.set_position(file_view_offset);
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('b'), .. } if !cursor.is_edit() => {
                            cursor.set_state(CursorState::Byte);

                            //move cursor to the screen range if needed
                            if cursor.position() < file_view_offset || cursor.position() >= file_view_offset + page_size {
                                cursor.set_position(file_view_offset);
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('s'), .. } if cursor.is_normal() => {
                            if !in_selection_mode && cursor.position() < file_buffers[active_fb_index].len() {
                                selection_start = cursor.position();
                                in_selection_mode = true;
                            } else {
                                in_selection_mode = false;
                            }
                        },
                        //select highlighted block from the cursor position
                        KeyEvent{ code: KeyCode::Char('H'), .. } if cursor.is_normal() => {
                            let fb = &mut file_buffers[active_fb_index];

                            fb.set_selection(
                                if let Some((s,e)) = fb.highlight_list().range(cursor.position()) {
                                    Some((s, std::cmp::min(e, fb.len())))
                                } else {
                                    None
                                }
                            );
                        },
                        //select ascii string from the cursor position
                        KeyEvent{ code: KeyCode::Char('S'), .. } if cursor.is_normal() => {
                            let fb = &mut file_buffers[active_fb_index];
                            fb.set_selection(command_functions::find_string_at_position(fb, cursor.position()));
                        },
                        //select "ascii-unicode" string under the cursor
                        KeyEvent{ code: KeyCode::Char('U'), .. } if cursor.is_normal() => {
                            let fb = &mut file_buffers[active_fb_index];
                            fb.set_selection(command_functions::find_unicode_string_at_position(fb, cursor.position()));
                        },
                        KeyEvent{ code: KeyCode::Char('y'), .. } if !cursor.is_edit() => {
                            command = Some(Command::YankBlock);
                        },
                        KeyEvent{ code: KeyCode::Char('m'), .. } if !cursor.is_edit() => {
                            if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                                let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                                file_buffers[active_fb_index].highlight_list_mut().add(s, e, Some(color));
                                file_buffers[active_fb_index].set_selection(None);
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('M'), .. } if !cursor.is_edit() => {
                            let fb = &mut file_buffers[active_fb_index];

                            if let Some((s,e)) = fb.selection() {
                                fb.highlight_list_mut().add(s, e, None);
                                fb.set_selection(None);
                            } else {
                                let offset = if cursor.is_visible() { cursor.position() } else { fb.position() };
                                if let Some((ho,_)) = fb.highlight_list().range(offset) {
                                    fb.highlight_list_mut().remove(offset);

                                    let ll = &mut fb.location_list_mut();
                                    if let Some(idx) = ll.find_idx(ho) {
                                        ll.remove_location(idx);
                                    }
                                }
                            }
                        },
                        KeyEvent{ code: KeyCode::Char('z'), modifiers: KeyModifiers::CONTROL, .. } => {
                            command = Some(Command::Suspend);
                        },
                        KeyEvent{ code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } => {
                            command = Some(Command::Quit(false));
                        },

                        //all other keys
                        KeyEvent{ code, .. } => {
                            match code {
                                //jump to bookmark
                                KeyCode::Char(ch) if !cursor.is_edit() && ch.is_ascii_digit() => {
                                    let ch = ch.to_digit(10).unwrap() as usize;
                                    command = Some(Command::GotoBookmark(ch));
                                },

                                //edit in text mode
                                KeyCode::Char(ch) if cursor.is_text() && (' '..='~').contains(&ch) => {
                                    file_buffers[active_fb_index].set(cursor.position(), ch as u8);
                                    command = Some(Command::GotoRelative(1));
                                },

                                //edit in byte mode
                                KeyCode::Char(ch) if cursor.is_byte() && ch.is_ascii_hexdigit() => {

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
                                if let Some(loc) = file_buffers[active_fb_index].location_list_mut().previous() {
                                    command = Some(Command::Goto(loc.offset))
                                }
                            }
                        },
                        MouseEvent{ kind: MouseEventKind::ScrollDown, column, row, .. } => {
                            if screens[active_screen_index].is_over_data_area(column, row) {
                                command = Some(Command::GotoRelative(scroll_size as isize));

                            } else if screens[active_screen_index].is_over_location_bar(column, row) {
                                if let Some(loc) = file_buffers[active_fb_index].location_list_mut().next() {
                                    command = Some(Command::Goto(loc.offset))
                                }
                            }
                        },
                        MouseEvent{ kind: MouseEventKind::Down(MouseButton::Left), column, row, .. } => {
                            let fb = &mut file_buffers[active_fb_index];
                            let screen = &screens[active_screen_index];
                            let is_double_click = column == last_mouse_col && row == last_mouse_row && last_click_time.elapsed().as_millis() < 500;

                            if screen.is_over_data_area(column, row) {

                                if let Some(fo) = screen.screen_coord_to_file_offset(file_view_offset, column, row) {
                                    if is_double_click {
                                        fb.set_selection(if let Some((s,e)) = fb.highlight_list().range(cursor.position()) {
                                                Some((s, std::cmp::min(e, fb.len())))
                                            } else {
                                                command_functions::find_string_at_position(fb, cursor.position())
                                            });
                                    } else {
                                        cursor.set_position(fo);
                                    }
                                }

                            } else if screen.is_over_location_bar(column, row) {

                                if let Some(loc_list_idx) = screen.location_list_index(column, row, fb.location_list()) {
                                    if let Some(loc) = fb.location_list().get(loc_list_idx) {
                                        if is_double_click && loc.size > 0 {
                                            fb.set_selection(Some((loc.offset, loc.offset + loc.size - 1)));
                                        } else {
                                            command = Some(Command::Goto(loc.offset));
                                            fb.location_list_mut().set_current_index(loc_list_idx);
                                        }
                                    }
                                }
                            }

                            last_click_time = Instant::now();
                            last_mouse_col = column;
                            last_mouse_row = row;
                        }
                        MouseEvent{ kind: MouseEventKind::Down(MouseButton::Right), column, row, .. } => {
                            let fb = &mut file_buffers[active_fb_index];
                            let screen = &screens[active_screen_index];

                            if screen.is_over_data_area(column, row) {
                                if let Some(fo) = screen.screen_coord_to_file_offset(file_view_offset, column, row) {
                                    fb.set_selection(Some((cursor.position(), fo)));
                                }

                            } else if screen.is_over_location_bar(column, row) {
                                if let Some(loc_list_idx) = screen.location_list_index(column, row, fb.location_list()) {
                                    if let Some(loc) = fb.location_list().get(loc_list_idx) {
                                        command = Some(Command::Goto(loc.offset + loc.size.saturating_sub(1)));
                                        fb.location_list_mut().set_current_index(loc_list_idx);
                                    }
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
            },
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
                                    if let Err(s) = command_functions::save_file(fb.filename(), fb.as_slice(), fb.truncate_on_save()) {
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
                        pos.saturating_sub(do_abs)
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
                            let new_file_offset = file_view_offset.saturating_sub(count * size);
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
                    let fb = &file_buffers[active_fb_index];
                    let start_offset = if cursor.is_visible() { cursor.position() } else { fb.position() } + 1;

                    match command_functions::find_patch(&file_buffers[active_fb_index], start_offset) {
                        Ok(o) if cursor.is_visible() => {
                            cursor.set_position(o);

                            if o < file_view_offset || o >= file_view_offset + page_size {
                                command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers);
                            }
                        },
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
                    let fb = &file_buffers[active_fb_index];

                    //if pattern is empty try to use data from selection
                    if b.is_empty() {
                        if let Some((s,e)) = fb.selection() {
                            b = fb.as_slice()[s..=e].to_vec();
                        }
                    }

                    //if it is still empty, no block was selected. Display error message
                    if b.is_empty() {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "No pattern or block specified!", MessageBoxType::Error, &color_scheme);
                    } else {
                        let start_offset = if cursor.is_visible() { cursor.position() } else { fb.position() } + 1;
                        match command_functions::find(fb.as_slice(), start_offset, &b) {
                            Ok(o) if cursor.is_visible() => {
                                cursor.set_position(o);

                                if o < file_view_offset || o >= file_view_offset + page_size {
                                    command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers);
                                }
                            },
                            Ok(o) => { command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers); },
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
                                if let Some(loc) = ll.get(0) {
                                    command_functions::set_position(&mut file_buffers, active_fb_index, loc.offset, config.lock_file_buffers);
                                    let hl = (&ll).into_iter()
                                                .map(|loc| {
                                                    let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                                                    (loc.offset, loc.offset + loc.size.saturating_sub(1), Some(color))
                                                })
                                                .collect::<HighlightList>();

                                    file_buffers[active_fb_index].set_location_list(ll);
                                    file_buffers[active_fb_index].set_highlight_list(hl);
                                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
                                }
                            },
                            Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                        }
                    }
                },
                Some(Command::FindString(min_size, substring)) => {
                    let fb = &file_buffers[active_fb_index];
                    let start_offset = if cursor.is_visible() { cursor.position() } else { fb.position() } + 1;

                    match command_functions::find_string(fb.as_slice(), start_offset, min_size, &substring) {
                        Ok(o) if cursor.is_visible() => {
                            cursor.set_position(o);

                            if o < file_view_offset || o >= file_view_offset + page_size {
                                command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers);
                            }
                        },
                        Ok(o) => command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers),
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindUnicodeString(min_size, substring)) => {
                    let fb = &file_buffers[active_fb_index];
                    let start_offset = if cursor.is_visible() { cursor.position() } else { fb.position() } + 1;

                    match command_functions::find_unicode_string(fb.as_slice(), start_offset, min_size, &substring) {
                        Ok(o) if cursor.is_visible() => {
                            cursor.set_position(o);

                            if o < file_view_offset || o >= file_view_offset + page_size {
                                command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers);
                            }
                        },
                        Ok(o) => command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers),
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindAllStrings(min_size, substring)) => {
                    match command_functions::find_all_strings(&file_buffers[active_fb_index], min_size, &substring) {
                        Ok(ll) => {
                            let hl = (&ll).into_iter()
                                        .map(|loc| {
                                            let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                                            (loc.offset, loc.offset + loc.size.saturating_sub(1), Some(color))
                                        })
                                        .collect::<HighlightList>();

                            file_buffers[active_fb_index].set_location_list(ll);
                            file_buffers[active_fb_index].set_highlight_list(hl);
                            screens.iter_mut().for_each(|s| s.show_location_bar(true));
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindAllUnicodeStrings(min_size, substring)) => {
                    match command_functions::find_all_unicode_strings(&file_buffers[active_fb_index], min_size, &substring) {
                        Ok(ll) => {
                            let hl = (&ll).into_iter()
                                        .map(|loc| {
                                            let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                                            (loc.offset, loc.offset + loc.size.saturating_sub(1), Some(color))
                                        })
                                        .collect::<HighlightList>();

                            file_buffers[active_fb_index].set_location_list(ll);
                            file_buffers[active_fb_index].set_highlight_list(hl);
                            screens.iter_mut().for_each(|s| s.show_location_bar(true));
                        },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::FindDiff) => {
                    let fb = &file_buffers[active_fb_index];
                    let start_offset = if cursor.is_visible() { cursor.position() } else { fb.position() } + 1;

                    match command_functions::find_diff(&file_buffers, start_offset, active_fb_index) {
                        Some(o) if cursor.is_visible() => {
                            cursor.set_position(o);

                            if o < file_view_offset || o >= file_view_offset + page_size {
                                command_functions::set_position(&mut file_buffers, active_fb_index, o, config.lock_file_buffers);
                            }
                        },
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
                Some(Command::FindAllHighlights) => {
                    let fb = &mut file_buffers[active_fb_index];
                    let ll = fb.highlight_list().iter().filter_map(|(o, c)| c.is_some().then_some((format!("{:08X}", o), *o)) ).collect::<LocationList>();

                    if ll.is_empty() {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "No highlights found!", MessageBoxType::Error, &color_scheme);
                    } else {
                        fb.set_location_list(ll);
                        screens.iter_mut().for_each(|s| s.show_location_bar(true));
                    }
                },
                Some(Command::ReplaceAll(b)) => {
                    match command_functions::replace_all(&mut file_buffers[active_fb_index], b) {
                        Ok(_) => {},
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::Entropy(block_size, margin)) => {
                    let ll = command_functions::calculate_entropy(&file_buffers[active_fb_index], block_size, margin);
                    file_buffers[active_fb_index].set_location_list(ll);
                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
                },
                Some(Command::Histogram) => {
                    let mut data = file_buffers[active_fb_index].as_slice();
                    if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                        data = &data[s..e];
                    };

                    let ll = command_functions::calculate_histogram(data);
                    file_buffers[active_fb_index].set_location_list(ll);
                    screens.iter_mut().for_each(|s| s.show_location_bar(true));
                },
                Some(Command::OpenFile(file_name, size_limit)) => {
                    match command_functions::read_file(&file_name, size_limit) {
                        Ok(file_data) => {
                            let mut fb = FileBuffer::from_vec(file_data);
                            fb.set_filename(&file_name);
                            fb.set_truncate_on_save(size_limit.is_none());
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
                                if let Err(s) = command_functions::save_file(filename, file_buffers[active_fb_index].as_slice(), file_buffers[active_fb_index].truncate_on_save()) {
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
                    match command_functions::save_file(&file_name, file_buffers[active_fb_index].as_slice(), file_buffers[active_fb_index].truncate_on_save()) {
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

                        //try to put data via stdin into external application
                        if let Err(s) = command_functions::pipe_block_to_program(&yank_buffer, &config.yank_to_program) {
                            MessageBox::new(0, rows-2, cols).show(&mut stdout, &s, MessageBoxType::Error, &color_scheme);
                        }

                    } else {
                        yank_buffer.clear();
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Nothing to yank. Buffer cleared.", MessageBoxType::Informative, &color_scheme);
                    }
                },
                Some(Command::PipeBlock(prog_cmd)) => {
                    if let Some((s,e)) = file_buffers[active_fb_index].selection() {
                        let data = file_buffers[active_fb_index].as_slice()[s..=e].to_vec();

                        if let Err(s) = command_functions::pipe_block_to_program(&data, &prog_cmd) {
                            MessageBox::new(0, rows-2, cols).show(&mut stdout, &s, MessageBoxType::Error, &color_scheme);
                        }

                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Please select the block first.", MessageBoxType::Error, &color_scheme);
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
                Some(Command::ExportBlock) => {
                    match command_functions::export_block(&file_buffers, active_fb_index) {
                        Ok(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Informative, &color_scheme); },
                        Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                    }
                },
                Some(Command::InsertFile(file_name)) => {
                    if cursor.is_visible() {
                        match command_functions::read_file(&file_name, None) {
                            Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                            Ok(data) => { file_buffers[active_fb_index].insert_block(cursor.position(), data); },
                        }
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Move cursor to the position where block should be inserted.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::AppendFile(file_name)) => {
                    if cursor.is_visible() {
                        match command_functions::read_file(&file_name, None) {
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
                Some(Command::ParseHeader(name)) => {
                    let fb = &mut file_buffers[active_fb_index];
                    let o = if cursor.is_visible() { cursor.position() } else { fb.position() };

                    if o < fb.len() {
                        match command_functions::parse_struct(&fb.as_slice()[o..], name) {
                            Err(s) => { MessageBox::new(0, rows-2, cols).show(&mut stdout, s.as_str(), MessageBoxType::Error, &color_scheme); },
                            Ok(ll) => {
                                let hl = (&ll).into_iter().filter_map(|fd| {
                                    if fd.size > 0 {
                                        let color = generate_highlight_color(&mut random_seed, config.highlight_style, &color_scheme);
                                        return Some((fd.offset + o, fd.offset + o + fd.size -1 , Some(color)));
                                    }
                                    None
                                }).collect::<HighlightList>();

                                fb.set_location_list(ll);
                                fb.set_highlight_list(hl);
                                screens.iter_mut().for_each(|s| s.show_location_bar(true));
                            },
                        }
                    } else {
                        MessageBox::new(0, rows-2, cols).show(&mut stdout, "Current position is out of the file range.", MessageBoxType::Error, &color_scheme);
                    }
                },
                Some(Command::Filter(filter_strings)) => {
                    let fb = &mut file_buffers[active_fb_index];
                    fb.set_filtered_location_list(None);

                    if !filter_strings.is_empty() {
                        let fll = fb.location_list()
                                        .into_iter()
                                        .filter_map(|loc| filter_strings.iter().any(|fs| loc.name.contains(fs)).then_some(loc.clone()))
                                        .collect();

                        fb.set_filtered_location_list(Some(fll));
                    }
                },
                Some(Command::ClearLocationBar) => {
                    file_buffers[active_fb_index].set_location_list(location_list::LocationList::new());
                    file_buffers[active_fb_index].highlight_list_mut().clear();
                },
                #[cfg(target_family = "unix")]
                Some(Command::Suspend) => {
                    //deinit terminal before suspend
                    if config.mouse_enabled {
                        stdout.queue(DisableMouseCapture).unwrap();
                    }
                    crossterm::terminal::disable_raw_mode().unwrap();
                    stdout.queue(crossterm::cursor::Show).unwrap();
                    stdout.queue(ResetColor).unwrap();
                    stdout.flush().unwrap();
                    low_level::emulate_default_handler(SIGTSTP).unwrap();
                },
                #[cfg(not(target_family = "unix"))]
                Some(Command::Suspend) => {
                    MessageBox::new(0, rows-2, cols).show(&mut stdout, "Supported only for OS from unix family.", MessageBoxType::Error, &color_scheme);
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
    } //main program loop

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
    stdout.flush().unwrap();
}
