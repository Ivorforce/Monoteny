#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tenlang);
mod ast;

const PROGRAM: &str = "\
fn square() {
    let a = 22 * 44 +  66;
    var b = 10;
};

fn main() {
    let a = 2 * 44 +  66;
    var b = 5;
}";

fn main() {
    let program = tenlang::ProgramParser::new()
        .parse(PROGRAM)
        .unwrap();

    println!("{:?}", program);
}
