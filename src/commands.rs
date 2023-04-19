pub enum Command {
    Quit(bool),
    Goto(usize),
    GotoRelative(isize),
    GotoBookmark(usize),
    Bookmark(usize, Option<usize>),
    Find(Vec<u8>),
    FindAll(Vec<u8>),
    FindString(usize, Vec<u8>),
    FindAllStrings(usize, Vec<u8>),
    FindDiff,
    FindAllDiffs,
    FindPatch,
    FindAllPatches,
    FindAllHighlights,
    FindAllSignatures(Option<Vec<String>>, bool),
    FindAllBookmarks,
    YankBlock,
    OpenBlock,
    InsertBlock,
    AppendBlock,
    DeleteBlock,
    SaveBlock(String),
    FillBlock(Vec<u8>),
    InsertFilledBlock(Vec<u8>),
    AppendFilledBlock(Vec<u8>),
    OpenFile(String),
    SaveFile(Option<String>),
    CloseFile,
    InsertFile(String),
    AppendFile(String),
    ClearLocationBar,
    Filter(Option<String>),
    Entropy(usize, f32),
    Histogram,
    ParseHeader(Option<String>),
    Set(String, String)
}


impl Command {

    pub fn from_str(command_string: &str) -> Result<Command, &'static str> {

        //split command string into the vector by whitespaces
        let cmd_vec: Vec<&str> = command_string.split_whitespace().collect();

        //first element should be the command word itself. the rest are parameters.
        match cmd_vec.first() {

            Some(&"quit") => Ok(Command::Quit(true)),
            Some(&"q") => Ok(Command::Quit(true)),
            Some(&"quit!") => Ok(Command::Quit(false)),
            Some(&"q!") => Ok(Command::Quit(false)),
            
            Some(&"goto") => Command::parse_goto(&cmd_vec),
            Some(&"g") => Command::parse_goto(&cmd_vec),

            Some(&"bookmark") => Command::parse_bookmark(&cmd_vec),
            Some(&"b") => Command::parse_bookmark(&cmd_vec),

            Some(&"findallbookmarks") => Ok(Command::FindAllBookmarks),
            Some(&"fab") => Ok(Command::FindAllBookmarks),

            Some(&"findallpatches") => Ok(Command::FindAllPatches),
            Some(&"fap") => Ok(Command::FindAllPatches),

            Some(&"find") => Command::parse_find(&cmd_vec),
            Some(&"f") => Command::parse_find(&cmd_vec),

            Some(&"findall") => Command::parse_find_all(&cmd_vec),
            Some(&"fa") => Command::parse_find_all(&cmd_vec),

            Some(&"findhex") => Command::parse_find_hex(&cmd_vec),
            Some(&"fx") => Command::parse_find_hex(&cmd_vec),

            Some(&"findallhex") => Command::parse_find_all_hex(&cmd_vec),
            Some(&"fax") => Command::parse_find_all_hex(&cmd_vec),

            Some(&"findstring") => Command::parse_find_string(&cmd_vec),
            Some(&"fs") => Command::parse_find_string(&cmd_vec),

            Some(&"findallstrings") => Command::parse_find_all_strings(&cmd_vec),
            Some(&"fas") => Command::parse_find_all_strings(&cmd_vec),

            Some(&"findalldiffs") => Ok(Command::FindAllDiffs),
            Some(&"fad") => Ok(Command::FindAllDiffs),

            Some(&"findallhighlights") => Ok(Command::FindAllHighlights),
            Some(&"fah") => Ok(Command::FindAllHighlights),

            Some(&"findallsignatures") => Command::parse_find_all_signatures(&cmd_vec, false),
            Some(&"fasi") => Command::parse_find_all_signatures(&cmd_vec, false),
            Some(&"findallsignatures!") => Command::parse_find_all_signatures(&cmd_vec, true),
            Some(&"fasi!") => Command::parse_find_all_signatures(&cmd_vec, true),

            Some(&"yankblock") => Ok(Command::YankBlock),
            Some(&"openblock") => Ok(Command::OpenBlock),
            Some(&"insertblock") => Ok(Command::InsertBlock),
            Some(&"appendblock") => Ok(Command::AppendBlock),
            Some(&"deleteblock") => Ok(Command::DeleteBlock),
            Some(&"saveblock") => Command::parse_save_block(&cmd_vec),
            Some(&"fillblock") => Command::parse_fill_block(&cmd_vec),
            Some(&"insertfilledblock") => Command::parse_insert_filled_block(&cmd_vec),
            Some(&"appendfilledblock") => Command::parse_append_filled_block(&cmd_vec),

            Some(&"openfile") => Command::parse_open_file(&cmd_vec),
            Some(&"savefile") => Command::parse_save_file(&cmd_vec),
            Some(&"closefile") => Ok(Command::CloseFile),
            Some(&"insertfile") => Command::parse_insert_file(&cmd_vec),
            Some(&"appendfile") => Command::parse_append_file(&cmd_vec),

            Some(&"clearlocationbar") => Ok(Command::ClearLocationBar),
            Some(&"filter") => Command::parse_filter(&cmd_vec),
            Some(&"entropy") => Command::parse_entropy(&cmd_vec),
            Some(&"ent") => Command::parse_entropy(&cmd_vec),
            Some(&"histogram") => Ok(Command::Histogram),
            Some(&"parseheader") => Command::parse_parse_header(&cmd_vec),
            Some(&"set") => Command::parse_set(&cmd_vec),
            
            //all unknown commands
            Some(_) => Err("Unknown command!"), 

            //just in the case of empty string.
            None => Err("Empty command."),
        }
    }

    fn parse_goto(v: &[&str]) -> Result<Command, &'static str> {
        match v.get(1) {
            Some(&s) => {
                let negative_goto = s.starts_with('-');
                let relative_goto = negative_goto || s.starts_with('+');
                let bookmark_goto = s.starts_with('b');
                let s = if relative_goto || bookmark_goto { &s[1..] } else { s };

                let (offset_str, radix) = if s.starts_with('x') || s.starts_with('X') { (&s[1..], 16) } else { (s, 10) };
                match isize::from_str_radix(offset_str, radix) {
                    Ok(offset) => Ok( 
                        if relative_goto {
                            Command::GotoRelative(if negative_goto { -offset } else { offset })
                        } else if bookmark_goto {
                            Command::GotoBookmark(offset as usize)
                        } else {
                            Command::Goto(offset as usize)
                        }),
                    Err(_) => Err("Can't convert 'position' to integer!"),
                }
            },
            None => Err("Missing 'position' parameter!"),
        }
    }

    fn parse_bookmark(v: &[&str]) -> Result<Command, &'static str> {
        //first parameter is bookmark idx
        let bookmark_idx: usize = match v.get(1) {
            Some(&s) => {
                match s.parse::<usize>() {
                    Ok(bs) => bs,
                    Err(_) => return Err("Can't convert 'bookmark_index' to integer!"),
                }
            },
            None => return Err("Please specify 'bookmark_index' from 0 to 9!"),
        };

        //second parameter is optional and is bookmark offset
        let offset: Option<usize> = match v.get(2) {
            Some(&s) => {
                let (offset_str, radix) = if s.starts_with('x') || s.starts_with('X') { (&s[1..], 16) } else { (s, 10) };
                match usize::from_str_radix(offset_str, radix) {
                    Ok(offset) => Some(offset),
                    Err(_) => return Err("Can't convert 'bookmark_offset' to integer!"),
                }
            },
            None => None,
        };
        Ok(Command::Bookmark(bookmark_idx, offset))
    }

    fn pattern_to_vec(pattern: &[u8]) -> Result<Vec<u8>, &'static str> {

        let mut ret_vec = Vec::<u8>::new();
        let mut idx = 0;
        let mut escaped = false;
        let mut hex_mode = false;

        while let Some(byte) = pattern.get(idx) {
            let b_char = *byte as char;

            if hex_mode {

                let b1: u8 =  match b_char.to_digit(16) {
                    Some(b1) => (b1 & 0x0F) as u8,
                    None => return Err("'pattern' syntax error. Expecting hex number after '\\x'!"),
                };

                //get next char
                idx += 1;
                match pattern.get(idx) {

                    //if there is something try to convert it to hex number
                    Some(&b2) => match (b2 as char).to_digit(16) {
                        Some(b2) => {
                            let b2 = (b1 << 4) | (b2 as u8 & 0x0F);
                            ret_vec.push(b2);
                        },
                        //if it can't be converted, it is following by non-hex character. Ignore it, push what
                        //we have and reduce idx back. So it will be processed by the next iteration
                        None => {
                            ret_vec.push(b1);
                            idx -= 1;
                        },
                    },
                    //if there is no next character, we are at the end. just push what we have
                    None => ret_vec.push(b1),
                }

                hex_mode = false;
                escaped = false;

            } else if b_char == '\\' {
                if escaped {
                    ret_vec.push(*byte);
                    escaped = false;
                } else {
                    escaped = true;
                }

            } else if escaped && (b_char == 'x' || b_char == 'X') {
                hex_mode = true;

            //if still escaped thats a unknown escaped character and therefore invalid syntax
            } else if escaped {
                return Err("'pattern' syntax error. Unknown escaped character!");

            } else {
                ret_vec.push(*byte);
            }

            idx += 1;
        }

        if hex_mode {
            Err("'pattern' syntax error. Expecting hex number after '\\x'!")
        } else if escaped {
            Err("'pattern' syntax error. Expecting '\\', 'x' or 'X' after the escape character!")
        } else {
            Ok(ret_vec)
        }
    }

    //find pattern in file buffer, if pattern is not defined, or is empty find selected block.
    fn parse_find(v: &[&str]) -> Result<Command, &'static str> {
        let bytes = match v.get(1) {
            Some(s) => s.as_bytes(),
            None => return Ok(Command::Find(Vec::new())),
        };

        match Self::pattern_to_vec(bytes) {
            Err(s) => Err(s),
            Ok(v) if v.is_empty() => Err("Invalid 'pattern' format!"),
            Ok(v)  => Ok(Command::Find(v)),
        }
    }

    //just wrapper to Command::Find
    fn parse_find_all(v: &[&str]) -> Result<Command, &'static str> {
        let result = Self::parse_find(v);

        if let Ok(Command::Find(vec)) = result {
            return Ok(Command::FindAll(vec))
        }
        result
    }

    //shortcut command to find \x..\x..\x..\x..
    fn parse_find_hex(v: &[&str]) -> Result<Command, &'static str> {

        let res = v.iter().skip(1)
                    .map(|s| u8::from_str_radix(s, 16))
                    .collect::<Result<Vec<u8>,_>>();

        match res {
            Err(_) => Err("Can't convert 'bytes' to hex integer!"),
            Ok(bytes) if bytes.is_empty() => Err("Missing 'bytes' parameter!"),
            Ok(bytes) => Ok(Command::Find(bytes)),
        }
    }

    //shortcut to findall \x..\x..\x..\x..
    fn parse_find_all_hex(v: &[&str]) -> Result<Command, &'static str> {
        let cmd = Command::parse_find_hex(v);

        if let Ok(Command::Find(b)) = cmd {
            return Ok(Command::FindAll(b));
        }
        cmd
    }

    //find string with defined minimium size and which contains specific substring. If defined.
    fn parse_find_string(v: &[&str]) -> Result<Command, &'static str> {

        //parse first parameter
        let min_size = match v.get(1) {
            Some(&s) => {

                //If is successfully converted into usize then it is 'min_size'. If not it is 'substring'
                match s.parse::<usize>() {
                    Ok(ms) => ms,
                    Err(_) => {
                        let substring = s.as_bytes().to_vec();
                        return Ok(Command::FindString(substring.len(), substring));
                    },
                }
            },
            None => return Err("At least 'min_size' or 'substring' parameter is required!"),
        };

        //parse second parameter. Should be 'substring' or nothing
        let substring = match v.get(2) {
            Some(&s) => s.as_bytes().to_vec(),
            None => Vec::new(),
        };

        Ok(Command::FindString(std::cmp::max(min_size, substring.len()), substring))
    }
    
    fn parse_find_all_strings(v: &[&str]) -> Result<Command, &'static str> {
        let cmd = Command::parse_find_string(v);

        if let Ok(Command::FindString(m,s)) = cmd {
            return Ok(Command::FindAllStrings(m,s));
        }
        cmd
    }

    fn parse_find_all_signatures(v: &[&str], ignored: bool) -> Result<Command, &'static str> {
        //if there is any parameter
        if v.len() > 1 {
            
            //get all parameters into the vector of strings.
            let sig_names = v.iter().skip(1)
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();

            return Ok(Command::FindAllSignatures(Some(sig_names), ignored));
        }
        Ok(Command::FindAllSignatures(None, ignored))
    }

    fn parse_filter(v: &[&str]) -> Result<Command, &'static str> {
        Ok(Command::Filter(v.get(1).map(|&s| s.to_owned())))
    }

    fn parse_entropy(v: &[&str]) -> Result<Command, &'static str> {
       
        //first parameter is block size
        let block_size: usize = match v.get(1) {
            Some(&s) => {
                match s.parse::<usize>() {
                    Ok(bs) => bs, 
                    Err(_) => return Err("Can't convert 'block_size' to integer!"),
                }
            },
            None => 1024,
        };

        //second parameter is margin
        let margin: f32 = match v.get(2) {
            Some(&s) => {
                match s.parse::<f32>() {
                    Ok(m) => 0.1_f32.max(m), 
                    Err(_) => return Err("Can't convert 'margin' to float!"),
                }
            },
            None => 1.1,
        };

        Ok(Command::Entropy(block_size, margin))
    }

    fn parse_open_file(v: &[&str]) -> Result<Command, &'static str> {
        match v.get(1) {
            Some(s) => Ok(Command::OpenFile(s.to_string())),
            None => Err("Missing 'filename' parameter!"),
        }
    }

    fn parse_save_file(v: &[&str]) -> Result<Command, &'static str> {
        Ok(Command::SaveFile(v.get(1).map(|s| s.to_string())))
    }

    fn parse_save_block(v: &[&str]) -> Result<Command, &'static str> {
        match v.get(1) {
            Some(s) => Ok(Command::SaveBlock(s.to_string())),
            None => Err("Missing 'filename' parameter!"),
        }
    }

    fn parse_fill_block(v: &[&str]) -> Result<Command, &'static str> {
        let bytes = match v.get(1) {
            Some(s) => s.as_bytes(),
            None => return Ok(Command::FillBlock(vec![0])),
        };

        match Self::pattern_to_vec(bytes) {
            Err(s) => Err(s),
            Ok(v) if v.is_empty() => Err("Invalid 'pattern' format!"),
            Ok(v) => Ok(Command::FillBlock(v)),
        }
    }

    fn parse_insert_file(v: &[&str]) -> Result<Command, &'static str> {
        match v.get(1) {
            Some(s) => Ok(Command::InsertFile(s.to_string())),
            None => Err("Missing 'filename' parameter!"),
        }
    }

    fn parse_append_file(v: &[&str]) -> Result<Command, &'static str> {
        let cmd = Command::parse_insert_file(v);

        if let Ok(Command::InsertFile(s)) = cmd {
            return Ok(Command::AppendFile(s));
        }
        cmd
    }

    fn parse_insert_filled_block(v: &[&str]) -> Result<Command, &'static str> {
        //first parameter is block size
        let block_size = match v.get(1) {
            Some(&s) => {
                let (value_str, radix) = if s.starts_with('x') || s.starts_with('X') { (&s[1..], 16) } else { (s, 10) };
                match usize::from_str_radix(value_str, radix ){
                    Ok(bs) => bs,
                    Err(_) => return Err("Can't convert 'block_size' to integer!"),
                }
            },
            None => return Err("Missing 'block_size' parameter!"),
        };

        //second parameter is a pattern. If not specified is used 0
        let tmp_vec = if let Some(pattern) = v.get(2) {
            match Self::pattern_to_vec(pattern.as_bytes()) {
                Err(s) => return Err(s),
                Ok(v) if v.is_empty() => return Err("Invalid 'pattern' format!"),
                Ok(v) => v,
            }
        } else {
            vec![0]
        };

        //create a bytes in its full size.
        let ret_vec = tmp_vec.iter()
                        .cycle()
                        .enumerate()
                        .take_while(|(i,_)| *i < block_size)
                        .map(|(_, b)| *b)
                        .collect();

        Ok(Command::InsertFilledBlock(ret_vec))
    }

    fn parse_append_filled_block(v: &[&str]) -> Result<Command, &'static str> {
        let cmd = Command::parse_insert_filled_block(v);

        if let Ok(Command::InsertFilledBlock(bytes)) = cmd {
            return Ok(Command::AppendFilledBlock(bytes));
        }
        cmd
    }

    fn parse_set(v: &[&str]) -> Result<Command, &'static str> {
        //first parameter is variable name
        let var_name = match v.get(1) {
            Some(s) => s.to_string(),
            None => return Err("Missing 'variable_name' parameter!"),
        };

        //second is value
        match v.get(2) {
            Some(s) => Ok(Command::Set(var_name, s.to_string())),
            None => Err("Missing 'variable_value' parameter!"),
        }
    }

    fn parse_parse_header(v: &[&str])-> Result<Command, &'static str> {
        let sig_name = v.get(1).map(|s| s.to_string());
        Ok(Command::ParseHeader(sig_name))
    }
}
