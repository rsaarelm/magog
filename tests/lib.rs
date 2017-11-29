extern crate serde_json;
extern crate calx;
extern crate rand;

use calx::WeightedChoice;
use rand::{Rng, SeedableRng, XorShiftRng};
use std::collections::HashMap;

#[test]
fn test_serialize_rng() {
    use calx::EncodeRng;

    let mut rng: EncodeRng<XorShiftRng> = SeedableRng::from_seed([1, 2, 3, 4]);

    let saved = serde_json::to_string(&rng).expect("Serialization failed");
    let mut rng2: EncodeRng<XorShiftRng> =
        serde_json::from_str(&saved).expect("Deserialization failed");

    assert_eq!(rng.next_u32(), rng2.next_u32());
}

#[test]
fn test_noise() {
    use calx::noise;

    for i in 0i32..100 {
        assert!(noise(i) >= -1.0 && noise(i) <= 1.0);
    }
}

fn splits_into(space: usize, line: &str, parts: &[&str]) {
    use calx::split_line;

    split_line(line, |_| 1.0, space as f32).zip(parts).all(
        |(actual, &expected)| {
            assert_eq!(expected, actual);
            true
        },
    );

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
        let choice = *items
            .iter()
            .weighted_choice(&mut rng, |&&x| x as f32)
            .unwrap();
        *histogram.entry(choice).or_insert(0.0) += 1.0;
    }

    let measurement = vec![
        histogram[&1] / n as f32,
        histogram[&2] / n as f32,
        histogram[&3] / n as f32,
        histogram[&4] / n as f32,
    ];

    // The weights match the values because 1+2+3+4 = 10.
    let ideal = vec![0.1, 0.2, 0.3, 0.4];

    let err = measurement
        .iter()
        .zip(ideal)
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>() / measurement.len() as f32;
    println!("Mean square error from expected: {}", err);
    assert!(err < 0.0001);
}

#[test]
fn test_random_permutation() {
    use calx::RandomPermutation;
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
    use calx::{compact_bits_by_2, spread_bits_by_2};
    use std::u16;

    for x in 0..(u16::MAX as u32 + 1) {
        if x > 1 {
            assert_ne!(x, spread_bits_by_2(x));
        }
        assert_eq!(compact_bits_by_2(spread_bits_by_2(x)), x);
    }
}

#[test]
fn test_retry_gen() {
    use calx::retry_gen;

    fn failing_gen<R: Rng>(rng: &mut R) -> Result<f32, ()> {
        let x = rng.next_f32();
        if x < 0.1 { Ok(x) } else { Err(()) }
    }

    assert!(retry_gen(1000, &mut rand::thread_rng(), failing_gen).is_ok());
}
