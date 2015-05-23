use std::hash::Hash;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeMap;
use num::{Zero, One};
use num::traits::{Num};

/// A node in a graph with a regular lattice.
pub trait LatticeNode: PartialEq+Eq+Clone+Hash+PartialOrd+Ord {
    /// List the neighbor nodes of this graph node.
    fn neighbors(&self) -> Vec<Self>;
}

/// A pathfinding map structure.
///
/// A Dijkstra map lets you run pathfinding from any graph node it covers
/// towards or away from the target nodes of the map. Currently the structure
/// only supports underlying graphs with a fixed grid graph where the
/// neighbors of each node must be the adjacent grid cells of that node.
pub struct Dijkstra<N> {
    weights: HashMap<N, u32>,
}

impl<N: LatticeNode> Dijkstra<N> {
    /// Create a new Dijkstra map up to limit distance from goals, omitting
    /// nodes for which the is_valid predicate returns false.
    pub fn new<F: Fn(&N) -> bool>(goals: Vec<N>, is_valid: F, limit: u32) -> Dijkstra<N> {
        assert!(goals.len() > 0);

        let mut weights = HashMap::new();
        let mut edge = HashSet::new();

        for n in goals.into_iter() {
            edge.insert(n);
        }

        for dist in 0..(limit) {
            for n in edge.iter() {
                weights.insert(n.clone(), dist);
            }

            let mut new_edge = HashSet::new();
            for n in edge.iter() {
                for m in n.neighbors().into_iter() {
                    if is_valid(&m) && !weights.contains_key(&m) {
                        new_edge.insert(m);
                    }
                }
            }

            edge = new_edge;

            if edge.is_empty() { break; }
        }

        Dijkstra {
            weights: weights,
        }
    }

    /// Return the neighbors of a cell (if any), sorted from downhill to
    /// uphill.
    pub fn sorted_neighbors(& self, node: &N) -> Vec<N> {
        let mut ret = Vec::new();
        for n in node.neighbors().iter() {
            if let Some(w) = self.weights.get(n) {
                ret.push((w, n.clone()));
            }
        }
        ret.sort_by(|&(w1, _), &(w2, _)| w1.cmp(w2));
        ret.into_iter().map(|(_, n)| n).collect()
    }
}

pub fn astar_path_with<N: LatticeNode, F, T>(metric: F, from: N, to: N, mut limit: u32) -> Option<Vec<N>>
    where F: Fn(&N, &N) -> T,
          T: Num+Ord+Copy
{
    fn build_path<'a, N: LatticeNode>(mut end: &'a N, path: &'a HashMap<N, N>) -> Vec<N> {
        let mut ret = Vec::new();
        loop {
            ret.push(end.clone());
            match path.get(end) {
                Some(n) => { end = n; }
                None => {
                    ret.reverse();
                    return ret;
                }
            }
        }
    }

    let mut visited = HashSet::new();
    let mut path = HashMap::new();

    let mut open: BTreeMap<N, T> = BTreeMap::new();
    open.insert(from, Zero::zero());

    while !open.is_empty() && limit > 0 {
        let (pick, dist) = open.iter().fold(None, |a, (x, &pathlen_x)| {
            let x_cost = pathlen_x + metric(x, &to);

            match a {
                None => Some((x.clone(), pathlen_x)),
                Some((y, pathlen_y)) => {
                    let y_cost = pathlen_y + metric(&y, &to);
                    if x_cost < y_cost { Some((x.clone(), pathlen_x)) } else { Some((y, pathlen_y)) }
                }
            }}).unwrap();

        if pick == to {
            return Some(build_path(&pick, &path));
        }

        open.remove(&pick);

        let new_pathlen = dist + One::one();
        for x in pick.neighbors().into_iter() {
            if visited.contains(&x) { continue; }

            if let Some(&old_pathlen) = open.get(&x) {
                if old_pathlen <= new_pathlen { continue; }
            }

            path.insert(x.clone(), pick.clone());
            open.insert(x, new_pathlen);
        }

        visited.insert(pick);
        limit -= 1;
    }

    return None;
}

#[cfg(test)]
mod test {
    #[test]
    fn test_astar() {
        use super::{LatticeNode, astar_path_with};
        use geom::{V2};

        impl LatticeNode for V2<i32> {
            fn neighbors(&self) -> Vec<V2<i32>> {
                vec![
                    V2(self.0 - 1, self.1),
                    V2(self.0, self.1 - 1),
                    V2(self.0 + 1, self.1),
                    V2(self.0, self.1 + 1),
                ]
            }
        }

        let path = astar_path_with(|a, b| { let v = *b - *a; v.dot(v) }, V2(1, 1), V2(10, 10), 10000).unwrap();
        assert!(path[0] == V2(1, 1));
        assert!(path[path.len() - 1] == V2(10, 10));
    }
}
