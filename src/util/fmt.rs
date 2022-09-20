use std::fmt::{Debug, Error, Formatter};

pub fn write_comma_separated_list<E>(fmt: &mut Formatter, list: &Vec<E>) -> Result<(), Error> where E: Debug {
    if let Some(first) = list.first() {
        write!(fmt, "{:?}", first)?
    }
    for item in list.iter().skip(1) { write!(fmt, ", {:?}", item)? }
    Ok(())
}
