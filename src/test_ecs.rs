// Tests for calx_ecs are off-crate because the Ecs macro expects to
// find Ecs stuff with an absolute crate path.

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct Desc {
    name: String,
    icon: usize,
}

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct Pos {
    x: i32,
    y: i32,
}

Ecs! {
    desc: ::test_ecs::Desc,
    pos: ::test_ecs::Pos,
}

#[test]
fn test_ecs() {
    let mut ecs = Ecs::new();

    let e1 = ecs.make();
    assert!(ecs.contains(e1));

    assert!(!ecs.pos.contains(e1));
    ecs.pos.insert(e1, Pos { x: 3, y: 4 });
    assert_eq!(ecs.pos[e1], Pos { x: 3, y: 4 });

    Desc { name: "Orc".to_string(), icon: 8 }.add_to(&mut ecs, e1);
    assert_eq!(ecs.desc[e1].name, "Orc");

    ecs.remove(e1);
    assert!(!ecs.pos.contains(e1));
    assert!(!ecs.contains(e1));

    let e2 = ecs.make();
    assert!(e2 != e1);
}
