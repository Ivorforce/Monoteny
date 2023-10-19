pub fn map_chars(string: &str, fun: impl Fn(char) -> Option<&'static str>) -> String {
    let mut output = String::with_capacity(string.len());
    for char in string.chars() {
        if let Some(map) = fun(char) {
            output.push_str(map);
        }
        else {
            output.push(char);
        }
    }
    output
}
