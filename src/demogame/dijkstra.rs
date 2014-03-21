use std::hash::Hash;
use collections::hashmap::HashMap;
use collections::hashmap::HashSet;

/// Build a Dijkstra map starting from the goal nodes and using the neighbors
/// function to define the graph to up to limit distance.
pub fn build_map<N: Hash + Eq + Clone>(
    goals: ~[N], neighbors: |&N| -> ~[N], limit: uint) -> HashMap<N, uint> {
    assert!(goals.len() > 0);
    let mut ret = HashMap::new();

    // Init goal nodes to zero score.
    for k in goals.iter() {
        ret.insert(k.clone(), 0);
    }

    let mut edge = ~HashSet::new();

    for k in goals.iter() {
        for n in neighbors(k).iter() {
            // XXX: Extra clone op here, should just shuffle references until
            // things get cloned for the ret structure.
            if !ret.contains_key(n) { edge.insert(n.clone()); }
        }
    }

    for dist in range(1, limit) {
        for k in edge.iter() {
            ret.insert(k.clone(), dist);
        }

        let mut new_edge = ~HashSet::new();
        for k in edge.iter() {
            for n in neighbors(k).iter() {
                if !ret.contains_key(n) { new_edge.insert(n.clone()); }
            }
        }

        edge = new_edge;
    }

    ret
}
