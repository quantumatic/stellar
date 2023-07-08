use ry_manifest::TomlDependency::Simple;
use ry_manifest::{parse_manifest, TomlManifest, TomlProject};
use std::collections::HashMap;

#[test]
fn simple_manifest() {
    let manifest = "[project]
name = \"json\"
version = \"1.0.0\"";
    assert_eq!(
        parse_manifest(manifest),
        Ok(TomlManifest {
            project: TomlProject {
                name: "json".to_owned(),
                version: "1.0.0".to_owned(),
                description: None,
                keywords: None,
                categories: None,
                license: None,
                authors: None,
                repository: None,
            },
            dependencies: None,
        })
    );
}

#[test]
fn full_project_metadata() {
    let manifest = "[project]
name = \"json\"
version = \"1.0.0\"
authors = [\"Adi Salimgereyev\"]
repository = \"github.com/ry-lang/ry\"
license = \"MIT\"
description = \"Parses and processes json.\"
categories = [\"json\", \"parser\", \"serialization\", \"deserialization\",
\"config\"]
keywords = [\"json\", \"config\"]";

    assert_eq!(
        parse_manifest(manifest),
        Ok(TomlManifest {
            project: TomlProject {
                name: "json".to_owned(),
                version: "1.0.0".to_owned(),
                description: Some("Parses and processes json.".to_owned()),
                authors: Some(vec!["Adi Salimgereyev".to_owned()]),
                repository: Some("github.com/ry-lang/ry".to_owned()),
                license: Some("MIT".to_owned()),
                keywords: Some(vec!["json".to_owned(), "config".to_owned()]),
                categories: Some(vec![
                    "json".to_owned(),
                    "parser".to_owned(),
                    "serialization".to_owned(),
                    "deserialization".to_owned(),
                    "config".to_owned()
                ]),
            },
            dependencies: None,
        })
    )
}

#[test]
fn dependencies() {
    let manifest = "[project]
name = \"json\"
version = \"1.0.0\"

[dependencies]
foo = \"1.0\"";

    let mut deps = HashMap::new();
    deps.insert("foo".to_owned(), Simple("1.0".to_owned()));

    assert_eq!(
        parse_manifest(manifest),
        Ok(TomlManifest {
            project: TomlProject {
                name: "json".to_owned(),
                version: "1.0.0".to_owned(),
                description: None,
                keywords: None,
                categories: None,
                license: None,
                authors: None,
                repository: None,
            },
            dependencies: Some(deps),
        })
    );
}
