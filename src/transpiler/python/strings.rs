use crate::util::strings;

pub fn escape_string(string: &str) -> String {
    strings::map_chars(string, |ch| {
        Some(match ch {
            '\\' => "\\\\",
            '\n' => "\\n",
            '\0' => "\\0",
            '\t' => "\\t",
            '\r' => "\\r",
            '\"' => "\\\"",
            _ => return None,
        })
    })
}
