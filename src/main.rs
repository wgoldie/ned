use std::env;
use std::io::Read;
use std::io::Write;
use std::io;
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

enum AddressOrRange {
    Address(Address),
    AddressRange(Address, Address),
}

enum NedCommand {
    Append(Address),
    Insert(Address),
    PrintLn(AddressOrRange),
    Print(AddressOrRange),
    Change(AddressOrRange),
    Delete(AddressOrRange),
    Quit
}

fn parse_leading_address(command_str: &str) -> (Option<AddressOrRange>, &str) {
    match command_str.chars().nth(0) {
        Some('.') => (Some(AddressOrRange::Address(Address::Current)), &command_str[1..]),
        Some('$') => (Some(AddressOrRange::Address(Address::Last)), &command_str[1..]),
        Some('-') => (Some(AddressOrRange::Address(Address::NthPrevious(1))), &command_str[1..]),
        Some('+') => (Some(AddressOrRange::Address(Address::NthNext(1))), &command_str[1..]),
        Some(';') => (Some(AddressOrRange::AddressRange(Address::Current, Address::Last)), &command_str[1..]),
        _ => (None, command_str)
    }
}

fn parse_command(command_str: &str, addr: Option<AddressOrRange>) -> Option<(NedCommand, &str)> {
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
    let (addresses, remainder) = parse_leading_address(command_str);
    let (command, remainder) = parse_command(remainder, addresses)?;
    Some(command)
}

fn run_command(state: &mut NedState, command_str: &str) {
    match parse_command_str(command_str) {
        Some(NedCommand::Append(addr)) => {
            let input = run_input();
            for (i, line) in input.iter().enumerate() { 
                // TODO this clone should be replaced with lifetime
                state.line_buffer.insert(state.current_address + i, line.to_owned());
            }
        },
        None => { println!("?") },
    }
}

struct NedState {
    line_buffer: Vec<String>,
    current_address: usize
}

fn run_editor(buffer: &str) {
    let line_buffer: Vec<String> = buffer.split("\n").map(|s| s.to_string()).collect();
    let current_address = line_buffer.len() - 1;
    let mut state = NedState { line_buffer, current_address };
    loop {
        print_flush("> ");
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("IO");
        run_command(&mut state, &command);
        println!("{:?}", state.line_buffer);
    }   
}

fn main() {
    let buffer = load_file();
    run_editor(&buffer);
}
