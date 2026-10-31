p = 'crates/awk/src/main.rs'
s = open(p, encoding='utf-8').read()

# 1. Token enum: Getline
old = "    Delete,\n    In,"
new = "    Delete,\n    In,\n    Getline,"
assert old in s, "enum"
s = s.replace(old, new)

# 2. tokenizer keyword
old2 = '                    "delete" => tokens.push(Token::Delete),'
new2 = '                    "delete" => tokens.push(Token::Delete),\n                    "getline" => tokens.push(Token::Getline),'
assert old2 in s, "tokenizer"
s = s.replace(old2, new2)

# 3. thread_local: getline sources
old3 = 'type FunctionTable ='
new3 = '''type GetlineSources = std::cell::RefCell<HashMap<String, (Vec<String>, usize)>>;

'''
assert old3 in s, "type alias"
s = s.replace(old3, new3)

# add to thread_local block (after RETURN_VALUE)
old4 = '    static RETURN_VALUE: std::cell::RefCell<Value> ='
new4 = '    static GETLINE_SOURCES: GetlineSources =\n        std::cell::RefCell::new(HashMap::new());\n    static RETURN_VALUE: std::cell::RefCell<Value> ='
assert old4 in s, "thread_local"
s = s.replace(old4, new4)

# 4. eval_tokens: handle Getline token
# Insert before the In arm
old5 = "            Token::In => {"
new5 = """            Token::Getline => {
                // getline [var] [< file]
                // Parse the optional target variable.
                let mut target: Option<String> = None;
                if let Some(Token::Var(v)) = tokens.get(index + 1) {
                    if !v.starts_with('$') && v != "in" {
                        target = Some(v.clone());
                        index += 1;
                    }
                }
                index += 1; // skip the Getline token itself

                // Check for `< file` redirect.
                let (result, line) = if matches!(tokens.get(index), Some(Token::Op(op)) if op == "<")
                {
                    index += 1;
                    // The filename expression extends until a non-value token.
                    let end = expr_end(tokens, index);
                    let file = eval_tokens(&tokens[index..end], vars).to_str();
                    index = end;
                    getline_from_file(&file)
                } else {
                    // Bare getline from main input: use NR+1 to peek.
                    getline_main(vars)
                };

                // Bind the target variable or set $0.
                if let Some(var) = target {
                    vars.insert(var, line);
                } else {
                    vars.insert("$0".to_string(), line);
                    // Re-split fields for the new $0.
                    let fs = vars.get("FS").cloned().unwrap_or_else(|| " ".to_string());
                    let fields: Vec<&str> = if fs == " " {
                        line.split_whitespace().collect()
                    } else {
                        line.split(&fs).collect()
                    };
                    vars.retain(|k, _| !(k.starts_with('$') && k != "$0"));
                    vars.insert("NF".to_string(), fields.len().to_string());
                    for (j, f) in fields.iter().enumerate() {
                        vars.insert(format!("${}", j + 1), (*f).to_string());
                    }
                }

                last_was_value = true;
                stack.push(result);
            }
            Token::In => {"""
assert old5 in s, "eval getline"
s = s.replace(old5, new5)

# 5. getline helper functions
old6 = "/// Dispatch one builtin function call."
new6 = """/// getline from a file: returns (return_code, line_read).
fn getline_from_file(path: &str) -> (Value, String) {
    GETLINE_SOURCES.with(|src| {
        let mut map = src.borrow_mut();
        if !map.contains_key(path) {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let lines: Vec<String> = content
                        .lines()
                        .map(|l| l.to_string())
                        .collect();
                    map.insert(path.to_string(), (lines, 0));
                }
                Err(_) => return (Value::Num(-1.0), String::new()),
            }
        }
        let (lines, pos) = map.get_mut(path).unwrap();
        if *pos < lines.len() {
            let line = lines[*pos].clone();
            *pos += 1;
            (Value::Num(1.0), line)
        } else {
            (Value::Num(0.0), String::new())
        }
    })
}

/// Bare getline from the main input: reads the next record.
/// Uses a thread-local cursor that tracks position independently of NR.
thread_local! {
    static MAIN_INPUT_LINES: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static MAIN_INPUT_POS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn getline_main(vars: &mut HashMap<String, String>) -> (Value, String) {
    MAIN_INPUT_LINES.with(|lines| {
        let mut lines = lines.borrow_mut();
        let pos = MAIN_INPUT_POS.with(|p| p.get());
        if pos < lines.len() {
            let line = lines[pos].clone();
            MAIN_INPUT_POS.with(|p| p.set(pos + 1));
            (Value::Num(1.0), line)
        } else {
            (Value::Num(0.0), String::new())
        }
    })
}

/// Dispatch one builtin function call."""
assert old6 in s, "getline helpers"
s = s.replace(old6, new6)

# 6. close builtin: also clear getline sources for the closed file
old7 = '''                    if v == "close" {
                        if let Some(target) = args.first() {
                            let name = target.to_str();
                            let removed = REDIRECTS.with(|r| {
                                r.borrow_mut().remove(&name).map(|mut h| {
                                    h.close();
                                })
                            });
                            stack.push(Value::Num(if removed.is_some() { 0.0 } else { -1.0 }));'''
new7 = '''                    if v == "close" {
                        if let Some(target) = args.first() {
                            let name = target.to_str();
                            GETLINE_SOURCES.with(|g| {
                                g.borrow_mut().remove(&name);
                            });
                            let removed = REDIRECTS.with(|r| {
                                r.borrow_mut().remove(&name).map(|mut h| {
                                    h.close();
                                })
                            });
                            stack.push(Value::Num(if removed.is_some() { 0.0 } else { -1.0 }));'''
assert old7 in s, "close getline"
s = s.replace(old7, new7)

# 7. process_input: store lines in MAIN_INPUT_LINES for bare getline
old8 = '''    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        lines.push(line.trim_end().to_string());
        line.clear();
    }'''
new8 = '''    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        lines.push(line.trim_end().to_string());
        line.clear();
    }

    MAIN_INPUT_LINES.with(|l| {
        *l.borrow_mut() = lines.clone();
    });
    MAIN_INPUT_POS.with(|p| p.set(0));'''
assert old8 in s, "process_input"
s = s.replace(old8, new8)

open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('getline ok')
