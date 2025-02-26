pub mod array;
pub mod fs;
pub mod io;
pub mod logic;
pub mod math;
pub mod rng;
pub mod string;
pub mod tcp;
pub mod time;

use super::logic::ExpressionToken;
use crate::runtime::Runtime;

use std::sync::{Arc, LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    let mut vec = Vec::new();

    vec.extend(&*io::FUNCTIONS);
    vec.extend(&*string::FUNCTIONS);
    vec.extend(&*fs::FUNCTIONS);
    vec.extend(&*math::FUNCTIONS);
    vec.extend(&*array::FUNCTIONS);
    vec.extend(&*logic::FUNCTIONS);
    vec.extend(&*time::FUNCTIONS);
    vec.extend(&*rng::FUNCTIONS);
    vec.extend(&*tcp::FUNCTIONS);

    vec
});

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    if io::FUNCTIONS.contains(&name) {
        io::run(name, args, runtime)
    } else if string::FUNCTIONS.contains(&name) {
        string::run(name, args, runtime)
    } else if fs::FUNCTIONS.contains(&name) {
        fs::run(name, args, runtime)
    } else if math::FUNCTIONS.contains(&name) {
        math::run(name, args, runtime)
    } else if array::FUNCTIONS.contains(&name) {
        array::run(name, args, runtime)
    } else if logic::FUNCTIONS.contains(&name) {
        logic::run(name, args, runtime)
    } else if time::FUNCTIONS.contains(&name) {
        time::run(name, args, runtime)
    } else if rng::FUNCTIONS.contains(&name) {
        rng::run(name, args, runtime)
    } else if tcp::FUNCTIONS.contains(&name) {
        tcp::run(name, args, runtime)
    } else {
        None
    }
}
