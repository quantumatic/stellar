use std::collections::BTreeMap;

use ry_manifest::{parse_manifest, DetailedTomlDependency, TomlManifest};
use ry_manifest::{TomlDependency, TomlPackage};

#[test]
fn simple_manifest() {
    let manifest = "[package]
name = \"json\"
version = \"1.0.0\"";
    assert_eq!(
        parse_manifest(manifest),
        Ok(TomlManifest {
            package: TomlPackage {
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
}

#[test]
fn full_package_metadata() {
    let manifest = "[package]
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
        parse_manifest(manifest),
        Ok(TomlManifest {
            package: TomlPackage {
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
}

#[test]
fn dependencies() {
    let manifest = "[package]
name = \"json\"
version = \"1.0.0\"
author = \"abs0luty\"

[dependencies]
foo = \"1.0\"
bar = { version = \"1.0\" }
foo2 = { path = \"../foo\" }";

    let mut deps = BTreeMap::new();
    deps.insert("foo".to_owned(), TomlDependency::Simple("1.0".to_owned()));
    deps.insert(
        "bar".to_owned(),
        TomlDependency::Detailed(DetailedTomlDependency {
            version: Some("1.0".to_owned()),
            path: None,
        }),
    );
    deps.insert(
        "foo2".to_owned(),
        TomlDependency::Detailed(DetailedTomlDependency {
            path: Some("../foo".to_owned()),
            version: None,
        }),
    );

    assert_eq!(
        parse_manifest(manifest),
        Ok(TomlManifest {
            package: TomlPackage {
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
}
