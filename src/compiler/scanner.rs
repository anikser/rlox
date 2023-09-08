pub struct Scanner {
    source: String,
    current: usize,
    line: u32,
}

impl Scanner {
    pub fn init(source: String) -> Self {
        Self {
            source: source,
            current: 0,
            line: 1,
        }
    }
}
