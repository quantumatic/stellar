use ry_config::{parse_config_from_source, ConfigTable};

#[test]
fn name() {
    let config = parse_config_from_source(
        "name = \"json\"
version = \"1.0.0\"",
    );
    assert_eq!(
        config,
        Ok(ConfigTable {
            name: "json".to_owned(),
            version: "1.0.0".to_owned(),
            author: None,
            license: None,
            description: None,
            repository: None,
            homepage: None,
            dependencies: None,
        })
    )
}
