// Tests for calx_ecs are off-crate because the Ecs macro expects to
// find Ecs stuff with an absolute crate path.

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
struct Desc {
    name: String,
    icon: usize,
}

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
struct Pos {
    x: i32,
    y: i32,
}

Ecs! {
    desc: ::test_ecs::Desc,
    pos: ::test_ecs::Pos,
}

#[test]
fn test_insert() {
    use calx_ecs::{Component};

    let mut ecs = Ecs::new();

    let e1 = ecs.make(None);

    assert!(ecs.contains(e1));
    assert_eq!(ecs.pos().get(e1), None);
    assert!(!ecs.has_indexed_component(Pos::id(), e1));
    ecs.mu().pos().insert(e1, Pos { x: 3, y: 4 });
    assert_eq!(ecs.pos().get(e1), Some(&Pos { x: 3, y: 4 }));
    assert!(ecs.has_indexed_component(Pos::id(), e1));

    ecs.remove(e1);
    assert!(!ecs.contains(e1));
}

#[test]
#[should_panic]
fn nonproto_inherit1() {
    let mut ecs = Ecs::new();

    let nonproto = ecs.make(None);
    ecs.make(Some(nonproto));
}

#[test]
#[should_panic]
fn nonproto_inherit2() {
    let mut ecs = Ecs::new();

    let nonproto = ecs.make(None);
    ecs.make_prototype(Some(nonproto));
}

#[test]
#[should_panic]
fn nonproto_reparent() {
    let mut ecs = Ecs::new();

    let nonproto = ecs.make(None);
    let blammo = ecs.make(None);

    ecs.set_parent(blammo, Some(nonproto));
}

#[test]
#[should_panic]
fn removed_parent() {
    let mut ecs = Ecs::new();

    let proto = ecs.make_prototype(None);
    ecs.remove(proto);

    ecs.make(Some(proto));
}

#[test]
fn test_prototype() {
    let mut ecs = Ecs::new();

    let p1 = ecs.make_prototype(None);
    ecs.mu().desc().insert(p1, Desc { name: "Orc".to_string(), icon: 5 });

    assert_eq!(ecs.largest_uid(), 0);
    let e1 = ecs.make(Some(p1));
    let e2 = ecs.make(None);
    assert_eq!(ecs.largest_uid(), 2);

    assert_eq!(ecs.desc().get(e1), Some(&Desc { name: "Orc".to_string(), icon: 5 }));
    assert_eq!(ecs.desc().get(e2), None);

    ecs.set_parent(e2, Some(p1));

    assert_eq!(ecs.desc().get(e1), Some(&Desc { name: "Orc".to_string(), icon: 5 }));
    assert_eq!(ecs.desc().get(e2), Some(&Desc { name: "Orc".to_string(), icon: 5 }));

    // Changing the prototype, should show up on both e1 and e2.
    ecs.mu().desc().get(p1).map(|x| x.icon = 8);

    assert_eq!(ecs.desc().get(e1), Some(&Desc { name: "Orc".to_string(), icon: 8 }));
    assert_eq!(ecs.desc().get(e2), Some(&Desc { name: "Orc".to_string(), icon: 8 }));

    // This should copy-on-write to e2 but keep e1 unchanged.
    ecs.mu().desc().get(e2).map(|x| x.name = "Blork".to_string());

    assert_eq!(ecs.desc().get(e1), Some(&Desc { name: "Orc".to_string(), icon: 8 }));
    assert_eq!(ecs.desc().get(e2), Some(&Desc { name: "Blork".to_string(), icon: 8 }));

    assert_eq!(ecs.desc().get_local(e1), None);
    assert_eq!(ecs.desc().get_local(e2), Some(&Desc { name: "Blork".to_string(), icon: 8 }));

    assert_eq!(ecs.get_parent(e1), Some(p1));
    assert_eq!(ecs.get_parent(p1), None);

    assert_eq!(ecs.first_entity(), Some(e1));
    assert_eq!(ecs.next_entity(e1), Some(e2));
    assert_eq!(ecs.next_entity(e2), None);
}

#[test]
fn test_builder() {
    let mut ecs = Ecs::new();

    let p1 = Build::prototype(&mut ecs, None)
        .c(Desc { name: "Ogre".to_string(), icon: 12 })
        .c(Pos { x: 4, y: 7 })
        .e();

    let e = Build::entity(&mut ecs, Some(p1))
        .c(Pos { x: 10, y: 11 })
        .e();


    assert!(p1.is_prototype());
    assert!(!e.is_prototype());

    assert_eq!(ecs.desc().get(e), Some(&Desc { name: "Ogre".to_string(), icon: 12 }));
    assert_eq!(ecs.pos().get(e), Some(&Pos { x: 10, y: 11 }));
}

#[test]
#[should_panic]
fn test_uid_invalidation() {
    let mut ecs = Ecs::new();
    let e1 = ecs.make(None);
    ecs.mu().pos().insert(e1, Pos { x: 3, y: 4 });

    ecs.remove(e1);
    let e2 = ecs.make(None);
    assert!(e2.idx == e1.idx, "Idx not reused after remove");

    assert!(ecs.pos().get(e2).is_none());
    ecs.mu().pos().insert(e2, Pos { x: 8, y: 10 });

    assert!(!ecs.contains(e1));
    assert!(ecs.pos().get(e1).is_none());
    assert!(ecs.mu().pos().get(e1).is_none());
}
