/// #Description
/// Convert a snake case string to pascal case
pub fn convert_snake_to_pascal(input: &str) -> String {
    // We will over allocate here but not by much in most cases
    let mut ret = String::with_capacity(input.len());
    let mut chars = input.chars();

    // Ensure the first character is upper case
    if let Some(c) = chars.next() {
        ret.push_str(&c.to_uppercase().to_string());
    } else {
        return ret;
    };

    while let Some(c) = chars.next() {
        if c == '_' {
            if let Some(next_char) = chars.next() {
                ret.push_str(&next_char.to_uppercase().to_string())
            } else {
                break;
            }
        } else {
            ret.push(c)
        }
    }
    ret
}
