pub fn preprocess(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();

    let min_indent = lines
        .iter()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    let mut result: Vec<String> = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];
    let mut expecting_indent = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let raw_indent = line.len() - line.trim_start().len();
        let indent = raw_indent.saturating_sub(min_indent);

        if expecting_indent {
            if indent > *indent_stack.last().unwrap() {
                indent_stack.push(indent);
            }
            expecting_indent = false;
        }

        while indent < *indent_stack.last().unwrap() {
            indent_stack.pop();
            result.push("}".to_string());
        }

        if is_block_opener(trimmed) {
            let without_colon = &trimmed[..trimmed.len() - 1];
            result.push(format!("{} {{", without_colon));
            expecting_indent = true;
        } else {
            result.push(trimmed.to_string());
        }
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        result.push("}".to_string());
    }

    result.join("\n")
}

fn is_block_opener(line: &str) -> bool {
    if !line.ends_with(':') {
        return false;
    }
    line.starts_with("if ")
        || line.starts_with("else if ")
        || line == "else:"
        || line.starts_with("while ")
        || line.starts_with("for ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_if_else() {
        let input = r#"
            if x > 0:
                print "yes"
            else:
                print "no"
        "#;
        let output = preprocess(input);
        assert!(output.contains("if x > 0 {"));
        assert!(output.contains("else {"));
        assert_eq!(output.matches('}').count(), 2);
    }

    #[test]
    fn test_while_loop() {
        let input = r#"
            while x > 0:
                print x
                x = x - 1
        "#;
        let output = preprocess(input);
        assert!(output.contains("while x > 0 {"));
        assert_eq!(output.matches('}').count(), 1);
    }

    #[test]
    fn test_for_loop() {
        let input = r#"
            for i in 0..5:
                print i
        "#;
        let output = preprocess(input);
        assert!(output.contains("for i in 0..5 {"));
        assert_eq!(output.matches('}').count(), 1);
    }

    #[test]
    fn test_nested_blocks() {
        let input = r#"
            for i in 0..3:
                for j in 0..3:
                    print i + j
        "#;
        let output = preprocess(input);
        assert!(output.contains("for i in 0..3 {"));
        assert!(output.contains("for j in 0..3 {"));
        assert_eq!(output.matches('}').count(), 2);
    }

    #[test]
    fn test_else_if() {
        let input = r#"
            if x > 5:
                print "big"
            else if x > 0:
                print "positive"
            else:
                print "other"
        "#;
        let output = preprocess(input);
        assert!(output.contains("if x > 5 {"));
        assert!(output.contains("else if x > 0 {"));
        assert!(output.contains("else {"));
        assert_eq!(output.matches('}').count(), 3);
    }
}
