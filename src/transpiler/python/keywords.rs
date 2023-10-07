use std::collections::HashMap;
use lazy_static::lazy_static;
use uuid::Uuid;
use crate::transpiler::namespaces;

lazy_static! {
    pub static ref KEYWORD_IDS: HashMap<&'static str, Uuid> = HashMap::from([
        ("class", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8001)),
        ("def", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8002)),
        ("from", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8003)),
        ("continue", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8004)),
        ("global", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8005)),
        ("pass", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8006)),
        ("if", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8007)),
        ("raise", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8008)),
        ("del", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8009)),
        ("import", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8010)),
        ("return", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8011)),
        ("as", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8012)),
        ("elif", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8013)),
        ("in", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8014)),
        ("try", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8015)),
        ("assert", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8016)),
        ("else", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8017)),
        ("is", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8018)),
        ("while", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8019)),
        ("async", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8020)),
        ("except", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8021)),
        ("lambda", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8022)),
        ("with", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8023)),
        ("await", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8024)),
        ("finally", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8025)),
        ("nonlocal", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8026)),
        ("yield", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8027)),
        ("break", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8028)),
        ("for", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8029)),

        ("==", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8030)),
        ("!=", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8031)),
        (">", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8032)),
        ("<", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8033)),
        (">=", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8034)),
        ("<=", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8035)),
        ("and", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8036)),
        ("or", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8037)),
        ("not", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8038)),
        ("-", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8039)),
        ("+", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8040)),
        ("*", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8041)),
        ("**", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8042)),
        ("/", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8043)),
        ("//", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8044)),
        ("%", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8045)),
        ("&", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8046)),
        ("^", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8047)),

        ("False", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8048)),
        ("True", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8049)),
        ("None", Uuid::from_u128(0x376e916d_3ca1_4d90_a931_789f911b8050)),
    ]);
}

pub fn register(namespace: &mut namespaces::Level) {
    // Keywords
    for (keyword, id) in KEYWORD_IDS.iter() {
        // Don't really need an ID but it's easy to just do it like this here.
        namespace.insert_fixed_name(*id, keyword);
    }
}
