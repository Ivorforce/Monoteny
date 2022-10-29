use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};

pub fn write_space_separated_list<E>(fmt: &mut Formatter, list: &Vec<E>) -> Result<(), Error> where E: Debug {
    if let Some(first) = list.first() {
        write!(fmt, "{:?}", first)?
    }
    for item in list.iter().skip(1) { write!(fmt, " {:?}", item)? }
    Ok(())
}

pub fn write_comma_separated_list<E>(fmt: &mut Formatter, list: &Vec<E>) -> Result<(), Error> where E: Debug {
    if let Some(first) = list.first() {
        write!(fmt, "{:?}", first)?
    }
    for item in list.iter().skip(1) { write!(fmt, ", {:?}", item)? }
    Ok(())
}

pub fn write_keyval<K, V>(fmt: &mut Formatter, mapping: &HashMap<K, V>) -> Result<(), Error> where K: Debug, V: Debug {
    let mut iterator = mapping.iter();

    if let Some((key, val)) = iterator.next() {
        write!(fmt, "{:?}: {:?}", key, val)?
    }
    for (key, val) in iterator.skip(1) {
        write!(fmt, ", {:?}: {:?}", key, val)?
    }

    Ok(())
}
