use std::fmt;

#[derive(Debug, PartialEq, Clone)]
// Tagged or terminal
enum ToT {
    String(String),
    Tagged(Tagged),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Tagged {
    label: String,
    prefix: Vec<ToT>,
    instructions: Vec<ToT>,
}

impl fmt::Display for Tagged {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tagged {{ label: \"{}\", prefix: {:#?}, instructions: {:#?} }}",
            self.label, self.prefix, self.instructions
        )
    }
}

fn parse_sexp(input: &str, start: usize) -> (String, usize) {
    let mut buf = String::new();
    let mut level = 0;
    let mut index = start;

    while index < input.len() {
        let c = input.chars().nth(index).unwrap();
        index += 1;

        match c {
            '(' => {
                level += 1;
                buf.push(c);
            }
            ')' => {
                level -= 1;
                if level == 0 {
                    buf.push(c);
                    break;
                } else {
                    buf.push(c);
                }
            }
            _ => {
                buf.push(c);
            }
        }
    }

    (buf, index)
}

fn parse_tagged(input: &str) -> Result<Tagged, String> {
    let mut label = String::new();
    let mut prefix = Vec::new();
    let mut instructions = Vec::new();
    let mut buf = String::new();
    let mut instr_mode = false;

    let mut index = 0;
    if input.chars().nth(index) != Some('(') {
        return Err("Expected opening parenthesis".to_string());
    }
    index += 1;

    while index < input.len() {
        let c = input.chars().nth(index).unwrap();
        index += 1;

        match c {
            ')' => {
                if !buf.is_empty() {
                    instructions.push(buf.clone().trim().to_string());
                    buf.clear();
                }
                break;
            }
            ' ' | '\t' | '\n' => {
                if !buf.is_empty() {
                    if label.is_empty() {
                        label = buf.clone();
                        buf.clear();
                    } else {
                        if prefix.is_empty() {
                            prefix.push(buf.clone().trim().to_string());
                            buf.clear();
                        } else {
                            //dbg!(&(input.to_string())[index..]);
                            //dbg!(input[index..].contains('('));
                            if !instr_mode {
                                let sexp_ahead = input[index..].contains('(');

                                if sexp_ahead {
                                    prefix.push(buf.clone().trim().to_string());
                                    buf.clear();
                                } else {
                                    instr_mode = true;
                                    if c == '\n' {
                                        instructions.push(buf.clone().trim().to_string());
                                        buf.clear();
                                    } else {
                                        buf.push(c);
                                        continue;
                                    }
                                }
                            } else {
                                if c == '\n' {
                                    instructions.push(buf.clone().trim().to_string());
                                    buf.clear();
                                } else {
                                    buf.push(c);
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            '(' => {
                //println!("* * *");
                //dbg!(index);
                let (sexp, new_index) = parse_sexp(input, index - 1);
                //dbg!(sexp.clone());
                //dbg!(new_index);
                //println!("* * *");
                index = new_index;
                prefix.push(sexp);
            }
            _ => {
                buf.push(c);
            }
        }
    }

    if label.is_empty() {
        return Err("Missing label".to_string());
    }

    Ok(Tagged {
        label,
        prefix: prefix.iter().map(|s| ToT::String(s.clone())).collect(),
        instructions: instructions
            .iter()
            .map(|s| ToT::String(s.clone()))
            .collect(),
    })
}

pub fn unfold_funcs(x: Tagged) -> Tagged {
    let mut new_prefix = Vec::new();

    for p in x.prefix {
        match p {
            ToT::String(s) => {
                // If the prefix is a function call, unfold it.
                if s.starts_with("(func ") {
                    let func = parse_tagged(&s).unwrap();
                    let unfolded = unfold_funcs(func);
                    new_prefix.push(ToT::Tagged(unfolded));
                } else {
                    new_prefix.push(ToT::String(s));
                }
            }
            ToT::Tagged(t) => new_prefix.push(ToT::Tagged(t)),
        }
    }

    Tagged {
        label: x.label,
        prefix: new_prefix,
        instructions: x.instructions,
    }
}

// SCOPEY_INSTRUCTIONS are, for the time being just vec!["block", "loop"].
const SCOPEY_INSTRUCTIONS: [&str; 2] = ["block", "loop"];

// Now we process SCOPEY_INSTRUCTIONS symbolicly. An instruction `"s ..."` becomes `"(s ..."` and
// "end" becomes `")"`.
pub fn replace_scopey(x: Tagged) -> Tagged {
    let mut new_instructions = Vec::new();

    for i in x.instructions {
        match i {
            ToT::String(s) => {
                // Check if the instruction starts with substring that is scopey.
                if SCOPEY_INSTRUCTIONS
                    .iter()
                    .any(|&scopey| s.starts_with(scopey))
                {
                    new_instructions.push(ToT::String(format!("({}", s)));
                } else if s == "end" {
                    new_instructions.push(ToT::String(")".to_string()));
                } else {
                    new_instructions.push(ToT::String(s));
                }
            }
            // We don't really have nested instructions, but if we did we would have had to recur
            // here. TODO
            ToT::Tagged(t) => new_instructions.push(ToT::Tagged(t)),
        }
    }

    Tagged {
        label: x.label,
        prefix: x.prefix,
        instructions: new_instructions,
    }
}

fn main() {
    // Get input from file "./output.wat", which is the output of preprocessing.
    let input = std::fs::read_to_string("./output.wat").unwrap();
    // TODO: function that manually iterates over Tagged and replaces scopeys for each underlying
    // Tagged found.
    let parsed: Tagged = unfold_funcs(parse_tagged(&input).unwrap());
    println!("{}", parsed)
}
