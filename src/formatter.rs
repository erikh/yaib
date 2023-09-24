pub type Rules<'a> = Vec<(&'a str, String)>;

pub struct Format<'a> {
    format: String,
    rules: Rules<'a>,
}

impl<'a> Format<'a> {
    pub fn new(format: String, rules: Rules<'a>) -> Self {
        Self { format, rules }
    }

    pub fn format(&self) -> String {
        let mut res = self.format.clone();

        for rule in &self.rules {
            res = res.replace(rule.0, &rule.1);
        }

        res
    }
}
