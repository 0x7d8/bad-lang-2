mod runtime;
mod token;

fn main() {
    let file = std::env::args().nth(1).expect("no file provided");

    let input = std::fs::read_to_string(file).unwrap();
    let mut tokenizer = token::Tokenizer::new(&input);

    tokenizer.parse();

    let mut runtime = runtime::Runtime::new(tokenizer.tokens.clone());
    runtime.run();
}
