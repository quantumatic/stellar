use pathdiff::diff_paths;
use ry_interner::{Interner, Symbol};
use std::path::{self, Component};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    /// Path to the module relative to the project root
    ///
    /// See [`Path`] for more details.
    module_path: Path,
}

impl Environment {
    #[inline]
    #[must_use]
    pub const fn new(module_path: Path) -> Self {
        Self {
            module_path,
        }
    }
}

/// Used to store name information in global scopes, because the
/// compiler needs to deal with namespaces.
///
/// This is why [`Path`] is not just a wrapper over [`Symbol`], but over a
/// list of [`Symbol`]s.
///
/// Also [`Path`] is analogous to [`ry_ast::Path`], but without storing
/// source locations for each symbol.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Path {
    /// Symbols in the path.
    pub symbols: Vec<Symbol>,
}

/// The error occurs when trying to parse a module path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseModulePathError {
    /// When the module's source file path is not relative to the project
    /// root path.
    ModuleDoesNotBelongToProject,

    /// When project's path is not canonical.
    ///
    /// [`fs::canonicalize`] must be used to prevent the error.
    ProjectPathIsNotCanonical,

    /// When project's path is not valid UTF-8.
    ProjectPathIsNotValidUtf8,

    /// When the path of the source file corresponding to the module
    /// is not canonical.
    ///
    /// [`fs::canonicalize`] must be used to prevent the error.
    SourceFilePathIsNotCanonical,

    /// When the path of the source file corresponding to the module
    /// is not valid UTF-8.
    SourceFilePathIsNotValidUtf8,

    /// When the path of the source file corresponding to the module
    /// does not end with `.ry`.
    SourceFileDoesNotHaveRyExtension,
}

impl From<GetProjectNameError> for ParseModulePathError {
    fn from(error: GetProjectNameError) -> Self {
        match error {
            GetProjectNameError::ProjectPathIsNotCanonical => Self::ProjectPathIsNotCanonical,
            GetProjectNameError::ProjectPathIsNotValidUtf8 => Self::ProjectPathIsNotValidUtf8,
        }
    }
}

/// Returns the module path from an absolute source file path corresponding to
/// the module.
#[allow(clippy::missing_errors_doc)]
pub fn parse_module_path<F, P>(
    file_path: F,
    project_root: P,
    interner: &mut Interner,
) -> Result<Path, ParseModulePathError>
where
    F: AsRef<path::Path>,
    P: AsRef<path::Path>,
{
    let Some(relative_path) = diff_paths(file_path, project_root.as_ref()) else {
        return Err(ParseModulePathError::ModuleDoesNotBelongToProject);
    };

    let project_name_symbol = match get_project_name_from_path(project_root.as_ref(), interner) {
        Ok(project_name_symbol) => project_name_symbol,
        Err(error) => return Err(ParseModulePathError::from(error)),
    };

    let mut module_path_symbols = vec![project_name_symbol];

    let mut components = relative_path.components().peekable();

    while let Some(component) = components.next() {
        match component {
            Component::CurDir
            | Component::Prefix(..)
            | Component::RootDir
            | Component::ParentDir => {
                return Err(ParseModulePathError::ProjectPathIsNotCanonical);
            }
            Component::Normal(component) => {
                let Some(component_str) = component.to_str() else {
                        return Err(ParseModulePathError::ProjectPathIsNotValidUtf8);
                    };

                // last file
                if components.peek().is_none() {
                    if !path::Path::new(component_str)
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("ry"))
                    {
                        return Err(ParseModulePathError::SourceFileDoesNotHaveRyExtension);
                    }

                    let mut component = component_str.to_owned();
                    for _ in 0..3 {
                        component.remove(component.len() - 1);
                    }

                    module_path_symbols.push(interner.get_or_intern(component));
                } else {
                    module_path_symbols.push(interner.get_or_intern(component_str));
                }
            }
        }
    }

    Ok(Path {
        symbols: module_path_symbols,
    })
}

/// The error occurs, when trying to get project name out of its
/// canonical path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetProjectNameError {
    /// When project's path is not canonical.
    ///
    /// [`fs::canonicalize`] must be used to prevent the error.
    ProjectPathIsNotCanonical,

    /// When project's path is not valid UTF-8.
    ProjectPathIsNotValidUtf8,
}

/// Gets the project name from a its root path.
#[allow(clippy::missing_errors_doc)]
pub fn get_project_name_from_path<P>(
    path: P,
    interner: &mut Interner,
) -> Result<Symbol, GetProjectNameError>
where
    P: AsRef<path::Path>,
{
    match path.as_ref().components().last() {
        Some(Component::Normal(name)) => {
            let Some(name) = name.to_str() else {
                return Err(GetProjectNameError::ProjectPathIsNotValidUtf8);
            };

            Ok(interner.get_or_intern(name))
        }
        Some(
            Component::RootDir | Component::Prefix(..) | Component::ParentDir | Component::CurDir,
        )
        | None => Err(GetProjectNameError::ProjectPathIsNotCanonical),
    }
}
