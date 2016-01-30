#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate bincode;
extern crate calx_alg;
extern crate rand;

use rand::{Rng, XorShiftRng, SeedableRng};

#[test]
fn test_serialize_rng() {
    use calx_alg::EncodeRng;
    use bincode::{serde, SizeLimit};

    let mut rng: EncodeRng<XorShiftRng> = SeedableRng::from_seed([1, 2, 3, 4]);

    let saved = serde::serialize(&rng, SizeLimit::Infinite)
                    .expect("Serialization failed");
    let mut rng2 = serde::deserialize::<EncodeRng<XorShiftRng>>(&saved)
                       .expect("Deserialization failed");

    assert!(rng.next_u32() == rng2.next_u32());
}

#[test]
fn test_noise() {
    use calx_alg::noise;

    for i in 0i32..100 {
        assert!(noise(i) >= -1.0 && noise(i) <= 1.0);
    }
}

#[test]
fn test_log_odds() {
    use calx_alg::{to_log_odds, from_log_odds};
    assert_eq!(from_log_odds(0.0), 0.5);
    assert_eq!(to_log_odds(0.5), 0.0);

    assert_eq!((from_log_odds(-5.0) * 100.0) as i32, 24);
    assert_eq!(to_log_odds(0.909091) as i32, 10);
}


#[test]
fn test_astar() {
    use calx_alg::{LatticeNode, astar_path_with};

    #[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
    struct V([i32; 2]);

    impl LatticeNode for V {
        fn neighbors(&self) -> Vec<V> {
            vec![
                V([self.0[0] - 1, self.0[1]]),
                V([self.0[0], self.0[1] - 1]),
                V([self.0[0] + 1, self.0[1]]),
                V([self.0[0], self.0[1] + 1]),
            ]
        }
    }

    let path = astar_path_with(|a, b| {
                                   (a.0[0] - b.0[0]).abs() +
                                   (a.0[1] - b.0[1]).abs()
                               },
                               V([1, 1]),
                               V([10, 10]),
                               10000)
                   .unwrap();
    assert!(path[0] == V([1, 1]));
    assert!(path[path.len() - 1] == V([10, 10]));
}

#[test]
fn test_split_line() {
    use calx_alg::split_line;

    assert_eq!(("", ""), split_line("", &|_| 1.0, 12.0));
    assert_eq!(("a", ""), split_line("a", &|_| 1.0, 5.0));
    assert_eq!(("the", "cat"), split_line("the cat", &|_| 1.0, 5.0));
    assert_eq!(("the", "cat"), split_line("the     cat", &|_| 1.0, 5.0));
    assert_eq!(("the", "cat"), split_line("the  \t cat", &|_| 1.0, 5.0));
    assert_eq!(("the", "cat"), split_line("the \ncat", &|_| 1.0, 32.0));
    assert_eq!(("the", "   cat"),
               split_line("the \n   cat", &|_| 1.0, 32.0));
    assert_eq!(("the  cat", ""), split_line("the  cat", &|_| 1.0, 32.0));
    assert_eq!(("the", "cat sat"), split_line("the cat sat", &|_| 1.0, 6.0));
    assert_eq!(("the cat", "sat"), split_line("the cat sat", &|_| 1.0, 7.0));
    assert_eq!(("a", "bc"), split_line("abc", &|_| 1.0, 0.01));
    assert_eq!(("dead", "beef"), split_line("deadbeef", &|_| 1.0, 4.0));
    assert_eq!(("the-", "cat"), split_line("the-cat", &|_| 1.0, 5.0));
}
