extern crate serde_json;
extern crate calx_alg;
extern crate rand;

use std::collections::HashMap;
use rand::{Rng, SeedableRng, XorShiftRng};

use calx_alg::WeightedChoice;

#[test]
fn test_serialize_rng() {
    use calx_alg::EncodeRng;

    let mut rng: EncodeRng<XorShiftRng> = SeedableRng::from_seed([1, 2, 3, 4]);

    let saved = serde_json::to_string(&rng).expect("Serialization failed");
    let mut rng2: EncodeRng<XorShiftRng> = serde_json::from_str(&saved)
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

fn splits_into(space: usize, line: &str, parts: &[&str]) {
    use calx_alg::split_line;

    split_line(line, |_| 1.0, space as f32)
        .zip(parts)
        .all(|(actual, &expected)| {
            assert_eq!(expected, actual);
            true
        });

    assert_eq!(parts.len(), split_line(line, |_| 1.0, space as f32).count());
}

#[test]
fn test_split_line() {
    splits_into(10, "", &[""]);
    splits_into(0, "", &[""]);
    splits_into(0, "abc", &["a", "b", "c"]);
    splits_into(10, "abc", &["abc"]);
    splits_into(6, "the cat", &["the", "cat"]);
    splits_into(7, "the cat", &["the cat"]);
    splits_into(10, "the   cat", &["the   cat"]);
    splits_into(5, "the     cat", &["the", "cat"]);
    splits_into(5, "the \t cat", &["the", "cat"]);
    splits_into(4, "deadbeef", &["dead", "beef"]);
    splits_into(5, "the \t cat", &["the", "cat"]);
}

#[test]
fn test_weighted_choice() {
    let mut histogram: HashMap<u32, f32> = HashMap::new();
    let mut rng: XorShiftRng = SeedableRng::from_seed([1, 2, 3, 4]);
    let items = vec![1u32, 2, 3, 4];
    let n = 1000;

    for _ in 0..n {
        let choice = *items.iter().weighted_choice(&mut rng, |&&x| x as f32).unwrap();
        *histogram.entry(choice).or_insert(0.0) += 1.0;
    }

    let measurement = vec![histogram.get(&1).unwrap() / n as f32,
                           histogram.get(&2).unwrap() / n as f32,
                           histogram.get(&3).unwrap() / n as f32,
                           histogram.get(&4).unwrap() / n as f32];

    // The weights match the values because 1+2+3+4 = 10.
    let ideal = vec![0.1, 0.2, 0.3, 0.4];

    let err = measurement.iter().zip(ideal).map(|(x, y)| (x - y) * (x - y)).sum::<f32>() /
              measurement.len() as f32;
    println!("Mean square error from expected: {}", err);
    assert!(err < 0.0001);
}

#[test]
fn test_random_permutation() {
    use calx_alg::RandomPermutation;
    let perm: Vec<usize> = RandomPermutation::new(&mut rand::thread_rng(), 100).collect();
    let mut sorted = perm.clone();
    sorted.sort();
    assert_eq!(sorted, (0..100).collect::<Vec<usize>>());

    // XXX: It is technically possible to get the unpermutating permutation and fail here.
    // Not very likely though.
    assert_ne!(perm, sorted);
    sorted.reverse();
    assert_ne!(perm, sorted);
}

#[test]
fn test_bit_spread() {
    use calx_alg::{compact_bits_by_2, spread_bits_by_2};
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let x = rng.gen::<u16>() as u32;
        assert!(x == 0 || spread_bits_by_2(x) != x);
        assert_eq!(compact_bits_by_2(spread_bits_by_2(x)), x);
    }
}
