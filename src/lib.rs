use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    memory_max: u32,
    proc_size_max: u32,
    num_proc: u32,
    max_proc_time: u32,
}

// this would have been a lot cleaner if I used the serde library
// but I opted not too, for your sanity's sake.
pub fn parse_config(s: &str) -> Option<Config> {
    let conf: HashMap<String, String> = s
        .lines()
        .map(|line| {
            line.chars()
                .filter(|i| !i.is_whitespace())
                .collect::<String>()
        })
        .filter(|i| !i.starts_with("#"))
        .map(|line| line.to_lowercase())
        .map(|pair| {
            pair.split("=")
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
        })
        .map(|pair| dbg!(pair).try_into().map(|[a, b]: [String; 2]| (a, b)))
        .collect::<Result<HashMap<_, _>, _>>()
        .ok()?;
    Some(Config {
        memory_max: conf
            .get("memory_max")
            .map(|i| i.parse().expect("COULDN'T PARSE MEMORY_MAX"))
            .unwrap_or(1024),
        proc_size_max: conf
            .get("proc_size_max")
            .map(|i| i.parse().expect("COULDN'T PARSE PROC_SIZE_MAX"))
            .unwrap_or(1024),
        num_proc: conf
            .get("num_proc")
            .map(|i| i.parse().expect("COULDN'T PARSE NUM_PROC"))
            .unwrap_or(10),
        max_proc_time: conf
            .get("max_proc_time")
            .map(|i| i.parse().expect("COULDN'T PARSE MAX_PROC_TIME"))
            .unwrap_or(10_000),
    })
}

#[test]
fn test_parse_config() {
    assert_eq!(
        parse_config(
            "# test test test
            # comment
            memory_MAx = 32
            proc_size_max = 78
            num_proc = 32
            max_proc_time = 9822"
        ),
        Some(Config {
            memory_max: 32,
            proc_size_max: 78,
            num_proc: 32,
            max_proc_time: 9822,
        })
    )
}
