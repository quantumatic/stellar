//! Defines [`ModuleScope`] to work with scopes that belong to a particular
//! Ry source file.

use pathdiff::diff_paths;
use ry_ast::typed::Type;
use ry_interner::{Interner, Symbol};
use ry_workspace::{file::SourceFile, span::Span, workspace::FileID};
use std::{
    collections::HashMap,
    path::{self, Component},
    sync::Arc,
};

/// Information that compiler has about a particular module.
#[derive(Debug, Clone)]
pub struct ModuleScope<'workspace> {
    /// Source file corresponding to the module.
    pub source_file: &'workspace SourceFile<'workspace>,

    /// File ID in the global workspace.
    pub file_id: FileID,

    /// Path to the module relative to the project root
    ///
    /// See [`Path`] for more details.
    pub module_path: Path,

    /// Imports inside the module.
    imports: Vec<ImportData>,

    /// Symbols in the module.
    symbols: HashMap<Symbol, ModuleSymbolData>,
}

impl<'workspace> ModuleScope<'workspace> {
    /// Creates a new [`ModuleScope`].
    #[inline]
    #[must_use]
    pub fn new(
        source_file: &'workspace SourceFile<'workspace>,
        file_id: FileID,
        module_path: Path,
    ) -> Self {
        Self {
            source_file,
            file_id,
            module_path,
            imports: vec![],
            symbols: HashMap::new(),
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
            parse_module_path(source_file.path, project_root, interner)?,
        ))
    }

    /// Returns the imports used by the module.
    #[inline]
    #[must_use]
    pub fn imports(&self) -> &[ImportData] {
        &self.imports
    }

    /// Adds an import to the module.
    #[inline]
    pub fn add_import(&mut self, import: ImportData) {
        self.imports.push(import);
    }

    #[inline]
    #[must_use]
    pub fn resolve(&self, symbol: Symbol) -> Option<&ModuleSymbolData> {
        self.symbols.get(&symbol)
    }
}

#[derive(Debug, Clone)]
pub enum ModuleSymbolData {
    Function(FunctionData),
    Struct {
        span: Span,
        fields: StructFieldsData,
    },
    TypeAlias {
        span: Span,
        value: Arc<Type>,
    },
    Enum {
        span: Span,
        variants: Vec<EnumVariantData>,
    },
    Trait {
        span: Span,
        items: Vec<TraitItemData>,
    },
}

#[derive(Debug, Clone)]
pub enum TraitItemData {
    Function(FunctionData),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionData {
    pub span: Span,
    pub generics: Vec<Symbol>,
    pub parameters: Vec<(Symbol, Arc<Type>)>,
    pub return_type: Arc<Type>,
    pub bounds: Vec<(Arc<Type>, TraitBounds)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TraitBounds {
    pub span: Span,
    pub bounds: Vec<Symbol>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnumVariantData {
    Just {
        span: Span,
        name: Symbol,
    },
    TupleLike {
        span: Span,
        fields: Vec<TupleLikeStructFieldData>,
    },
    Struct {
        span: Span,
        fields: StructFieldsData,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructFieldsData {
    TupleLikeStructFieldsData {
        span: Span,
        fields: Vec<TupleLikeStructFieldData>,
    },
    StructFieldsData {
        span: Span,
        fields: Vec<StructFieldData>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct TupleLikeStructFieldData {
    pub span: Span,
    pub ty: Arc<Type>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructFieldData {
    pub span: Span,
    pub name: Symbol,
    pub ty: Arc<Type>,
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
    pub symbols: Vec<Symbol>,
}

/// Used to store information about imports in global scopes.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImportData {
    /// Span of the entire import path (not including `import` keyword).
    pub span: Span,

    /// A regular path (left part of the import path).
    ///
    /// ```txt
    /// import std.io.*;
    ///        ^^^^^^
    ///
    /// import std.io as myio;
    ///        ^^^^^^
    /// ```
    pub path: Path,

    /// Span of the `*` symbol.
    pub star: Option<Span>,

    /// Span of the `as` right-hand side (not including `as` keyword).
    pub r#as: Option<Span>,
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
