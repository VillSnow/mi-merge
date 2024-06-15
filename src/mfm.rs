use fancy_regex::Regex;

pub enum DecomposedTextItem<'a> {
    Text(&'a str),
    Emoji(&'a str), // "icon_syuilo", not ":icon_syuilo:"
}

pub struct Decomposer {
    re: Regex,
}

impl Decomposer {
    pub fn new() -> Self {
        Self {
            re: Regex::new(r":(\w+):(?=\W|$)").unwrap(),
        }
    }

    pub fn decompose<'a>(&self, mut s: &'a str) -> Vec<DecomposedTextItem<'a>> {
        let mut result = Vec::new();
        while let Ok(Some(m)) = self.re.find(s) {
            if m.start() != 0 {
                result.push(DecomposedTextItem::Text(&s[..m.start()]))
            }

            result.push(DecomposedTextItem::Emoji(&s[m.start() + 1..m.end() - 1]));

            s = &s[m.end()..];
        }

        if !s.is_empty() {
            result.push(DecomposedTextItem::Text(s))
        }

        result
    }
}
