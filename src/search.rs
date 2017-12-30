use num::{One, Zero};
use num::traits::Num;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::hash::Hash;

/// A node in a graph with a regular grid.
pub trait GridNode: PartialEq + Eq + Clone + Hash + PartialOrd + Ord {
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
    pub weights: HashMap<N, u32>,
}

impl<N: GridNode> Dijkstra<N> {
    /// Create a new Dijkstra map up to limit distance from goals, omitting
    /// nodes for which the is_valid predicate returns false.
    pub fn new<F: Fn(&N) -> bool>(goals: Vec<N>, is_valid: F, limit: u32) -> Dijkstra<N> {
        assert!(!goals.is_empty());

        let mut weights = HashMap::new();
        let mut edge = HashSet::new();

        for n in goals {
            edge.insert(n);
        }

        for dist in 0..(limit) {
            for n in &edge {
                weights.insert(n.clone(), dist);
            }

            let mut new_edge = HashSet::new();
            for n in &edge {
                for m in n.neighbors() {
                    if is_valid(&m) && !weights.contains_key(&m) {
                        new_edge.insert(m);
                    }
                }
            }

            edge = new_edge;

            if edge.is_empty() {
                break;
            }
        }

        Dijkstra { weights: weights }
    }

    /// Return the neighbors of a cell (if any), sorted from downhill to
    /// uphill.
    pub fn sorted_neighbors(&self, node: &N) -> Vec<N> {
        let mut ret = Vec::new();
        for n in &node.neighbors() {
            if let Some(w) = self.weights.get(n) {
                ret.push((w, n.clone()));
            }
        }
        ret.sort_by(|&(w1, _), &(w2, _)| w1.cmp(w2));
        ret.into_iter().map(|(_, n)| n).collect()
    }
}

/// Find a path between two grid points using the A* algorithm.
pub fn astar_grid<N, F, T>(metric: F, from: N, to: N, mut limit: u32) -> Option<Vec<N>>
where
    N: GridNode,
    F: Fn(&N, &N) -> T,
    T: Num + Ord + Copy,
{
    fn build_path<'a, N>(mut end: &'a N, path: &'a HashMap<N, N>) -> Vec<N>
    where
        N: GridNode,
    {
        let mut ret = Vec::new();
        loop {
            ret.push(end.clone());
            match path.get(end) {
                Some(n) => {
                    end = n;
                }
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
        let (pick, dist) = open.iter()
            .fold(None, |a, (x, &pathlen_x)| {
                let x_cost = pathlen_x + metric(x, &to);

                match a {
                    None => Some((x.clone(), pathlen_x)),
                    Some((y, pathlen_y)) => {
                        let y_cost = pathlen_y + metric(&y, &to);
                        if x_cost < y_cost {
                            Some((x.clone(), pathlen_x))
                        } else {
                            Some((y, pathlen_y))
                        }
                    }
                }
            })
            .unwrap();

        if pick == to {
            return Some(build_path(&pick, &path));
        }

        open.remove(&pick);

        let new_pathlen = dist + One::one();
        for x in pick.neighbors() {
            if visited.contains(&x) {
                continue;
            }

            if let Some(&old_pathlen) = open.get(&x) {
                if old_pathlen <= new_pathlen {
                    continue;
                }
            }

            path.insert(x.clone(), pick.clone());
            open.insert(x, new_pathlen);
        }

        visited.insert(pick);
        limit -= 1;
    }

    None
}

/// Find A* path in freeform graph.
///
/// The `neighbors` function returns neighboring nodes and their estimated distance from the goal.
/// The search will treat any node whose distance is zero as a goal and return a path leading to
/// it.
pub fn astar_path<'a, N, F>(start: N, end: &N, neighbors: F) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    F: Fn(&N) -> Vec<(N, f32)>,
{
    #[derive(Eq, PartialEq)]
    struct MetricNode<N> {
        value: u32,
        item: N,
        come_from: Option<N>,
    }
    impl<N: Eq> Ord for MetricNode<N> {
        fn cmp(&self, other: &Self) -> Ordering { self.value.cmp(&other.value) }
    }
    impl<N: Eq> PartialOrd for MetricNode<N> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
    }

    fn node<N: Eq>(item: N, dist: f32, come_from: Option<N>) -> MetricNode<N> {
        debug_assert!(dist >= 0.0);
        // Convert dist to integers so we can push MetricNodes into BinaryHeap that expects Ord.
        // The trick here is that non-negative IEEE 754 floats have the same ordering as their
        // binary representations interpreted as integers.
        //
        // Also flip the sign on the value, shorter distance means bigger value, since BinaryHeap
        // returns the largest item first.
        let value = ::std::u32::MAX - dist.to_bits();
        MetricNode {
            item,
            value,
            come_from,
        }
    }

    let mut come_from = HashMap::new();

    let mut open = BinaryHeap::new();
    open.push(node(start.clone(), ::std::f32::MAX, None));

    // Find shortest path.
    let mut goal = loop {
        if let Some(closest) = open.pop() {
            debug_assert!(!come_from.contains_key(&closest.item));

            if let Some(from) = closest.come_from {
                come_from.insert(closest.item.clone(), from);
            }

            if &closest.item == end {
                break Some(closest.item);
            }

            for (item, dist) in neighbors(&closest.item) {
                let already_seen = come_from.contains_key(&item) || item == start;
                if already_seen {
                    continue;
                }
                open.push(node(item, dist, Some(closest.item.clone())));
            }
        } else {
            break None;
        }
    };

    // Extract path from the graph structure.
    let mut path = Vec::new();
    while let Some(x) = goal {
        goal = come_from.remove(&x);
        path.push(x);
    }
    path.reverse();

    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_old_astar() {
        #[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
        struct V([i32; 2]);

        impl GridNode for V {
            fn neighbors(&self) -> Vec<V> {
                vec![
                    V([self.0[0] - 1, self.0[1]]),
                    V([self.0[0], self.0[1] - 1]),
                    V([self.0[0] + 1, self.0[1]]),
                    V([self.0[0], self.0[1] + 1]),
                ]
            }
        }

        let path = astar_grid(
            |a, b| (a.0[0] - b.0[0]).abs() + (a.0[1] - b.0[1]).abs(),
            V([1, 1]),
            V([10, 10]),
            10000,
        ).unwrap();
        assert!(path[0] == V([1, 1]));
        assert!(path[path.len() - 1] == V([10, 10]));
    }

    #[test]
    fn test_astar() {
        fn neighbors(origin: i32, &x: &i32) -> Vec<(i32, f32)> {
            let mut ret = Vec::with_capacity(2);
            for i in &[-1, 1] {
                let x = x + i;
                ret.push((x, (x - origin).abs() as f32));
            }
            ret
        }

        assert_eq!(Some(vec![8]), astar_path(8, &8, |_| Vec::new()));
        assert_eq!(None, astar_path(8, &12, |_| Vec::new()));

        assert_eq!(Some(vec![8]), astar_path(8, &8, |x| neighbors(8, x)));
        assert_eq!(
            Some(vec![8, 9, 10, 11, 12]),
            astar_path(8, &12, |x| neighbors(8, x))
        );
    }
}
