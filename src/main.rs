mod runtime;
mod token;

fn main() {
    let file = std::env::args().nth(1).expect("no file provided");

    let input = std::fs::read_to_string(&file).unwrap();
    let mut tokenizer = token::Tokenizer::new(&input, &file);

    tokenizer.parse();

    if std::env::args().any(|arg| arg.starts_with("--tokens=")) {
        let token = format!("{:#?}", tokenizer.tokens);
        let file = std::env::args()
            .find(|arg| arg.starts_with("--tokens="))
            .unwrap()[9..]
            .to_string();

        std::fs::write(&file, token).unwrap();
        println!("tokens written to {}", file);

        return;
    }

    let mut runtime = runtime::Runtime::new(tokenizer.tokens.clone());
    runtime.run();
}
