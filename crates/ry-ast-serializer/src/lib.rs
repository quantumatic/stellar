use ry_ast::{location::WithSpan, visitor::Visitor, *};
use string_interner::{DefaultSymbol, StringInterner};

pub struct ASTSerializer<'a> {
    content: String,
    indent: usize,
    identifier_interner: &'a StringInterner,
}

impl<'a> ASTSerializer<'a> {
    fn new(indent: usize, identifier_interner: &'a StringInterner) -> Self {
        Self {
            content: "".to_owned(),
            indent: 4,
            identifier_interner,
        }
    }

    fn ident(&mut self) {
        self.content += &" ".repeat(self.indent);
    }

    fn write_ident(&mut self, symbol: DefaultSymbol) {
        self.content
            .push_str(self.identifier_interner.resolve(symbol).unwrap());
    }

    fn serialize(&mut self, ast: &ProgramUnit) -> &String {
        self.visit_program_unit(ast);
        &self.content
    }
}

impl<'a> Visitor for ASTSerializer<'a> {
    fn visit_import(&mut self, node: &ry_ast::Import) {
        self.content.push_str("import ");
        self.visit_static_name(&node.path);
        self.content.push_str(";\n");

        self.walk_import(node);
    }

    fn visit_static_name(&mut self, node: &WithSpan<Vec<DefaultSymbol>>) {
        for ident in &node.value {
            self.write_ident(*ident);
            self.content.push_str("::");
        }

        self.content.truncate(self.content.len() - 2);
    }

    fn visit_items(&mut self, node: &Items) {
        self.content.push('\n');

        self.walk_items(node);
    }

    fn visit_enum_decl(&mut self, node: (&str, &EnumDecl)) {
        if node.1.public.is_some() {
            self.content.push_str("pub ");
        }
        self.content.push_str("enum ");
        self.write_ident(node.1.name.value);
        self.content.push_str(" {\n");
        for variant in &node.1.variants {
            self.write_ident(variant.1.value);
            self.content.push_str(",\n");
        }

        self.content.truncate(self.content.len() - 2);
        self.content.push_str("\n}\n");
    }

    fn visit_function_decl(&mut self, node: (&str, &FunctionDecl)) {
        if node.1.def.public.is_some() {
            self.content.push_str("pub ");
        }

        self.content.push_str("fun ");
        self.write_ident(node.1.def.name.value);
    }

    fn visit_generic_annotations(&mut self, node: &ry_ast::GenericAnnotations) {
        if node.len() != 0 {
            self.content.push('[');

            for annotation in node {
                self.walk_generic_annotation(annotation);
            }

            self.content.push(']');
        }
    }

    fn visit_generic_annotation(&mut self, node: &ry_ast::GenericAnnotation) {
        self.content
            .push_str(self.identifier_interner.resolve(node.0.value).unwrap());

        if let Some(constraint) = &node.1 {
            self.visit_type(constraint);
        }
    }
}
