pub fn remove_brackets(input: &str) -> String {
    let mut result = String::new();
    let mut inside_brackets = false;

    for c in input.chars() {
        if c == '[' {
            inside_brackets = true;
        } else if c == ']' {
            inside_brackets = false;
        } else if !inside_brackets {
            result.push(c);
        }
    }

    result
}
