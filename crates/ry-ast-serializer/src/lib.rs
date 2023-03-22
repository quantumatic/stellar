use ry_ast::{location::WithSpan, visitor::Visitor, *};
use string_interner::{DefaultSymbol, StringInterner};

pub struct ASTSerializer<'a> {
    content: String,
    indent: usize,
    string_interner: &'a StringInterner,
}

impl<'a> ASTSerializer<'a> {
    pub fn new(string_interner: &'a StringInterner) -> Self {
        Self {
            content: "".to_owned(),
            indent: 0,
            string_interner,
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
            .push_str(self.string_interner.resolve(symbol).unwrap());
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

    fn visit_item(&mut self, node: (&Docstring, &Item)) {
        self.walk_item(node);

        self.new_line();
        self.new_line();
    }

    fn visit_enum_decl(&mut self, node: (&Docstring, &EnumDecl)) {
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

    fn visit_function_decl(&mut self, node: (&Docstring, &FunctionDecl)) {
        if node.1.def.public.is_some() {
            self.content += "pub ";
        }

        self.content += "fun ";
        self.write_ident(node.1.def.name.value);

        self.visit_generic_annotations(&node.1.def.generic_annotations);

        self.visit_arguments(&node.1.def.params);

        if let Some(r) = &node.1.def.return_type {
            self.content += " ";
            self.visit_type(r);
        }

        self.content += " ";

        self.visit_where_clause(&node.1.def.r#where);

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
        self.content += self.string_interner.resolve(node.0.value).unwrap();

        if let Some(constraint) = &node.1 {
            self.content += " ";
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
        self.content += self.string_interner.resolve(node.name.value).unwrap();
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

        self.walk_block(node);

        self.indent -= 1;
        self.new_line();
        self.content += "}";
    }

    fn visit_expression_statement(&mut self, node: (bool, &Expression)) {
        self.new_line();

        self.visit_expression(node.1);

        if node.0 {
            self.content += ";";
        }
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

    fn visit_bool_literal(&mut self, node: bool) {
        if node {
            self.content += "true";
        } else {
            self.content += "false";
        }
    }

    fn visit_float_literal(&mut self, node: f64) {
        self.content += &node.to_string();
    }

    fn visit_integer_literal(&mut self, node: u64) {
        self.content += &node.to_string();
    }

    fn visit_string_literal(&mut self, node: DefaultSymbol) {
        self.content += "\"";
        self.content += self.string_interner.resolve(node).unwrap();
        self.content += "\"";
    }

    fn visit_char_literal(&mut self, node: char) {
        self.content += "'";
        self.content += &node.to_string();
        self.content += "'";
    }

    fn visit_binary_expression(&mut self, node: (&Expression, &token::Token, &Expression)) {
        self.visit_expression(node.0);
        self.content += " ";
        self.content += &node.1.value.dump_op();
        self.content += " ";
        self.visit_expression(node.2);
    }

    fn visit_prefix_or_postfix(&mut self, node: (bool, &token::Token, &Expression)) {
        if node.0 {
            self.content += &node.1.value.dump_op();
            self.visit_expression(node.2);
        } else {
            self.visit_expression(node.2);
            self.content += &node.1.value.dump_op();
        }
    }

    fn visit_call(&mut self, node: (&Vec<Type>, &Expression, &Vec<Expression>)) {
        self.visit_expression(node.1);
        self.visit_generics(node.0);
        self.visit_call_arguments(node.2);
    }

    fn visit_call_arguments(&mut self, node: &Vec<Expression>) {
        self.content += "(";

        for argument in node {
            self.visit_expression(argument);
            self.content += ", ";
        }

        self.content.truncate(self.content.len() - 2);

        self.content += ")";
    }

    fn visit_where_clause(&mut self, node: &WhereClause) {
        if node.is_empty() {
            return;
        }

        self.content += "where ";

        for cond in node {
            self.visit_type(&cond.0);

            self.content += " = ";

            self.visit_type(&cond.1);
        }

        self.content += " ";
    }

    fn visit_generics(&mut self, node: &Vec<Type>) {
        if node.is_empty() {
            return;
        }

        self.content += ".[";

        for generic in node {
            self.visit_type(generic);
            self.content += ", ";
        }

        self.content.truncate(self.content.len() - 2);

        self.content += "]";
    }

    fn visit_struct_decl(&mut self, node: (&Docstring, &StructDecl)) {
        if node.1.public.is_some() {
            self.content += "pub ";
        }

        self.content += "struct ";
        self.content += self.string_interner.resolve(node.1.name.value).unwrap();

        self.visit_generic_annotations(&node.1.generic_annotations);

        self.content += " ";

        self.visit_where_clause(&node.1.r#where);

        self.content += "{";
        self.indent += 1;

        for member in &node.1.members {
            self.new_line();

            if member.1.public.is_some() {
                self.content += "pub ";
            }

            if member.1.r#mut.is_some() {
                self.content += "mut ";
            }

            self.content += self.string_interner.resolve(member.1.name.value).unwrap();
            self.content += " ";
            self.visit_type(&member.1.r#type);
            self.content += ";";
        }

        self.indent -= 1;
        self.new_line();
        self.content += "}";
    }

    fn visit_impl(&mut self, node: (&Docstring, &Impl)) {
        self.content += "impl";
        self.visit_generic_annotations(&node.1.global_generic_annotations);
        self.content += " ";

        if let Some(t) = &node.1.r#trait {
            self.visit_type(t);
            self.content += " for ";
        }

        self.visit_type(&node.1.r#type);
        self.content += " {";
        self.indent += 1;

        self.visit_trait_methods_in_impl(&node.1.methods);

        self.indent -= 1;
        self.new_line();
        self.content += "}";
    }

    fn visit_trait_method_in_impl(&mut self, node: &(Docstring, TraitMethod)) {
        self.new_line();

        if node.1.public.is_some() {
            self.content += "pub ";
        }

        self.content += "fun ";
        self.content += self.string_interner.resolve(node.1.name.value).unwrap();

        self.visit_generic_annotations(&node.1.generic_annotations);

        self.visit_arguments(&node.1.params);

        self.content += " ";

        if let Some(r) = &node.1.return_type {
            self.visit_type(r);
            self.content += " ";
        }

        self.visit_where_clause(&node.1.r#where);

        self.visit_block(node.1.body.as_ref().unwrap());
    }
}
