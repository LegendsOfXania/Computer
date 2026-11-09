#[derive(Clone, Copy)]
pub struct Operator {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub takes_value: bool,
    pub completion_colon: bool,
}

pub const OPERATORS: &[Operator] = &[
    Operator { name: "entry", aliases: &["e"], takes_value: true, completion_colon: false },
    Operator { name: "page", aliases: &["p"], takes_value: true, completion_colon: false },
    Operator { name: "ne", aliases: &[], takes_value: true, completion_colon: false },
    Operator { name: "ee", aliases: &[], takes_value: true, completion_colon: false },
    Operator { name: "id", aliases: &[], takes_value: false, completion_colon: false },
    Operator { name: "kind", aliases: &["k", "type"], takes_value: true, completion_colon: true },
    Operator { name: "extension", aliases: &["ext"], takes_value: true, completion_colon: true },
    Operator { name: "chapter", aliases: &["c"], takes_value: true, completion_colon: true },
];

fn find(name: &str) -> Option<&'static Operator> {
    OPERATORS.iter().find(|operator| {
        operator.name.eq_ignore_ascii_case(name)
            || operator.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name))
    })
}

pub fn is_operator(token: &str) -> bool {
    let Some(token) = token.strip_prefix('!') else {
        return false;
    };
    let (name, _) = token.split_once(':').unwrap_or((token, ""));
    let has_value_separator = token.contains(':');
    find(name).is_some_and(|operator| {
        if operator.takes_value { true } else { !has_value_separator }
    })
}

pub fn complete_operator(input: &str) -> Option<String> {
    let start = input.rfind(char::is_whitespace).map_or(0, |index| index + 1);
    let token = &input[start..];
    let partial = token.strip_prefix('!')?;

    if partial.is_empty() || partial.contains(':') {
        return None;
    }

    let partial = partial.to_ascii_lowercase();
    let matches: Vec<&Operator> = OPERATORS
        .iter()
        .filter(|operator| {
            operator.name.starts_with(partial.as_str())
                || operator.aliases.iter().any(|alias| alias.starts_with(partial.as_str()))
        })
        .collect();

    if matches.len() != 1 {
        return None;
    }

    let operator = matches[0];
    let completion = if operator.completion_colon {
        format!("!{}:", operator.name)
    } else {
        format!("!{}", operator.name)
    };

    if token.eq_ignore_ascii_case(completion.as_str()) {
        return None;
    }

    Some(format!("{}{}", &input[..start], completion))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_common_operator_prefixes() {
        assert_eq!(complete_operator("!en").as_deref(), Some("!entry"));
        assert_eq!(complete_operator("!pa").as_deref(), Some("!page"));
        assert_eq!(complete_operator("!ex").as_deref(), Some("!extension:"));
        assert_eq!(complete_operator("!ki").as_deref(), Some("!kind:"));
        assert_eq!(complete_operator("!ch").as_deref(), Some("!chapter:"));
    }

    #[test]
    fn completes_only_the_current_unfinished_operator() {
        assert_eq!(complete_operator("hello !en").as_deref(), Some("hello !entry"));
        assert_eq!(complete_operator("!chapter:Act").as_deref(), None);
        assert_eq!(complete_operator("!e").as_deref(), None);
    }

    #[test]
    fn recognizes_only_supported_operator_arguments() {
        assert!(is_operator("!entry:name"));
        assert!(is_operator("!kind:message"));
        assert!(!is_operator("!id:message"));
    }
}
