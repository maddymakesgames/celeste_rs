use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookupIndex(pub(crate) u16);

#[derive(Debug)]
pub struct LookupTable {
    lookup_strings: Vec<String>,
    strings_to_add: Vec<String>,
}

impl LookupTable {
    pub fn new() -> LookupTable {
        LookupTable {
            lookup_strings: Vec::new(),
            strings_to_add: Vec::new(),
        }
    }

    fn binary_search(&self, str: &str) -> Result<usize, usize> {
        self.lookup_strings
            .binary_search_by(|a| str.cmp(a).reverse())
    }

    pub fn from_vec(vec: Vec<String>) -> LookupTable {
        LookupTable {
            lookup_strings: vec,
            strings_to_add: Vec::new(),
        }
    }

    pub fn lookup_contains(&self, str: impl AsRef<str>) -> bool {
        let str = str.as_ref();
        self.binary_search(str).is_ok()
    }

    pub fn add_string(&mut self, str: impl AsRef<str>) -> ResolvableString {
        let str = str.as_ref();
        if !self.lookup_contains(str) {
            if let Some(index) = self.strings_to_add.iter().position(|s| s == str) {
                if let Err(idx) = self.binary_search(str) {
                    let str = self.strings_to_add.swap_remove(index);
                    self.lookup_strings.insert(idx, str);
                }
            } else {
                self.strings_to_add.push(str.to_owned());
            }
        }

        ResolvableString::String(str.to_owned())
    }

    pub fn index_string(&mut self, str: impl AsRef<str>) -> ResolvableString {
        let str = str.as_ref();
        if let Err(idx) = self.binary_search(str) {
            if let Some(idx_to_remove) = self.strings_to_add.iter().position(|s| s == str) {
                self.strings_to_add.remove(idx_to_remove);
            }

            self.lookup_strings.insert(idx, str.to_owned());
        }

        ResolvableString::String(str.to_owned())
    }

    /// Resolves a string to it's lookup position if it exists
    ///
    /// Any indecies returned from this become invalid if [add_string](Self::add_string) is run again
    pub fn lookup_string(&self, str: impl AsRef<str>) -> Option<LookupIndex> {
        let str = str.as_ref();
        if self.lookup_contains(str) {
            self.lookup_strings
                .iter()
                .position(|s| s == str)
                .map(|i| LookupIndex(i as u16))
        } else {
            None
        }
    }

    pub(crate) fn to_string(&self, depth: u8) -> String {
        let mut buf = String::new();
        buf.push('[');

        for str in &self.lookup_strings {
            buf.push('\n');
            for _ in 0 .. depth + 1 {
                buf.push('\t');
            }
            buf.push('"');
            buf.push_str(str);
            buf.push('"');
        }

        if !self.lookup_strings.is_empty() {
            buf.push('\n');
            for _ in 0 .. depth {
                buf.push('\t');
            }
        }

        buf.push(']');

        buf
    }

    pub fn get(&self, index: LookupIndex) -> Option<&String> {
        self.lookup_strings.get(index.0 as usize)
    }
}

impl Index<LookupIndex> for LookupTable {
    type Output = String;

    fn index(&self, index: LookupIndex) -> &Self::Output {
        &self.lookup_strings[index.0 as usize]
    }
}

impl Default for LookupTable {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum ResolvableString {
    LookupIndex(LookupIndex),
    String(String),
}

impl ResolvableString {
    pub fn to_string<'a>(&'a self, lookup_table: &'a LookupTable) -> &'a str {
        match &self {
            ResolvableString::LookupIndex(i) => &lookup_table[*i],
            ResolvableString::String(s) => s,
        }
    }

    pub fn resolve(&mut self, lookup_table: &LookupTable) {
        match self {
            ResolvableString::LookupIndex(i) =>
                *self = ResolvableString::String(lookup_table[*i].clone()),
            ResolvableString::String(_) => {}
        }
    }

    pub fn to_index(&mut self, lookup_table: &LookupTable) {
        match self {
            ResolvableString::LookupIndex(_) => {}
            ResolvableString::String(s) =>
                if let Some(idx) = lookup_table.lookup_string(s) {
                    *self = ResolvableString::LookupIndex(idx);
                },
        }
    }
}
