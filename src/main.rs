#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tenlang);
mod ast;

const PROGRAM: &str = "\
fn a() {
    let a = 22 * 44 +  66;
    var b = 10;
}";

fn main() {
    let statements = tenlang::ProgramParser::new()
        .parse(PROGRAM)
        .unwrap();

    assert_eq!(
        &format!("{:?}", statements),
        "[fn a() {[let a = ((22 * 44) + 66), var b = 10]}]"
    );
}
