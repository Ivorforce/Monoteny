#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tenlang);
mod ast;

const PROGRAM: &str = "\
fn square() {
    return 2 * 2;
}

fn main() {
    let a = 2 * 3;
    var b = 5;
}";

fn main() {
    let program = tenlang::ProgramParser::new()
        .parse(PROGRAM)
        .unwrap();

    println!("{:?}", program);
}
