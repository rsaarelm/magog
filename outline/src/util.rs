/// Turn "Gödel, Escher, Bach" into "godel-escher-bach"
pub fn normalize_title(title: &str) -> String {
    let mut title: String = deunicode::deunicode(title);
    title.make_ascii_lowercase();

    let title = title
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    title
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_normalize_title() {
        assert_eq!(normalize_title("Gödel, Escher, Bach"), "godel-escher-bach");
        assert_eq!(
            normalize_title("Gödel, !$!@#^*!@ Escher, Bach"),
            "godel-escher-bach"
        );
        assert_eq!(
            normalize_title("  Gödel, Escher, Bach    "),
            "godel-escher-bach"
        );
    }
}
