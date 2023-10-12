use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};

pub fn fmta<F: Fn(&mut Formatter) -> std::fmt::Result>(fun: F) -> String {
    struct Mock<F: Fn(&mut Formatter) -> std::fmt::Result> {
        fun: F,
    }

    impl<F: Fn(&mut Formatter) -> std::fmt::Result> Display for Mock<F> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            (&self.fun)(f)
        }
    }

    format!("{}", Mock { fun })
}

pub fn write_space_separated_list<E>(fmt: &mut Formatter, list: &Vec<E>) -> Result<(), Error> where E: Display {
    if let Some(first) = list.first() {
        write!(fmt, "{}", first)?
    }
    for item in list.iter().skip(1) { write!(fmt, " {}", item)? }
    Ok(())
}

pub fn write_comma_separated_list<E>(fmt: &mut Formatter, list: &Vec<E>) -> Result<(), Error> where E: Display {
    if let Some(first) = list.first() {
        write!(fmt, "{}", first)?
    }
    for item in list.iter().skip(1) { write!(fmt, ", {}", item)? }
    Ok(())
}

pub fn write_comma_separated_list_debug<E>(fmt: &mut Formatter, list: &Vec<E>) -> Result<(), Error> where E: Debug {
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
