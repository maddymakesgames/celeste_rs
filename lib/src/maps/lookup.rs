use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// An index into a [LookupTable]
pub struct LookupIndex(pub(crate) u16);

#[derive(Debug)]
/// A lookup table holding all the strings for [ResolvableString]s
pub struct LookupTable {
    pub(super) lookup_strings: Vec<String>,
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

    /// Creates a new lookup table containing the strings in the passed `Vec`.
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

    /// Register the string in the lookup table.
    ///
    /// This does not necessarily mean the string will be in the lookup table during serialization.<br>
    /// We only actually store a string in the lookup table if it is used more than once.
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

    /// Puts the string into the lookup table.
    ///
    /// Unlike [add_string](Self::add_string) strings passed will always be included in the lookup table.
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

    /// Resolves a [LookupIndex] into a string if the index is valid
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
/// A string that can be included in the lookup table
///
/// Needs to be resolved before you can actually get the [String]
pub enum ResolvableString {
    LookupIndex(LookupIndex),
    String(String),
}

impl ResolvableString {
    /// Attempts to return the string, failing if it is unresolved.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ResolvableString::LookupIndex(_) => None,
            ResolvableString::String(s) => Some(s),
        }
    }

    /// Gets the string, looking into the lookup table if necessary.
    ///
    /// Does not resolve the string, ideally call [resolve](Self::resolve) and then [as_str](Self::as_str)
    pub fn to_string<'a>(&'a self, lookup_table: &'a LookupTable) -> &'a str {
        match &self {
            ResolvableString::LookupIndex(i) => &lookup_table[*i],
            ResolvableString::String(s) => s,
        }
    }

    /// Resolves the index from the lookup table if necessary.
    pub fn resolve(&mut self, lookup_table: &LookupTable) {
        match self {
            ResolvableString::LookupIndex(i) =>
                *self = ResolvableString::String(lookup_table[*i].clone()),
            ResolvableString::String(_) => {}
        }
    }

    /// Converts the string into an index if necessary.
    ///
    /// Does nothing if the string is not already in the lookup table
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
