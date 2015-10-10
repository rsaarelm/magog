// Tests for calx_ecs are off-crate because the Ecs macro expects to
// find Ecs stuff with an absolute crate path.

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Desc {
    name: String,
    icon: usize,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Pos {
    x: i32,
    y: i32,
}

Ecs! {
    desc: Desc,
    pos: Pos,
}

#[test]
fn test_ecs() {
    use bincode::{serde, SizeLimit};

    let mut ecs = Ecs::new();

    let e1 = ecs.make();
    assert!(ecs.contains(e1));

    assert!(!ecs.pos.contains(e1));
    ecs.pos.insert(e1, Pos { x: 3, y: 4 });
    assert_eq!(ecs.pos[e1], Pos { x: 3, y: 4 });

    Desc {
        name: "Orc".to_string(),
        icon: 8,
    }.add_to_ecs(&mut ecs, e1);
    assert_eq!(ecs.desc[e1].name, "Orc");

    ecs.remove(e1);
    assert!(!ecs.pos.contains(e1));
    assert!(!ecs.contains(e1));

    let e2 = ecs.make();
    assert!(e2 != e1);

    // Use the loadout system to create an entity.
    let loadout = Loadout::new().c(Desc {
        name: "Critter".to_string(),
        icon: 10,
    });

    // Then instantiate an entity with that form.
    let e3 = loadout.make(&mut ecs);
    assert!(ecs.desc[e3].icon == 10);

    // Check that serialization works.
    let saved = serde::serialize(&ecs, SizeLimit::Infinite).expect("ECS serialization failed");
    let ecs2 = serde::deserialize::<Ecs>(&saved).expect("ECS deserialization failed");
    assert!(ecs2.desc[e3].icon == 10);
}
