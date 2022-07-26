#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tenlang);
mod ast;

const PROGRAM: &str = "\
fn copy_3_times(a: Int32) -> Int32[3] {
    return [a, a, a];
}

fn square(_ a: Int32) {
    return a * a;
}

fn main() {
    var b = copy_3_times(a: 5 * 2 + 1);
    let b = b.square();
    print(b[0]);
}";

fn main() {
    let program = tenlang::ProgramParser::new()
        .parse(PROGRAM)
        .unwrap();

    println!("{:?}", program);
}
