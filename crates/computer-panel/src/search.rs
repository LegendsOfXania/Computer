use frizbee::{Config, Matcher, Pattern};

pub const PAGE: u8 = 1;
pub const ENTRY: u8 = 2;
pub const NEW_ENTRY: u8 = 4;

const ANY: usize = 0;
const NAME: usize = 1;
const KIND: usize = 2;
const ID: usize = 3;

struct Document<T> {
    target: T,
    scope: u8,
    fields: [String; 4],
}

pub struct SearchIndex<T>(Vec<Document<T>>);

impl<T> SearchIndex<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_page(&mut self, target: T, name: &str, kind: &str, id: &str, chapter: Option<&str>) {
        self.add(PAGE, target, name, kind, id, chapter.unwrap_or(""));
    }

    pub fn add_entry(
        &mut self,
        target: T,
        name: &str,
        kind: &str,
        id: &str,
        parent_page: Option<&str>,
    ) {
        self.add(ENTRY, target, name, kind, id, parent_page.unwrap_or(""));
    }

    pub fn add_new_entry(&mut self, target: T, kind: &str, tags: &[String]) {
        self.add(NEW_ENTRY, target, kind, kind, "", &tags.join(" "));
    }

    fn add(&mut self, scope: u8, target: T, name: &str, kind: &str, id: &str, extra: &str) {
        let any = format!("{name} {kind} {id} {extra}");
        let fields = [any, name.into(), kind.into(), id.into()];
        self.0.push(Document {
            target,
            scope,
            fields,
        });
    }

    pub fn search(&self, query: &str, default: u8) -> Vec<&T> {
        let query = Query::parse(query, default);

        let documents: Vec<&Document<T>> = self
            .0
            .iter()
            .filter(|d| d.scope & query.scope != 0)
            .filter(|d| query.kind.as_ref().is_none_or(|k| *k == d.fields[KIND]))
            .collect();

        if query.fuzzy.is_empty() {
            return documents.into_iter().map(|d| &d.target).collect();
        }

        let patterns: Vec<Pattern> = Pattern::parse_query(&query.fuzzy)
            .into_iter()
            .map(|p| {
                let typos = (p.needle.chars().count() / 4).min(2) as u16;
                p.max_typos(Some(typos))
            })
            .collect();

        let haystacks: Vec<&str> = documents
            .iter()
            .map(|d| d.fields[query.field].as_str())
            .collect();
        Matcher::from_patterns(&patterns, &Config::default())
            .match_list(&haystacks)
            .into_iter()
            .map(|m| &documents[m.index as usize].target)
            .collect()
    }
}

pub fn scope(query: &str, default: u8) -> u8 {
    Query::parse(query, default).scope
}

struct Query {
    scope: u8,
    field: usize,
    kind: Option<String>,
    fuzzy: String,
}

impl Query {
    fn parse(input: &str, default: u8) -> Self {
        let mut scope = !0;
        let mut field = ANY;
        let mut kind = None;
        let mut fuzzy = Vec::new();

        for token in split_query(input) {
            let (op, arg) = match token.strip_prefix('!') {
                Some(rest) => rest.split_once(':').unwrap_or((rest, "")),
                None => ("", ""),
            };
            match (op, arg) {
                ("page", "") => scope &= PAGE,
                ("entry", "") => scope &= ENTRY,
                ("id", "") => field = ID,
                ("kind" | "k", "") => {
                    scope &= ENTRY;
                    field = KIND;
                }
                ("page" | "entry", "name" | "kind" | "id") => {
                    scope &= if op == "page" { PAGE } else { ENTRY };
                    field = match arg {
                        "name" => NAME,
                        "kind" => KIND,
                        _ => ID,
                    };
                }
                ("ne" | "ee", _) => {
                    scope &= if op == "ne" { NEW_ENTRY } else { ENTRY };
                    if !arg.is_empty() {
                        kind = Some(arg.to_owned());
                    }
                }
                _ => fuzzy.push(token),
            }
        }

        if scope == !0 {
            scope = default;
        }
        Self {
            scope,
            field,
            kind,
            fuzzy: fuzzy.join(" "),
        }
    }
}

fn split_query(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut escaped = false;
    for c in input.chars() {
        if c.is_whitespace() && !escaped {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
        } else {
            token.push(c);
            escaped = c == '\\' && !escaped;
        }
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}
