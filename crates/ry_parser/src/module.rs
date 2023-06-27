use crate::{items::ItemsParser, Parse, TokenIterator};
use ry_ast::Module;
use ry_diagnostics::CompilerDiagnostic;
use ry_interner::Interner;

/// Parse a Ry module.
pub fn parse_module<'a>(
    iterator: &'a mut TokenIterator<'a>,
) -> (Module<'a>, &'a Vec<CompilerDiagnostic>, &'a Interner) {
    let (global_docstring, first_docstring) = iterator.consume_module_and_first_item_docstrings();

    (
        Module {
            path: iterator.source_file.path(),
            file_id: iterator.file_id,
            docstring: global_docstring,
            items: ItemsParser { first_docstring }.parse_using(iterator),
        },
        &iterator.diagnostics,
        iterator.interner(),
    )
}
