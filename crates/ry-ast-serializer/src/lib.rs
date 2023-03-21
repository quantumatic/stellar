use ry_ast::{location::WithSpan, visitor::Visitor, *};
use string_interner::{DefaultSymbol, StringInterner};

pub struct ASTSerializer<'a> {
    content: String,
    indent: usize,
    identifier_interner: &'a StringInterner,
}

impl<'a> ASTSerializer<'a> {
    pub fn new(identifier_interner: &'a StringInterner) -> Self {
        Self {
            content: "".to_owned(),
            indent: 0,
            identifier_interner,
        }
    }

    fn indent(&mut self) {
        self.content += &"  ".repeat(self.indent);
    }

    fn new_line(&mut self) {
        self.content += "\n";

        self.indent();
    }

    fn write_ident(&mut self, symbol: DefaultSymbol) {
        self.content
            .push_str(self.identifier_interner.resolve(symbol).unwrap());
    }

    pub fn serialize(&mut self, ast: &ProgramUnit) -> &str {
        self.visit_program_unit(ast);
        &self.content
    }
}

impl<'a> Visitor for ASTSerializer<'a> {
    fn visit_import(&mut self, node: &ry_ast::Import) {
        self.content += "import ";
        self.visit_static_name(&node.path);
        self.content += ";\n";

        self.walk_import(node);
    }

    fn visit_static_name(&mut self, node: &WithSpan<Vec<DefaultSymbol>>) {
        for ident in &node.value {
            self.write_ident(*ident);
            self.content += "::";
        }

        self.content.truncate(self.content.len() - 2);
    }

    fn visit_item(&mut self, node: (&str, &Item)) {
        self.walk_item(node);

        self.new_line();
        self.new_line();
    }

    fn visit_enum_decl(&mut self, node: (&str, &EnumDecl)) {
        if node.1.public.is_some() {
            self.content += "pub ";
        }

        self.content += "enum ";
        self.write_ident(node.1.name.value);

        if node.1.variants.is_empty() {
            self.content += " {}";
        } else {
            self.content += " {";

            self.indent += 1;
            self.new_line();
            for variant in &node.1.variants {
                self.write_ident(variant.1.value);
                self.content += ",";

                self.new_line();
            }

            self.content.truncate(self.content.len() - 4);

            self.indent -= 1;
            self.new_line();

            self.content += "}";
        }
    }

    fn visit_function_decl(&mut self, node: (&str, &FunctionDecl)) {
        if node.1.def.public.is_some() {
            self.content += "pub ";
        }

        self.content += "fun ";
        self.write_ident(node.1.def.name.value);

        self.visit_generic_annotations(&node.1.def.generic_annotations);

        self.visit_arguments(&node.1.def.params);

        self.content += " ";

        self.visit_block(&node.1.stmts);
    }

    fn visit_generic_annotations(&mut self, node: &GenericAnnotations) {
        if node.len() != 0 {
            self.content.push('[');

            for annotation in node {
                self.visit_generic_annotation(annotation);
            }

            self.content.push(']');
        }
    }

    fn visit_generic_annotation(&mut self, node: &GenericAnnotation) {
        self.content += self.identifier_interner.resolve(node.0.value).unwrap();

        if let Some(constraint) = &node.1 {
            self.visit_type(constraint);
        }
    }

    fn visit_arguments(&mut self, node: &Vec<FunctionParam>) {
        self.content += "(";
        self.walk_arguments(node);
        self.content.truncate(self.content.len() - 2);
        self.content += ")";
    }

    fn visit_argument(&mut self, node: &FunctionParam) {
        self.content += self.identifier_interner.resolve(node.name.value).unwrap();
        self.content += " ";
        self.visit_type(&node.r#type);

        if let Some(default_value) = &node.default_value {
            self.content += " = ";
            self.visit_expression(default_value);
        }

        self.content += ", ";
    }

    fn visit_block(&mut self, node: &Vec<Statement>) {
        self.content += "{";
        self.indent += 1;
        self.new_line();

        self.walk_block(node);

        self.content += "}";
        self.indent -= 1;
    }

    fn visit_expression_statement(&mut self, node: (bool, &Expression)) {
        self.visit_expression(node.1);

        if node.0 {
            self.content += ";";
        }

        self.content += "\n";
    }

    fn visit_array(&mut self, node: &Type) {
        self.content += "[";
        self.visit_type(node);
        self.content += "]";
    }

    fn visit_option(&mut self, node: &Type) {
        self.visit_type(node);
        self.content += "?";
    }

    fn visit_reference(&mut self, node: (bool, &Type)) {
        self.content += "&";

        if node.0 {
            self.content += "mut";
        }

        self.content += " ";
        self.visit_type(node.1);
    }

    fn visit_primary(&mut self, node: (&WithSpan<Vec<DefaultSymbol>>, &Vec<Type>)) {
        self.visit_static_name(node.0);

        self.visit_generic_annotations_in_type(node.1);
    }

    fn visit_generic_annotations_in_type(&mut self, node: &Vec<Type>) {
        if !node.is_empty() {
            self.content += "[";

            for r#type in node {
                self.visit_type(r#type);
                self.content += ", ";
            }

            self.content.truncate(self.content.len() - 2);

            self.content += "]";
        }
    }
}
