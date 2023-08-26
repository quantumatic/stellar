use ry_manifest::TomlPackage;
use ry_manifest::{parse_manifest, TomlDependency, TomlManifest};

#[test]
fn simple_manifest() {
    let manifest = "[package]
name = \"json\"
version = \"1.0.0\"";
    assert_eq!(
        parse_manifest(manifest),
        Ok(TomlManifest::new(TomlPackage::new("json", "1.0.0")))
    );
}

#[test]
fn full_package_metadata() {
    let manifest = "[package]
name = \"json\"
version = \"1.0.0\"
author = \"abs0luty\"
repository = \"github.com/quantumatic/ry\"
license = \"MIT\"
description = \"Parses and processes json.\"
categories = [\"json\", \"parser\", \"serialization\", \"deserialization\",
\"config\"]
keywords = [\"json\", \"config\"]";

    assert_eq!(
        parse_manifest(manifest),
        Ok(TomlManifest::new(
            TomlPackage::new("json", "1.0.0")
                .with_author("abs0luty")
                .with_repository("github.com/quantumatic/ry")
                .with_license("MIT")
                .with_description("Parses and processes json.")
                .with_categories([
                    "json",
                    "parser",
                    "serialization",
                    "deserialization",
                    "config"
                ])
                .with_keywords(["json", "config"])
        ))
    );
}

#[test]
fn dependencies() {
    let manifest = "[package]
name = \"json\"
version = \"1.0.0\"
author = \"abs0luty\"

[dependencies]
foo = { version = \"1.0\", author = \"foo\" }
bar = { version = \"1.0\", author = \"bar\" }
foo2 = { path = \"../foo\" }";

    assert_eq!(
        parse_manifest(manifest),
        Ok(
            TomlManifest::new(TomlPackage::new("json", "1.0.0").with_author("abs0luty"))
                .with_dependencies([
                    (
                        "foo",
                        TomlDependency::new().with_version("1.0").with_author("foo")
                    ),
                    (
                        "bar",
                        TomlDependency::new().with_version("1.0").with_author("bar")
                    ),
                    ("foo2", TomlDependency::new().with_path("../foo")),
                ])
        )
    );
}
