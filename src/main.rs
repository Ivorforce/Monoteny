#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tenlang);
mod ast;

const PROGRAM: &str = "\
fn square(a: Int32) {
    return a * a;
}

fn main() {
    let a = 1 + 2 * 3;
    var b = square(a: a);
}";

fn main() {
    let program = tenlang::ProgramParser::new()
        .parse(PROGRAM)
        .unwrap();

    println!("{:?}", program);
}
