use std::io::Write;
use world::{attack_damage, roll};

fn ev<F>(n: usize, f: F) -> f32
where
    F: Fn(&mut rand::prelude::ThreadRng) -> f32,
{
    let mut acc = 0.0;
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        acc += f(&mut rng);
    }

    acc / n as f32
}

fn expected_dmg(advantage: i32) -> f32 {
    const REPEAT_ROLLS: usize = 1_000_000;

    ev(REPEAT_ROLLS, |rng| {
        let roll = roll(rng);
        let dmg = attack_damage(roll, advantage, 100);
        dmg as f32 / 100.0
    })
}

fn main() {
    print!("     ");
    for one in 0..10 {
        print!("    0{}", one);
    }
    println!("");

    for tens in -3..10 {
        print!("{:>3}0  ", tens);
        for ones in 0..10 {
            let n = tens * 10 + ones;
            print!("{:.3} ", expected_dmg(n));
            let _ = ::std::io::stdout().flush();
        }
        println!("");
    }
    let e = ev(1_000_000, |rng| {
        let roll = roll(rng);
        let dmg = attack_damage(roll, 0, 100);
        dmg as f32 / 100.0
    });
    println!("Hello, world!");
    println!("Expected dmg: {}", e);
}
