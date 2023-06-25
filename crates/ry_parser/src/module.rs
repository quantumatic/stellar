use crate::{items::ItemsParser, Parse, TokenIterator};
use ry_ast::Module;
use ry_diagnostics::CompilerDiagnostic;
use ry_interner::Interner;
use ry_source_file::source_file::SourceFile;

/// Parses Ry module.
pub fn parse_module<'a>(
    file_id: usize,
    source_file: &'a SourceFile<'a>,
    interner: &'a mut Interner,
    diagnostics: &'a mut Vec<CompilerDiagnostic>,
) -> Module<'a> {
    parse_module_using(&mut TokenIterator::new(
        file_id,
        source_file,
        interner,
        diagnostics,
    ))
}

fn parse_module_using<'a>(iterator: &mut TokenIterator<'a>) -> Module<'a> {
    let (global_docstring, first_docstring) = iterator.consume_module_and_first_item_docstrings();

    Module {
        filepath: iterator.source_file.path(),
        docstring: global_docstring,
        items: ItemsParser { first_docstring }.parse_using(iterator),
    }
}
