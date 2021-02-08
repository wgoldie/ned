use std::env;
use std::io::Read;
use std::ops::Range;
use std::io::Write;
use std::io;
use std::cmp::Ordering;
use std::fs::OpenOptions;

fn load_file() -> String {
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
    buffer
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

enum Address {
    Current,
    Last,
    Nth(usize),
    NthPrevious(usize),
    NthNext(usize),
}

enum AddressOrRange<T> {
    Address(T),
    AddressRange(T, T),
}

impl AddressOrRange<usize> {
    fn to_range(self) -> Range<usize> {
        match self {
            AddressOrRange::Address(a) => (a..(a+1)),
            AddressOrRange::AddressRange(a, b) => a..b,
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

fn parse_command(command_str: &str, addr: Option<AddressOrRange<Address>>) -> Option<(NedCommand, &str)> {
    let addr_specified = addr.is_none();
    match (command_str.chars().nth(0), addr.unwrap_or(AddressOrRange::Address(Address::Current)), addr_specified) {
        (Some('a'), AddressOrRange::Address(x), _) => Some((NedCommand::Append(x), &command_str[1..])),
        (Some('i'), AddressOrRange::Address(x), _) => Some((NedCommand::Insert(x), &command_str[1..])),
        (Some('n'), x, _) => Some((NedCommand::PrintLn(x), &command_str[1..])),
        (Some('p'), x, _) => Some((NedCommand::Print(x), &command_str[1..])),
        (Some('c'), x, _) => Some((NedCommand::Change(x), &command_str[1..])),
        (Some('d'), x, _) => Some((NedCommand::Delete(x), &command_str[1..])),
        (Some('q'), _, true) => Some((NedCommand::Quit, &command_str[1..])),
        _ => None 
    }
}

fn parse_command_str(command_str: &str) -> Option<NedCommand> {
    let (aorr_1, remainder) = parse_leading_address(command_str);
    let (aorr, remainder) = match (aorr_1, remainder.chars().nth(0)) {
        (Some(AddressOrRange::Address(addr)), Some(',')) => {
            let (aorr_2, remainder_2) = parse_address(command_str);

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
            for (i, line) in input.iter().enumerate() { 
                // TODO this clone should be replaced with lifetime
                state.line_buffer.insert(state.reify_address(&addr)? + i, line.to_owned());
            }
            Some(CommandResult::Noop)
        },
        Some(NedCommand::Insert(addr)) => {
            let input = run_input();
            for (i, line) in input.iter().enumerate() { 
                // TODO this clone should be replaced with lifetime
                state.line_buffer.insert(state.reify_address(&addr)? - 1 + i, line.to_owned());
            }
            Some(CommandResult::Noop)
        },
        Some(NedCommand::Print(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            for line in state.line_buffer.get(range).unwrap() {
                println!("{}", line);
            }
            Some(CommandResult::Noop)
        },
        Some(NedCommand::PrintLn(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            for i in range {
                println!("{} {}", i, state.line_buffer.get(i).unwrap());
            }
            Some(CommandResult::Noop)
        },
        Some(NedCommand::Change(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            let input = run_input();
            state.line_buffer = state.line_buffer.splice(range, input).collect();
            Some(CommandResult::Noop)
        }
        Some(NedCommand::Delete(addr_or_range)) => {
            let range = state.reify_address_or_range(&addr_or_range)?.to_range();
            state.line_buffer = state.line_buffer.splice(range, vec![]).collect();
            Some(CommandResult::Noop)
        },
        Some(NedCommand::Quit) => Some(CommandResult::Quit),
        None => None,
    }
}

struct NedState {
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

fn run_editor(buffer: &str) {
    let line_buffer: Vec<String> = buffer.split("\n").map(|s| s.to_string()).collect();
    let current_address = line_buffer.len() - 1;
    let mut state = NedState { line_buffer, current_address };
    loop {
        print_flush("> ");
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("IO");
        let result = run_command(&mut state, &command);
        match result {
            Some(CommandResult::Quit) => break,
            Some(CommandResult::Noop) => (),
            None => println!("?"),
        }
        println!("{:?}", state.line_buffer);
    }   
}

fn main() {
    let buffer = load_file();
    run_editor(&buffer);
}
