use std::env;

use crate::execute;
use crate::shell;
use crate::tools;

const DEFAULT_PROMPT: &str =
    "${COLOR_STATUS}$USER${RESET}@${COLOR_STATUS}$HOSTNAME${RESET}: ${COLOR_STATUS}$CWD${RESET}$ ";
use super::preset::apply_preset_item;
use super::preset::apply_pyenv;

fn is_prefix_char(c: char) -> bool {
    c == '[' || c == '{'
}

fn is_suffix_char(c: char) -> bool {
    c == ']' || c == '}'
}

fn is_prompt_item_char(c: char) -> bool {
    let s = c.to_string();
    tools::re_contains(&s, r#"^[a-zA-Z_]$"#)
}

pub fn get_prompt_string() -> String {
    if let Ok(x) = env::var("PROMPT") {
        return x;
    }
    DEFAULT_PROMPT.to_string()
}

fn apply_prompt_item(sh: &shell::Shell, result: &mut String, token: &str) {
    if let Ok(x) = env::var(token) {
        result.push_str(&x);
        return;
    }
    apply_preset_item(sh, result, token);
}

fn apply_command(result: &mut String, token: &str, prefix: &str, suffix: &str) {
    let cr = execute::run(&token);
    let output = cr.stdout.trim();
    if !output.is_empty() {
        result.push_str(&prefix);
        result.push_str(&output);
        result.push_str(&suffix);
    }
}

pub fn render_prompt(sh: &shell::Shell, ps: &str) -> String {
    let mut prompt = String::new();
    apply_pyenv(&mut prompt);

    let mut met_dollar = false;
    let mut met_brace = false;
    let mut met_paren = false;
    let mut token = String::new();
    let mut prefix = String::new();
    let mut suffix = String::new();
    for c in ps.chars() {
        if met_dollar {
            if c == '(' && !met_brace && !met_paren {
                met_paren = true;
                continue;
            }
            if c == ')' && met_paren {
                apply_command(&mut prompt, &token, &prefix, &suffix);
                token.clear();
                prefix.clear();
                suffix.clear();
                met_dollar = false;
                met_paren = false;
                continue;
            }
            if c == '{' && !met_brace && !met_paren {
                met_brace = true;
                continue;
            } else if c == '}' && met_brace {
                apply_prompt_item(sh, &mut prompt, &token);
                token.clear();
                met_dollar = false;
                met_brace = false;
                continue;
            } else if c == '$' {
                if token.is_empty() {
                    // to make single $ as a plain $
                    prompt.push('$');
                    met_dollar = true;
                    continue;
                } else {
                    apply_prompt_item(sh, &mut prompt, &token);
                    token.clear();
                    // met_dollar is still true
                    continue;
                }
            } else if met_paren {
                if is_prefix_char(c) {
                    prefix.push(c);
                } else if is_suffix_char(c) {
                    suffix.push(c);
                } else {
                    token.push(c);
                }
                continue;
            } else if is_prompt_item_char(c) {
                token.push(c);
                continue;
            } else if token.is_empty() {
                prompt.push('$');
                prompt.push(c);
                met_dollar = false;
                continue;
            }
        }

        if c == '$' {
            met_dollar = true;
            continue;
        }

        if !token.is_empty() {
            apply_prompt_item(sh, &mut prompt, &token);
            token.clear();
        }
        prompt.push(c);
        met_dollar = false;
    }

    if !token.is_empty() {
        apply_prompt_item(sh, &mut prompt, &token);
        met_dollar = false;
    }

    if met_dollar {
        // for cases like PROMPT='$$'
        prompt.push('$');
    }

    if prompt.trim().is_empty() {
        return format!("cicada-{} >> ", env!("CARGO_PKG_VERSION"));
    }
    prompt
}

#[cfg(test)]
mod tests {
    use super::render_prompt;
    use super::shell::Shell;

    #[test]
    fn test_render_prompt() {
        let mut sh = Shell::new();
        sh.set_env("USER", "mitnk");

        assert_eq!("mitnk$\n", render_prompt(&sh, "$USER$${newline}"));
        assert_eq!("mitnk$\n", render_prompt(&sh, "$USER$$newline"));

        assert_eq!("$", render_prompt(&sh, "$"));
        assert_eq!("$$", render_prompt(&sh, "$$"));
        assert_eq!("$$$", render_prompt(&sh, "$$$"));
        assert_eq!("$ ", render_prompt(&sh, "$ "));
        assert_eq!("$$ ", render_prompt(&sh, "$$ "));
        assert_eq!("$$$ ", render_prompt(&sh, "$$$ "));
    }
}
