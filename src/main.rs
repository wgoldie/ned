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

fn run_command(line_buffer: &mut Vec<String>, current_address: usize, command: &str) {
    match command.trim() {
        "a" => {
            let input = run_input();
            for (i, line) in input.iter().enumerate() { 
                // TODO this clone should be replaced with lifetime
                line_buffer.insert(current_address + i, line.to_owned());
            }
        },
        _ => (),
    }
}

fn run_editor(buffer: &str) {
    let mut line_buffer: Vec<String> = buffer.split("\n").map(|s| s.to_string()).collect();
    let current_address: usize = line_buffer.len() - 1;
    loop {
        print_flush("> ");
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("IO");
        run_command(&mut line_buffer, current_address, &command);
        println!("{:?}", line_buffer);
    }   
}

fn main() {
    let buffer = load_file();
    run_editor(&buffer);
}
