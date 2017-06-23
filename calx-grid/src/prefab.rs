use euclid::{Vector2D, vec2, Size2D};
use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map;
use std::fmt;
use std::hash::Hash;
use std::i32;
use std::iter::{FromIterator, IntoIterator};

/// A structure for storing a piece of a grid map.
///
/// All contents are guaranteed to be stored in the rectangle between (0, 0) (inclusive) and `dim`
/// (exclusive).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Prefab<T> {
    terrain: HashMap<Vector2D<i32>, usize>,
    elements: Vec<T>,
    dim: Size2D<u32>,
}

impl<T: Clone + Eq + Hash> Prefab<T> {
    fn new() -> Prefab<T> {
        Prefab {
            terrain: HashMap::new(),
            elements: Vec::new(),
            dim: Size2D::new(0, 0),
        }
    }

    pub fn get(&self, pos: Vector2D<i32>) -> Option<&T> {
        self.terrain.get(&pos).map(|&idx| &self.elements[idx])
    }

    pub fn map<F: FnMut(T) -> U, U>(self, f: F) -> Prefab<U> {
        Prefab {
            terrain: self.terrain,
            elements: self.elements.into_iter().map(f).collect(),
            dim: self.dim,
        }
    }

    pub fn dim(&self) -> Size2D<u32> { self.dim }

    pub fn iter(&self) -> PrefabIterator<T> {
        PrefabIterator {
            prefab: self,
            iter: self.terrain.iter(),
        }
    }
}

pub struct PrefabIterator<'a, T: 'a> {
    prefab: &'a Prefab<T>,
    iter: hash_map::Iter<'a, Vector2D<i32>, usize>,
}

impl<'a, T: 'a> Iterator for PrefabIterator<'a, T> {
    type Item = (Vector2D<i32>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((&pos, &idx)) => Some((pos, &self.prefab.elements[idx])),
            _ => None,
        }
    }
}


impl<T: Clone + Eq + Hash> FromIterator<(Vector2D<i32>, T)> for Prefab<T> {
    fn from_iter<I: IntoIterator<Item = (Vector2D<i32>, T)>>(iter: I) -> Self {
        // List of unique values.
        let mut element_idx: HashMap<T, usize> = HashMap::new();
        let mut ret = Prefab::new();

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;

        // Temporary storage before positions have been normalized.
        let mut temp_buffer = Vec::new();

        for (p, e) in iter {
            min_x = min(min_x, p.x);
            min_y = min(min_y, p.y);

            let val = *element_idx.entry(e.clone()).or_insert_with(|| {
                ret.elements.push(e);
                ret.elements.len() - 1
            });

            temp_buffer.push((p, val));
        }

        // Normalization: Snap bounding box to origin.
        let mut max_x = 0;
        let mut max_y = 0;

        for (mut p, e) in temp_buffer {
            p.x -= min_x;
            assert!(p.x >= 0);
            p.y -= min_y;
            assert!(p.y >= 0);

            max_x = max(p.x as u32, max_x);
            max_y = max(p.y as u32, max_y);

            ret.terrain.insert(p, e);
        }

        ret.dim = Size2D::new(max_x + 1, max_y + 1);

        ret
    }
}

impl Prefab<char> {
    fn from_text<F>(map: &str, project: F) -> Prefab<char>
    where
        F: Fn(usize, usize) -> (i32, i32),
    {
        let mut buf = Vec::new();
        for (y, line) in map.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let (map_x, map_y) = project(x, y);
                if !c.is_whitespace() {
                    buf.push((vec2(map_x, map_y), c));
                }
            }
        }

        Prefab::from_iter(buf.into_iter())
    }

    pub fn from_text_map(map: &str) -> Prefab<char> {
        Prefab::from_text(map, |x, y| (x as i32, y as i32))
    }

    pub fn from_text_hexmap(map: &str) -> Prefab<char> {
        Prefab::from_text(map, |x, y| ((x + y) as i32 / 2, y as i32))
    }
}

impl<T: fmt::Display + Clone + Eq + Hash> fmt::Display for Prefab<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.dim.height {
            for x in 0..self.dim.width {
                if let Some(c) = self.get(vec2(x as i32, y as i32)) {
                    write!(f, "{}", c)?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl<T: fmt::Display + Clone + Eq + Hash> Prefab<T> {
    /// Return a wrapper for printing the map in hex layout.
    ///
    /// Without the wrapper the print format will be a traditional dense text map.
    pub fn hexmap_display(&self) -> HexmapDisplay<T> { HexmapDisplay(self) }
}

/// Wrapper type for displaying the `Prefab` as a text hexmap.
pub struct HexmapDisplay<'a, T: 'a>(&'a Prefab<T>);

impl<'a, T: fmt::Display + Clone + Eq + Hash> fmt::Display for HexmapDisplay<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let max_width = (self.0.dim.width * 2 + self.0.dim.height) as i32;

        // Find the smallest displayed x-coordinate that actually shows up in the map.
        let min_x = (0..self.0.dim.width)
            .flat_map(move |x| {
                (0..self.0.dim.height).map(move |y| vec2(x as i32, y as i32))
            })
            .filter(|&p| self.0.get(p).is_some())
            .map(|p| p.x * 2 - p.y)
            .min()
            .unwrap_or(0);

        for y in 0..(self.0.dim.height as i32) {
            for x in min_x..(min_x + max_width) {
                if (x - y) % 2 != 0 {
                    write!(f, " ")?;
                    continue;
                }
                let map_x = (x + y) / 2;
                let map_y = y;
                if let Some(c) = self.0.get(vec2(map_x, map_y)) {
                    write!(f, "{}", c)?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

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

#[cfg(test)]
mod test {
    use super::Prefab;
    use euclid::vec2;

    #[test]
    fn test_from_text() {
        let a = Prefab::from_text_map(
            "
###
#..
##.
#..
",
        );
        let b = Prefab::from_text_hexmap(
            "
    # # #
   # . .
  # # .
 # . .
",
        );

        assert_eq!(a, b);

        assert_eq!(Some(&'#'), a.get(vec2(0, 0)));
        assert_eq!(Some(&'.'), a.get(vec2(1, 1)));
        assert_eq!(Some(&'#'), a.get(vec2(1, 2)));
    }

    /// Remove whitespace differences from text map strings.
    fn normalize(dirty: &str) -> String {
        // XXX: This should also normalize indetation by removing the longest whitespace prefix
        // shared by every line from each line.

        let mut ret = String::new();
        // Remove trailing whitespace and empty lines.
        let dirty = dirty.trim_right();
        // Remove heading empty lines in iterator construction.
        for line in dirty.lines().skip_while(|line| line.trim() == "") {
            ret.push_str(line.trim_right());
            ret.push('\n');
        }
        ret
    }

    #[test]
    fn test_print() {
        let hex_text = "
      # #
   # . .
  # # . .
 # . .
@ @ @ @";

        let dense_text = "
 ##
#..
##..
#..
@@@@";
        let map = Prefab::from_text_hexmap(&hex_text);

        assert_eq!(
            normalize(&format!("{}", map.hexmap_display())),
            normalize(hex_text)
        );
        assert_eq!(normalize(&format!("{}", map)), normalize(dense_text));
    }

    #[test]
    fn test_left_align() {
        let big_hex = "
                                                           * *
                                                        * * * *
                                                     * * * * * *
                                                  * * * * * * * *
                                               * * * * * * * * * *
                                            * * * * * * * * * * * *
                                         * * * * * * * * * * * * * *
                                      * * * * * * * * * * * * * * * *
                                   * * * * * * * * * * * * * * * * * *
                                * * * * * * * * * * * * * * * * * * * *
                             * * * * * * * * * * * * * * * * * * * * * *
                          * * * * * * * * * * * * * * * * * * * * * * * *
                       * * * * * * * * * * * * * * ^ * * * * * * * * * * *
                    * * * * * * * * * * * * * * * * * * * * * * * * * * * *
                 * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
              * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
           * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
        * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
     * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
  * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
 * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
 * * * * * * * * * * * * * * * * * * * * , , % # # , , , * * * * * * * * * * * * * *
  * * * * * * * * * * * * * * * * * * * , , , , , , , , * * * * * * * * * * * * * *
   * * * * * * * * * * * * * * * * * * , , , * * , , , * * * * * * * * * * * * * *
    * * * * * * * * * * * * * * * * * , , , , , , , , * * * * * * * * * * * * *
     * * * * * * * * * * * * * * * * , , , , , , , , * * * * * * * * * * * *
      * * * * * * * * * * * * * * * , , , , , , , , * * * * * * * * * * *
       * * * * * * * * * * * * * * , , , , , , , , * * * * * * * * * *
        * * * * * * * * * * * * * , , , , , , , , * * * * * * * * *
         * * * * * * * * * * * * * * * * * * * * * * * * * * * *
          * * * * * * * * * * * * * * * * * * * * * * * * * *
           * * * * * * * * * * * * * * * * * * * * * * * *
            * * * * * * * * * * * * * * * * * * * * * *
             * * * * * * * * * * * * * * * * * * * *
              * * * * * * * * * * * * * * * * * *
               * * * * * * * * * * * * * * * *
                * * * * * * * * * * * * * *
                 * * * * * * * * * * * *
                  * * * * * * * * * *
                   * * * * * * * *
                    * * * * * *
                     * * * *
                      * *";
        let map = Prefab::from_text_hexmap(&big_hex);

        let map2 = Prefab::from_text_hexmap(&format!("{}", map.hexmap_display()));
        assert_eq!(map, map2);

        assert_eq!(
            normalize(&format!("{}", map.hexmap_display())),
            normalize(big_hex)
        );

    }
}
