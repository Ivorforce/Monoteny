#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tenlang);
mod ast;

const PROGRAM: &str = "\
fn square(a: Int32) {
    return a * a;
}

fn main() {
    let a: Int32[3] = [1 + 2, 5 * 3, 2];
    var b = square(a: a);
    let b = b.square();
    print(b);
}";

fn main() {
    let program = tenlang::ProgramParser::new()
        .parse(PROGRAM)
        .unwrap();

    println!("{:?}", program);
}
