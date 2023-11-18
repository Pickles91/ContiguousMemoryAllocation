use std::collections::HashMap;

pub fn parse_config(s: &str) -> HashMap<String, String> {
    s
      .lines()
      .map(|line| line.trim())
      .map(|pair| pair.split("="))
      .map(|mut pair| (pair.next().unwrap().into(), pair.next().unwrap().into()))
      .collect()
}

#[test]
fn test_parse_config() {
    assert_eq!(
        parse_config(
            "foo=bar
            baz=qux
            qux=foobar"
        ),
        [("foo".into(),"bar".into()), ("baz".into(), "qux".into()), ("qux".into(), "foobar".into())].into_iter().collect()
    )
}
