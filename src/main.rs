use std::env;
use std::io::Read;
use std::ops::Range;
use std::io::Write;
use std::io;
use std::fs::File;
use std::cmp::Ordering;
use std::cmp::min;
use std::fs::OpenOptions;

fn load_file() -> (File, String) {
    let filename = env::args()
        .nth(1)
        .expect("ned must be passed a filename");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&filename)
        .unwrap_or_else(|_| panic!("ned could not open file {}", filename));
    
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .expect("error reading file");
    (file, buffer)
}

fn print_flush(string: &str) {
    print!("{}", string);
    io::stdout().flush().expect("IO error");
}

fn run_input() -> Vec<String> {
    let mut input = vec![];
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("IO");
        buffer.pop(); // EOL
        if buffer == "." { break }
        input.push(buffer);
    }
    input
}

#[derive(Debug)]
enum Address {
    Current,
    Last,
    Nth(usize),
    NthPrevious(usize),
    NthNext(usize),
}

#[derive(Debug)]
enum AddressOrRange<T> {
    Address(T),
    AddressRange(T, T),
}

impl AddressOrRange<usize> {
    fn to_range(self) -> Range<usize> {
        match self {
            AddressOrRange::Address(a) => ((a-1)..(a)),
            AddressOrRange::AddressRange(a, b) => (a-1)..(b),
        }
    }
}

enum NedCommand {
    Append(Address),
    Insert(Address),
    PrintLn(AddressOrRange<Address>),
    Print(AddressOrRange<Address>),
    Change(AddressOrRange<Address>),
    Delete(AddressOrRange<Address>),
    Save(AddressOrRange<Address>),
    Quit
}

fn parse_address(command_str: &str) -> (Option<Address>, &str) {
    match command_str.chars().nth(0) {
        Some('.') => (Some(Address::Current), &command_str[1..]),
        Some('$') => (Some(Address::Last), &command_str[1..]),
        Some('-') => (Some(Address::NthPrevious(1)), &command_str[1..]),
        Some('+') => (Some(Address::NthNext(1)), &command_str[1..]),
        Some(c) => {
            if c.is_digit(10) {
                let numeric_string = command_str.chars()
                    .take_while(|c| c.is_digit(10))
                    .collect::<String>();
                let n = numeric_string.parse::<usize>().unwrap();
                (Some(Address::Nth(n)), &command_str[numeric_string.len()..])
            } else {
                (None, command_str)
            }
        }
        _ => (None, command_str)
    }
}

fn parse_address_range_shorthand(command_str: &str) -> (Option<AddressOrRange<Address>>, &str) {
    match command_str.chars().nth(0) {
        Some(',') => (Some(AddressOrRange::AddressRange(Address::Nth(1), Address::Last)), &command_str[1..]),
        Some(';') => (Some(AddressOrRange::AddressRange(Address::Current, Address::Last)), &command_str[1..]),
        _ => (None, command_str)
    }
}

fn parse_leading_address(command_str: &str) -> (Option<AddressOrRange<Address>>, &str) {
    if let (Some(addr), remainder) = parse_address(command_str) {
        (Some(AddressOrRange::Address(addr)), remainder)
    } else {
        parse_address_range_shorthand(command_str)
    }
}

const CURRENT_RANGE: AddressOrRange::<Address> = AddressOrRange::AddressRange(Address::Current, Address::Current);

fn parse_command(command_str: &str, addr: Option<AddressOrRange<Address>>) -> Option<(NedCommand, &str)> {
    let remainder = &command_str[1..];
    match (command_str.chars().nth(0), addr) {
        (Some('a'), Some(AddressOrRange::Address(addr))) => Some((NedCommand::Append(addr), remainder)),
        (Some('a'), None) => Some((NedCommand::Append(Address::Current), remainder)),
        (Some('i'), Some(AddressOrRange::Address(addr))) => Some((NedCommand::Insert(addr), remainder)),
        (Some('i'), None) => Some((NedCommand::Insert(Address::Current), remainder)),
        (Some('n'), Some(aorr)) => Some((NedCommand::PrintLn(aorr), remainder)),
        (Some('n'), None) => Some((NedCommand::PrintLn(CURRENT_RANGE), remainder)),
        (Some('p'), Some(aorr)) => Some((NedCommand::Print(aorr), remainder)),
        (Some('p'), None) => Some((NedCommand::Print(CURRENT_RANGE), remainder)),
        (Some('c'), Some(aorr)) => Some((NedCommand::Change(aorr), remainder)),
        (Some('c'), None) => Some((NedCommand::Change(CURRENT_RANGE), remainder)),
        (Some('d'), Some(aorr)) => Some((NedCommand::Delete(aorr), remainder)),
        (Some('d'), None) => Some((NedCommand::Delete(CURRENT_RANGE), remainder)),
        (Some('w'), Some(aorr)) => Some((NedCommand::Save(aorr), remainder)),
        (Some('w'), None) => Some((NedCommand::Save(AddressOrRange::AddressRange(Address::Nth(1), Address::Last)), remainder)),
        (Some('q'), None) => Some((NedCommand::Quit, remainder)),
        _ => None 
    }
}

fn parse_command_str(command_str: &str) -> Option<NedCommand> {
    let (aorr_1, remainder) = parse_leading_address(command_str);
    let (aorr, remainder) = match (aorr_1, remainder.chars().nth(0)) {
        (Some(AddressOrRange::Address(addr)), Some(',')) => {
            let (aorr_2, remainder_2) = parse_address(&remainder[1..]);

            (Some(AddressOrRange::AddressRange(addr, aorr_2.unwrap())), remainder_2)
        },
        (x, _) => (x, remainder),
    };

    let (command, _remainder) = parse_command(remainder, aorr)?;
    Some(command)
}

enum CommandResult {
    Noop,
    Quit,
}

fn run_command(state: &mut NedState, command_str: &str) -> Option<CommandResult> {
    match parse_command_str(command_str) {
        Some(NedCommand::Append(addr)) => {
            let input = run_input();
            let start_idx = state.reify_address(&addr)?;
            for (i, line) in input.iter().enumerate() { 
                // TODO this clone should be replaced with lifetime
                state.line_buffer.insert(start_idx + i, line.to_owned());
            }
            state.current_address = start_idx + input.len();
            Some(CommandResult::Noop)
        },
        Some(NedCommand::Insert(addr)) => {
            let input = run_input();
            let start_idx = state.reify_address(&addr)? - 1;
            for (i, line) in input.iter().enumerate() { 
                // TODO this clone should be replaced with lifetime
                state.line_buffer.insert(start_idx + i, line.to_owned());
            }
            state.current_address = start_idx + input.len();
            Some(CommandResult::Noop)
        },
        Some(NedCommand::Print(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            state.current_address = range.end;
            for line in state.line_buffer.get(range).unwrap() {
                println!("{}", line);
            }
            Some(CommandResult::Noop)
        },
        Some(NedCommand::PrintLn(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            state.current_address = range.end;
            for i in range {
                println!("{}\t{}", i + 1, state.line_buffer.get(i).unwrap());
            }
            Some(CommandResult::Noop)
        },
        Some(NedCommand::Change(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            let input = run_input();
            state.current_address = range.end;
            state.line_buffer.splice(range, input);
            Some(CommandResult::Noop)
        }
        Some(NedCommand::Delete(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            let end = range.start + 1;
            state.line_buffer.splice(range, vec![]);
            state.current_address = min(end, state.line_buffer.len());
            Some(CommandResult::Noop)
        },
        Some(NedCommand::Save(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            let lines = state.line_buffer.get(range).unwrap().join("\n");
            state.file.set_len(0).unwrap();
            write!(&mut state.file, "{}", lines).unwrap();
            state.file.flush().unwrap();
            Some(CommandResult::Noop)
        }
        Some(NedCommand::Quit) => Some(CommandResult::Quit),
        None => None,
    }
}

struct NedState {
    file: File,
    line_buffer: Vec<String>,
    current_address: usize
}

impl NedState {
    fn reify_address(&self, address: &Address) -> Option<usize> {
        let reified = match address {
            Address::Current => self.current_address,
            Address::Last => self.line_buffer.len(),
            Address::Nth(n) => n + 0, // TODO why do I have to do this?
            Address::NthPrevious(n) => self.current_address - n,
            Address::NthNext(n) => self.current_address + n,
        };

        match (reified.cmp(&1), reified.cmp(&self.line_buffer.len())) {
            (Ordering::Less, _) => None,
            (_, Ordering::Greater) => None,
            _ => Some(reified),
        }
    }

    fn reify_address_or_range(&self, address_or_range: &AddressOrRange<Address>) -> Option<AddressOrRange<usize>> {
        match address_or_range {
            AddressOrRange::Address(a) => Some(AddressOrRange::Address(self.reify_address(a)?)),
            AddressOrRange::AddressRange(a_1, a_2) => Some(AddressOrRange::AddressRange(self.reify_address(a_1)?, self.reify_address(a_2)?)),
        }
    }
}

fn run_editor(file: File, buffer: &str) {
    let line_buffer: Vec<String> = buffer.split("\n").map(|s| s.to_string()).collect();
    let current_address = line_buffer.len();
    let mut state = NedState { file, line_buffer, current_address };
    loop {
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("IO");
        let result = run_command(&mut state, &command);
        match result {
            Some(CommandResult::Quit) => break,
            Some(CommandResult::Noop) => (),
            None => println!("?"),
        }
    }   
}

fn main() {
    let (file, buffer) = load_file();
    run_editor(file, &buffer);
}
