use ry_manifest::TomlDependency;
use ry_manifest::{parse_manifest, DetailedTomlDependency, TomlManifest, TomlProject};
use std::collections::BTreeMap;

#[test]
fn simple_manifest() {
    let mut warnings = vec![];
    let manifest = "[project]
name = \"json\"
version = \"1.0.0\"";
    assert_eq!(
        parse_manifest(manifest, &mut warnings),
        Ok(TomlManifest {
            project: TomlProject {
                name: "json".to_owned(),
                version: "1.0.0".to_owned(),
                description: None,
                keywords: None,
                categories: None,
                license: None,
                author: None,
                repository: None,
            },
            dependencies: None,
        })
    );
    assert_eq!(warnings.len(), 0);
}

#[test]
fn full_project_metadata() {
    let mut warnings = vec![];
    let manifest = "[project]
name = \"json\"
version = \"1.0.0\"
author = \"abs0luty\"
repository = \"github.com/ry-lang/ry\"
license = \"MIT\"
description = \"Parses and processes json.\"
categories = [\"json\", \"parser\", \"serialization\", \"deserialization\",
\"config\"]
keywords = [\"json\", \"config\"]";

    assert_eq!(
        parse_manifest(manifest, &mut warnings),
        Ok(TomlManifest {
            project: TomlProject {
                name: "json".to_owned(),
                version: "1.0.0".to_owned(),
                description: Some("Parses and processes json.".to_owned()),
                author: Some("abs0luty".to_owned()),
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
    );
    assert_eq!(warnings.len(), 0);
}

#[test]
fn dependencies() {
    let mut warnings = vec![];
    let manifest = "[project]
name = \"json\"
version = \"1.0.0\"
author = \"abs0luty\"

[dependencies]
foo = \"1.0\"
bar = { version = \"1.0\", author = \"abs0luty\" }
foo2 = { path = \"../foo\" }";

    let mut deps = BTreeMap::new();
    deps.insert("foo".to_owned(), TomlDependency::Simple("1.0".to_owned()));
    deps.insert(
        "bar".to_owned(),
        TomlDependency::Detailed(DetailedTomlDependency {
            version: Some("1.0".to_owned()),
            author: Some("abs0luty".to_owned()),
            path: None,
        }),
    );
    deps.insert(
        "foo2".to_owned(),
        TomlDependency::Detailed(DetailedTomlDependency {
            path: Some("../foo".to_owned()),
            author: None,
            version: None,
        }),
    );

    assert_eq!(
        parse_manifest(manifest, &mut warnings),
        Ok(TomlManifest {
            project: TomlProject {
                name: "json".to_owned(),
                version: "1.0.0".to_owned(),
                author: Some("abs0luty".to_owned()),
                description: None,
                keywords: None,
                categories: None,
                license: None,
                repository: None,
            },
            dependencies: Some(deps),
        })
    );
    assert_eq!(warnings.len(), 0);
}
