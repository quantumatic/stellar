//! Defines [`ModuleScope`] to work with scopes that belong to a particular
//! Ry source file.

use pathdiff::diff_paths;
use ry_interner::{Interner, Symbol};
use ry_workspace::{file::SourceFile, span::Span, workspace::FileID};
use std::path::{self, Component};

/// Information that compiler has about a particular module.
#[derive(Debug, Clone)]
pub struct ModuleScope<'workspace> {
    /// Source file corresponding to the module.
    source_file: &'workspace SourceFile<'workspace>,

    /// File ID in the global workspace.
    file_id: FileID,

    /// Path to the module relative to the project root
    ///
    /// See [`Path`] for more details.
    module_path: Path,

    /// Imports inside the module.
    imports: Vec<Import>,
}

impl<'workspace> ModuleScope<'workspace> {
    /// Creates a new [`ModuleScope`].
    #[inline]
    #[must_use]
    pub const fn new(
        source_file: &'workspace SourceFile<'workspace>,
        file_id: FileID,
        module_path: Path,
    ) -> Self {
        Self {
            source_file,
            file_id,
            module_path,
            imports: vec![],
        }
    }

    /// Creates a new [`ModuleScope`] from a [`SourceFile`] and a project root path.
    #[allow(clippy::missing_errors_doc)]
    pub fn new_from_project_root<P>(
        source_file: &'workspace SourceFile<'workspace>,
        file_id: FileID,
        project_root: P,
        interner: &mut Interner,
    ) -> Result<Self, ParseModulePathError>
    where
        P: AsRef<path::Path>,
    {
        Ok(Self::new(
            source_file,
            file_id,
            parse_module_path(source_file.path(), project_root, interner)?,
        ))
    }

    /// Returns the imports used by the module.
    #[inline]
    #[must_use]
    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    /// Returns the file ID of the module.
    #[inline]
    #[must_use]
    pub const fn file_id(&self) -> FileID {
        self.file_id
    }

    /// Returns the module's source file.
    #[inline]
    #[must_use]
    pub const fn source_file(&self) -> &'workspace SourceFile<'workspace> {
        self.source_file
    }

    /// Returns the path to the module relative to the project root.
    #[inline]
    #[must_use]
    pub const fn module_path(&self) -> &Path {
        &self.module_path
    }

    /// Adds an import to the module.
    #[inline]
    pub fn add_import(&mut self, import: Import) {
        self.imports.push(import);
    }
}

/// Used to store name information in global scopes, because the
/// compiler needs to deal with namespaces.
///
/// This is why [`Path`] is not just a wrapper over [`Symbol`], but over a
/// list of [`Symbol`].
///
/// Also [`Path`] is analogous to [`ry_ast::Path`], but without storing
/// source locations for each symbol.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Path {
    /// Symbols in the path.
    symbols: Vec<Symbol>,
}

impl Path {
    /// Creates a new [`Path`] instance.
    #[inline]
    #[must_use]
    pub const fn new(symbols: Vec<Symbol>) -> Self {
        Self { symbols }
    }

    /// Returns the symbols in the path.
    #[inline]
    #[must_use]
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// Returns the number of symbols in the path.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Returns `true` if the path is empty (there are no symbols in it).
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

/// Used to store information about imports in global scopes.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Import {
    /// Span of the entire import path (not including `import` keyword).
    span: Span,

    /// A regular path (left part of the import path).
    ///
    /// ```txt
    /// import std.io.*;
    ///        ^^^^^^
    ///
    /// import std.io as myio;
    ///        ^^^^^^
    /// ```
    path: Path,

    /// Span of the `*` symbol.
    star: Option<Span>,

    /// Span of the `as` right-hand side (not including `as` keyword).
    r#as: Option<Span>,
}

impl Import {
    /// Creates a new [`Import`] instance.
    #[inline]
    #[must_use]
    pub const fn new(span: Span, path: Path, star: Option<Span>, r#as: Option<Span>) -> Self {
        Self {
            span,
            path,
            star,
            r#as,
        }
    }

    /// Returns the span of the entire import path (not including `import` keyword).
    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns a left/regular part of the import path.
    ///
    /// ```txt
    /// import std.io.*;
    ///        ^^^^^^
    ///
    /// import std.io as myio;
    ///        ^^^^^^
    /// ```
    #[inline]
    #[must_use]
    pub const fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the span of the `*` symbol.
    #[inline]
    #[must_use]
    pub const fn star(&self) -> Option<Span> {
        self.star
    }

    /// Returns the span of the `as` right-hand side (not including `as` keyword).
    #[inline]
    #[must_use]
    pub const fn r#as(&self) -> Option<Span> {
        self.r#as
    }
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
                return Err(ParseModulePathError::SourceFilePathIsNotCanonical);
            }
            Component::Normal(component) => {
                let Some(component_str) = component.to_str() else {
                        return Err(ParseModulePathError::SourceFilePathIsNotValidUtf8);
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

    Ok(Path::new(module_path_symbols))
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
