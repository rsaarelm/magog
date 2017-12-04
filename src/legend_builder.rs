use std::collections::BTreeMap;

/// Build a text map legend given an alphabet and a sequence of elements.
///
/// Each unique element will be assigned a letter from the alphabet. A preference function can
/// provide an additional alphabet prefix for specific values, which will be sampled before the
/// main alphabet. This will allow having standard special characters for eg. certain types of
/// terrain tiles. When no special alphabet is required, the preference function can return an
/// empty string.
pub struct LegendBuilder<T, F> {
    /// The generated legend map.
    pub legend: BTreeMap<char, T>,
    /// Set to true if an add operation failed to assign a symbol.
    pub out_of_alphabet: bool,
    seen_values: BTreeMap<T, char>,
    prefix_fn: F,
    alphabet: String,
}

impl<T, F> LegendBuilder<T, F>
where
    T: Ord + Eq + Clone,
    F: FnMut(&T) -> &'static str,
{
    /// Initialize the legend builder.
    pub fn new(alphabet: String, prefix_fn: F) -> LegendBuilder<T, F> {
        LegendBuilder {
            legend: BTreeMap::new(),
            out_of_alphabet: false,
            seen_values: BTreeMap::new(),
            prefix_fn: prefix_fn,
            alphabet: alphabet,
        }
    }

    /// Show a value to `LegendBuilder` and get its legend key.
    ///
    /// Returns an error if the alphabet has been exhausted.
    pub fn add(&mut self, value: &T) -> Result<char, ()> {
        if let Some(&c) = self.seen_values.get(value) {
            return Ok(c);
        }

        for c in (self.prefix_fn)(value).chars().chain(self.alphabet.chars()) {
            if self.legend.contains_key(&c) {
                continue;
            }

            self.legend.insert(c, value.clone());
            self.seen_values.insert(value.clone(), c);
            return Ok(c);
        }
        self.out_of_alphabet = true;
        Err(())
    }
}
