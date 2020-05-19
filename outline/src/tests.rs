use crate::{from_outline, into_outline, outline::Outline};
use pretty_assertions::assert_eq;
use serde::{de, Deserialize, Serialize};
use std::fmt;

fn _test<T: de::DeserializeOwned + Serialize + fmt::Debug + PartialEq>(
    outline: &str,
    value: T,
    test_pretty_print: bool,
) {
    let mut outline = Outline::from(outline);

    print!("\ntesting\n{}", outline);

    let ser_outline = into_outline(&value).expect("Value did not serialize into outline");
    print!("\nValue serializes as\n{}", ser_outline);

    // Test deserialize

    // String to outline always produces empty headline and content in children.
    // Extract the first child as the unit to deserialize the type from.
    outline.lift_singleton();

    let outline_value: T = from_outline(&outline).expect("Outline did not parse into value");

    assert_eq!(outline_value, value);

    // Serialization tests.
    let ser_outline_print = if ser_outline.headline.is_some() {
        Outline::list(vec![ser_outline.clone()])
    } else {
        ser_outline.clone()
    };

    print!("\nserialized\n{}", ser_outline_print);

    if test_pretty_print {
        assert_eq!(
            ser_outline_print.to_string(),
            outline.to_string(),
            "Failed to pretty-print"
        );
    }

    let roundtrip_value: T =
        from_outline(&ser_outline).expect("Serialized outline did not parse into value");
    assert_eq!(
        roundtrip_value, value,
        "Value changed after serialization roundtrip"
    );

    println!("test ok");
}

fn test<T: de::DeserializeOwned + Serialize + fmt::Debug + PartialEq>(outline: &str, value: T) {
    _test(outline, value, true);
}

/// Don't require serialization to print like input.
fn test_no_pp<T: de::DeserializeOwned + Serialize + fmt::Debug + PartialEq>(
    outline: &str,
    value: T,
) {
    _test(outline, value, false);
}

fn not_parsed<T: de::DeserializeOwned + fmt::Debug + PartialEq>(outline: &str) {
    let mut outline = Outline::from(outline);

    outline.lift_singleton();

    assert!((from_outline(&outline) as Result<T, _>).is_err());
}

#[test]
fn test_simple() {
    test("123", 123u32);
    test("2.71828", 2.71828f32);
    test("true", true);
    test("false", false);
    test("symbol", "symbol".to_string());
    test("two words", "two words".to_string());

    test("a", 'a');
    not_parsed::<char>("aa");
    test("殺", '殺');
    not_parsed::<char>("殺殺殺殺殺殺殺");

    not_parsed::<u32>("123 junk");
}

#[test]
fn test_tuple() {
    test("123", (123u32,));
    test("123 zomg", (123u32, "zomg".to_string()));
}

#[test]
fn test_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Simple {
        num: i32,
        title: String,
        #[serde(default)]
        tags: Vec<String>,
    }

    test(
        "\
\tnum 32
\ttitle foo bar
\ttags foo bar",
        Simple {
            num: 32,
            title: "foo bar".into(),
            tags: vec!["foo".into(), "bar".into()],
        },
    );

    // Default value for missing field
    test(
        "\
\tnum 32
\ttitle foo bar",
        Simple {
            num: 32,
            title: "foo bar".into(),
            tags: Vec::new(),
        },
    );

    // Extra field in input
    test_no_pp(
        "\
\tnum 32
\ttitle foo bar
\ttags foo bar
\tthe-wot um",
        Simple {
            num: 32,
            title: "foo bar".into(),
            tags: vec!["foo".into(), "bar".into()],
        },
    );

    not_parsed::<Simple>(
        "\
\tnum 32 garbage
\ttitle foo bar
\ttags foo bar",
    );

    test_no_pp(
        "\
\tnum 32
\ttitle
\t\tmany
\t\tlines
\ttags
\t\tfoo
\t\tbar",
        Simple {
            num: 32,
            title: "many\nlines\n".into(),
            tags: vec!["foo".into(), "bar".into()],
        },
    );

    not_parsed::<Simple>(
        "\
\tnom 32
\ttitle foo bar
\ttags foo bar",
    );
}

#[test]
fn test_heading_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Item {
        #[serde(rename = "__heading__")]
        title: String,
        a: i32,
        b: i32,
    }

    test(
        "\
\tFoo
\t\ta 1
\t\tb 2",
        Item {
            title: "Foo".into(),
            a: 1,
            b: 2,
        },
    );

    test(
        "\
\tFoo
\t\ta 1
\t\tb 2
\tBar
\t\ta 3
\t\tb 4",
        vec![
            Item {
                title: "Foo".into(),
                a: 1,
                b: 2,
            },
            Item {
                title: "Bar".into(),
                a: 3,
                b: 4,
            },
        ],
    );
}

/*

// XXX: This seems to be impossible to get to work. Serde switches from struct parsing to map
// parsing when flatten is used, and expected field names do not get communicated to map parsing.
// The magic heading field can't be sent to all map parses because it usually isn't wanted, and
// there's no way to tell when it is wanted.

#[test]
fn test_flattened_heading_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Frame {
        #[serde(rename = "__heading__")]
        title: String,
        #[serde(flatten)]
        item: Item,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Item {
        a: i32,
        b: i32,
    }

    test(
        "\
\tFoo
\t\ta 1
\t\tb 2",
        Frame {
            title: "Foo".into(),
            item: Item { a: 1, b: 2 },
        },
    );
}
*/

#[test]
fn test_inline_struct() {
    #[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
    struct Vec {
        x: i32,
        y: i32,
    }

    test("x -5 y 10", Vec { x: -5, y: 10 });
}

#[test]
fn test_nested_struct() {
    #[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
    struct Nesting {
        x: i32,
        y: i32,
        #[serde(default)]
        tail: Option<Box<Nesting>>,
    }

    test(
        "x 1 y 2",
        Nesting {
            x: 1,
            y: 2,
            tail: None,
        },
    );

    test_no_pp(
        "\
\tx 1
\ty 2
\ttail
\t\tx 3
\t\ty 4",
        Nesting {
            x: 1,
            y: 2,
            tail: Some(Box::new(Nesting {
                x: 3,
                y: 4,
                tail: None,
            })),
        },
    );

    test(
        "\
\tx 1
\ty 2
\ttail x 3 y 4",
        Nesting {
            x: 1,
            y: 2,
            tail: Some(Box::new(Nesting {
                x: 3,
                y: 4,
                tail: None,
            })),
        },
    );
}

#[test]
fn test_inline_list() {
    test("1 2 3", vec![1u32, 2u32, 3u32]);

    test(
        "foo bar baz",
        vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
    );
}

#[test]
fn test_nested_inline_list() {
    // They shouldn't be parseable.
    not_parsed::<Vec<Vec<u32>>>("1 2 3");
    not_parsed::<Vec<Vec<String>>>("foo bar baz");
}

#[test]
fn test_simple_vertical_list() {
    test_no_pp(
        "\
\t1
\t2
\t3",
        vec![1u32, 2u32, 3u32],
    );
}

#[test]
fn test_string_block() {
    test(
        "\
\tEs brillig war. Die schlichte Toven
\tWirrten und wimmelten in Waben;",
        "Es brillig war. Die schlichte Toven\nWirrten und wimmelten in Waben;\n".to_string(),
    );
}

#[test]
fn test_string_list() {
    // Vertical list
    test(
        "\
\tfoo bar
\tbaz",
        vec!["foo bar".to_string(), "baz".to_string()],
    );

    // XXX: Should this be made to be an error?
    //        // Must have comma before the nested item.
    //        not_parsed::<Vec<String>>(
    //            "\
    //\tfoo
    //\t\tbar
    //\t\tbaz",
    //        );
}

/* FIXME: Get this working
    #[test]
    fn test_nested_string_list() {
        test(
            r#"\
\tfoo bar
\tbaz xyzzy"#,
            vec![
                vec!["foo".to_string(), "bar".to_string()],
                vec!["baz".to_string(), "xyzzy".to_string()],
            ],
        );
    }
*/

#[test]
fn test_serialize_outline() {
    #[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
    struct OutlineContainer {
        content: Outline,
    }

    test(
        "\
\tcontent
\t\tfoo
\t\t\tbar",
        OutlineContainer {
            content: Outline::from(
                "\
foo
\tbar",
            ),
        },
    );
}

#[test]
fn test_comma() {
    // A single inline comma is a magic extra element to separate indented objects. It can be
    // escaped by doubling it (any sequence of more than 1 comma gets one comma removed from
    // it). A comma is just a comma in an multi-line string block, no escaping needed there.
    test(
        "\
\t\tEs brillig war. Die schlichte Toven
\t\tWirrten und wimmelten in Waben;
\t,
\t\tUnd aller-mümsige Burggoven
\t\tDie mohmen Räth' ausgraben.",
        vec![
            "Es brillig war. Die schlichte Toven\nWirrten und wimmelten in Waben;\n".to_string(),
            "Und aller-mümsige Burggoven\nDie mohmen Räth' ausgraben.\n".to_string(),
        ],
    );

    // An optional starting comma is allowed
    test_no_pp(
        "\
\t,
\t\tEs brillig war. Die schlichte Toven
\t\tWirrten und wimmelten in Waben;
\t,
\t\tUnd aller-mümsige Burggoven
\t\tDie mohmen Räth' ausgraben.",
        vec![
            "Es brillig war. Die schlichte Toven\nWirrten und wimmelten in Waben;\n".to_string(),
            "Und aller-mümsige Burggoven\nDie mohmen Räth' ausgraben.\n".to_string(),
        ],
    );

    // Need to also separate an indented block with comma
    test_no_pp(
        "\
\tfoo
\t,
\t\tbar
\t\tbaz",
        vec!["foo".to_string(), "bar\nbaz\n".to_string()],
    );
}

#[test]
fn test_escaped_comma() {
    // Double comma in vertical list becomes single comma
    test(
        "\
\tlorem ipsum
\t,,
\t,,,
\tdolor sit",
        vec![
            "lorem ipsum".to_string(),
            ",".to_string(),
            ",,".to_string(),
            "dolor sit".to_string(),
        ],
    );
}
