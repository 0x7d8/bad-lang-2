mod runtime;
mod token;

fn main() {
    let input = std::fs::read_to_string("input.bl2").unwrap();
    let mut tokenizer = token::Tokenizer::new(&input);

    tokenizer.parse();

    let mut runtime = runtime::Runtime::new(tokenizer.tokens.clone());
    runtime.run();
}
