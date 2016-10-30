#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate bincode;
extern crate calx_alg;
extern crate rand;

use std::collections::HashMap;
use rand::{Rng, SeedableRng, XorShiftRng};

use calx_alg::WeightedChoice;

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
