use pathdiff::diff_paths;
use ry_interner::{Interner, Symbol};
use ry_workspace::{file::SourceFile, span::Span, workspace::FileID};
use std::{
    ffi::OsString,
    path::{self, Component, PathBuf},
};

#[derive(Debug, Clone)]
pub struct ModuleScope<'workspace> {
    source_file: &'workspace SourceFile<'workspace>,
    file_id: FileID,
    module_path: Path,
    imports: Vec<Import>,
}

impl<'workspace> ModuleScope<'workspace> {
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
    ///
    /// # Errors
    ///
    /// Errors are the same as when calling [`parse_module_path_using_project_root()`],
    /// because the method uses it to get a module path and then constructs a new [`ModuleScope`].
    pub fn new_from_project_root<P>(
        source_file: &'workspace SourceFile<'workspace>,
        file_id: FileID,
        project_root: P,
        interner: &mut Interner,
    ) -> Result<Self, ParseModulePathUsingProjectRootError>
    where
        P: AsRef<path::Path>,
    {
        Ok(Self::new(
            source_file,
            file_id,
            parse_module_path_using_project_root(source_file.path(), project_root, interner)?,
        ))
    }

    #[inline]
    #[must_use]
    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    #[inline]
    #[must_use]
    pub const fn file_id(&self) -> FileID {
        self.file_id
    }

    #[inline]
    #[must_use]
    pub const fn source_file(&self) -> &'workspace SourceFile<'workspace> {
        self.source_file
    }

    #[inline]
    #[must_use]
    pub const fn absolute_path(&self) -> &Path {
        &self.module_path
    }

    #[inline]
    pub fn add_import(&mut self, import: Import) {
        self.imports.push(import);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Path {
    symbols: Vec<Symbol>,
}

impl Path {
    #[inline]
    #[must_use]
    pub const fn new(symbols: Vec<Symbol>) -> Self {
        Self { symbols }
    }

    #[inline]
    #[must_use]
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Import {
    span: Span,
    path: Path,
    all: Option<Span>,
    r#as: Option<Span>,
}

impl Import {
    #[inline]
    #[must_use]
    pub const fn new(span: Span, path: Path, all: Option<Span>, r#as: Option<Span>) -> Self {
        Self {
            span,
            path,
            all,
            r#as,
        }
    }

    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    #[inline]
    #[must_use]
    pub const fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    #[must_use]
    pub const fn all(&self) -> Option<Span> {
        self.all
    }

    #[inline]
    #[must_use]
    pub const fn r#as(&self) -> Option<Span> {
        self.r#as
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseModulePathUsingProjectRootError {
    GotRelativeInsteadOfAbsolutePath,
    CannotGetRelativePath,
    ParseModulePathError(ParseModulePathError),
    ExtractProjectNameError(ExtractProjectNameError),
}

impl From<ParseModulePathError> for ParseModulePathUsingProjectRootError {
    fn from(error: ParseModulePathError) -> Self {
        Self::ParseModulePathError(error)
    }
}

impl From<ExtractProjectNameError> for ParseModulePathUsingProjectRootError {
    fn from(error: ExtractProjectNameError) -> Self {
        Self::ExtractProjectNameError(error)
    }
}

/// Returns the module path from an absolute source file path corresponding to
/// the module.
///
/// # Errors
///
/// * [`CannotGetRelativePath`] - when module path is not relative to the
/// project root.
/// * [`ExtractProjectNameError`] - when project project name cannot be extracted
/// from the project root absolute path.
/// * [`ParseModulePathError`] - when module path cannot be parsed, due to other reasons.
/// See the enum documentation for more details.
///
/// [`CannotGetRelativePath`]: ParseModulePathUsingProjectRootError::CannotGetRelativePath
/// [`ParseModulePathError`]: ParseModulePathUsingProjectRootError::ParseModulePathError
/// [`ExtractProjectNameError`]: ParseModulePathUsingProjectRootError::ExtractProjectNameError
pub fn parse_module_path_using_project_root<F, P>(
    file_path: F,
    project_root: P,
    interner: &mut Interner,
) -> Result<Path, ParseModulePathUsingProjectRootError>
where
    F: AsRef<path::Path>,
    P: AsRef<path::Path>,
{
    if !file_path.as_ref().is_absolute() || !project_root.as_ref().is_absolute() {
        return Err(ParseModulePathUsingProjectRootError::GotRelativeInsteadOfAbsolutePath);
    }

    let Some(relative_path) = diff_paths(file_path, project_root.as_ref()) else {
        return Err(ParseModulePathUsingProjectRootError::CannotGetRelativePath);
    };

    let project_name_symbol = match extract_project_name_from_path(project_root.as_ref(), interner)
    {
        Ok(project_name_symbol) => project_name_symbol,
        Err(error) => {
            return Err(ParseModulePathUsingProjectRootError::ExtractProjectNameError(error))
        }
    };

    match extract_module_path_using_relative_file_path(relative_path, project_name_symbol, interner)
    {
        Ok(path) => Ok(path),
        Err(error) => Err(ParseModulePathUsingProjectRootError::ParseModulePathError(
            error,
        )),
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseModulePathError {
    RootDirComponentIsNotAllowed,
    PrefixComponentIsNotAllowed(OsString),
    ParentDirComponentIsNotAllowed,
    InvalidUTF8PathComponent(OsString),
    InvalidFileExtensionOrFolder(String),
}

fn extract_module_path_using_relative_file_path<P>(
    module_relative_path: P,
    project_name_symbol: Symbol,
    interner: &mut Interner,
) -> Result<Path, ParseModulePathError>
where
    P: AsRef<path::Path>,
{
    let mut path_symbols = vec![project_name_symbol];

    let mut components = module_relative_path.as_ref().components().peekable();

    while let Some(component) = components.next() {
        match component {
            Component::CurDir => continue,
            Component::Prefix(prefix) => {
                return Err(ParseModulePathError::PrefixComponentIsNotAllowed(
                    prefix.as_os_str().to_owned(),
                ));
            }
            Component::RootDir => {
                return Err(ParseModulePathError::RootDirComponentIsNotAllowed);
            }
            Component::ParentDir => {
                return Err(ParseModulePathError::ParentDirComponentIsNotAllowed);
            }
            Component::Normal(component) => {
                let Some(component_str) = component.to_str() else {
                        return Err(ParseModulePathError::InvalidUTF8PathComponent(component.to_owned()));
                    };

                // last file
                if components.peek().is_none() {
                    if !path::Path::new(component_str)
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("ry"))
                    {
                        return Err(ParseModulePathError::InvalidFileExtensionOrFolder(
                            component_str.to_owned(),
                        ));
                    }

                    let mut component = component_str.to_owned();
                    for _ in 0..3 {
                        component.remove(component.len() - 1);
                    }

                    path_symbols.push(interner.get_or_intern(component));
                } else {
                    path_symbols.push(interner.get_or_intern(component_str));
                }
            }
        }
    }

    Ok(Path::new(path_symbols))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractProjectNameError {
    RootDirComponentIsNotAllowed,
    PrefixComponentIsNotAllowed(OsString),
    ParentDirComponentIsNotAllowed,
    InvalidUTF8PathComponent(OsString),
    EmptyPath,
}

fn extract_project_name_from_path<P>(
    path: P,
    interner: &mut Interner,
) -> Result<Symbol, ExtractProjectNameError>
where
    P: AsRef<path::Path>,
{
    match path.as_ref().components().last() {
        Some(Component::CurDir) => {
            let mut path_buf = PathBuf::from(path.as_ref());
            path_buf.pop();
            extract_project_name_from_path(path_buf, interner)
        }
        Some(Component::Normal(name)) => {
            let Some(name) = name.to_str() else {
                return Err(ExtractProjectNameError::InvalidUTF8PathComponent(name.to_owned()));
            };

            Ok(interner.get_or_intern(name))
        }
        Some(Component::RootDir) => Err(ExtractProjectNameError::RootDirComponentIsNotAllowed),
        Some(Component::Prefix(prefix)) => Err(
            ExtractProjectNameError::PrefixComponentIsNotAllowed(prefix.as_os_str().to_owned()),
        ),
        Some(Component::ParentDir) => Err(ExtractProjectNameError::ParentDirComponentIsNotAllowed),
        None => Err(ExtractProjectNameError::EmptyPath),
    }
}
