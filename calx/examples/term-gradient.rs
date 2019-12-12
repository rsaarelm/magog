use calx;
use term;

use calx::{term_color, PseudoTermColor};

fn print(t: &mut Box<term::StdoutTerminal>, c: PseudoTermColor) {
    t.reset().unwrap();
    let fg;
    match c {
        PseudoTermColor::Mixed { fore, back, mix: _ } => {
            fg = u32::from(fore);
            t.bg(u32::from(back)).unwrap();
        }
        PseudoTermColor::Solid(fore) => {
            fg = u32::from(fore);
        }
    }

    t.fg(fg).unwrap();
    if fg > 8 {
        t.attr(term::Attr::Bold).unwrap();
    }
    print!("{}", c.ch());
}

fn main() {
    let mut t = term::stdout().unwrap();

    for x in 0..80 {
        let a = (x as f32) / 80.0;
        let c = calx::lerp(term_color::MAROON, term_color::YELLOW, a);
        print(&mut t, c);
    }
    let _ = t.reset();
    println!();

    let mut path = calx::LerpPath::new((0.0f32, term_color::BLACK), (1.0f32, term_color::YELLOW));
    path.add((0.20, term_color::NAVY));
    path.add((0.40, term_color::BLUE));
    path.add((0.60, term_color::TEAL));
    path.add((0.80, term_color::AQUA));
    path.add((0.90, term_color::WHITE));
    for x in 0..80 {
        let a = (x as f32) / 80.0;
        print(&mut t, path.sample(a));
    }
    let _ = t.reset();
    println!();
}
