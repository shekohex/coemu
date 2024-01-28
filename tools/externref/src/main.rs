use externref::processor::Processor;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let mut args = args.iter().skip(1);
    let input = args.next().unwrap_or_else(print_usage);
    let output = args.next().unwrap_or_else(print_usage);
    let module = std::fs::read(input).unwrap_or_else(|err| {
        eprintln!("Failed to read input file: {}", err);
        std::process::exit(1);
    });
    let processed: Vec<u8> = Processor::default().process_bytes(&module).unwrap();
    std::fs::write(output, processed).unwrap_or_else(|err| {
        eprintln!("Failed to write output file: {}", err);
        std::process::exit(1);
    });
}

fn print_usage<'a>() -> &'a String {
    eprintln!("Usage: {} <input> <output>", std::env::args().next().unwrap());
    std::process::exit(1);
}
