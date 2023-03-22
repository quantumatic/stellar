use ry_proc_macros::visit_fn;
use string_interner::DefaultSymbol;

use crate::*;

pub trait Visitor: Sized {
    visit_fn!(program_unit for &ProgramUnit {
        for import in &node.imports {
            self.visit_import(import);
        }

        for item in &node.items {
            self.visit_item((&item.0, &item.1));
        }
    });

    visit_fn!(import for &Import);
    visit_fn!(import_after_first_item for &Import);

    visit_fn!(items for &Items {
        for item in node {
            self.visit_item((&item.0, &item.1));
        }
    });
    visit_fn!(item for (&Docstring, &Item) {
        match node.1 {
            Item::EnumDecl(e) => self.visit_enum_decl((node.0, e)),
            Item::FunctionDecl(f) => self.visit_function_decl((node.0, f)),
            Item::StructDecl(s) => self.visit_struct_decl((node.0, s)),
            Item::Impl(i) => self.visit_impl((node.0, i)),
            Item::TraitDecl(t) => self.visit_trait_decl((node.0, t)),
            Item::Import(i) => self.visit_import_after_first_item(i),
        }
    });

    visit_fn!(enum_decl for (&Docstring, &EnumDecl));
    visit_fn!(function_decl for (&Docstring, &FunctionDecl));
    visit_fn!(struct_decl for (&Docstring, &StructDecl));
    visit_fn!(trait_decl for (&Docstring, &TraitDecl));
    visit_fn!(r#impl for (&Docstring, &Impl));

    visit_fn!(generic_annotations for &GenericAnnotations);
    visit_fn!(generic_annotation for &GenericAnnotation);

    visit_fn!(generics for &Vec<Type> {
        for generic in node {
            self.visit_generic(generic);
        }
    });
    visit_fn!(generic for &Type {
        self.visit_type(node);
    });

    visit_fn!(arguments for &Vec<FunctionParam> {
        for argument in node {
            self.visit_argument(argument);
        }
    });

    visit_fn!(where_clause for &WhereClause);

    visit_fn!(argument for &FunctionParam);

    visit_fn!(r#type for &Type {
        match &*node.value {
            RawType::Array(inner) => self.visit_array(inner),
            RawType::NegativeTrait(r#trait) => self.visit_negative_trait(r#trait),
            RawType::Option(inner) => self.visit_option(inner),
            RawType::Primary(name, generics) => self.visit_primary((name, generics)),
            RawType::Reference(mutable, inner) => self.visit_reference((*mutable, inner)),
        }
    });

    visit_fn!(array for &Type);
    visit_fn!(option for &Type);
    visit_fn!(reference for (bool, &Type));
    visit_fn!(generic_annotations_in_type for &Vec<Type>);
    visit_fn!(primary for (&WithSpan<Vec<DefaultSymbol>>, &Vec<Type>) {
        self.visit_static_name(node.0);

        self.visit_generic_annotations_in_type(node.1);
    });
    visit_fn!(negative_trait for &Type);

    visit_fn!(trait_methods for &Vec<(Docstring, TraitMethod)> {
        for method in node {
            self.visit_trait_method(method);
        }
    });
    visit_fn!(trait_methods_in_impl for &Vec<(Docstring, TraitMethod)> {
        for method in node {
            self.visit_trait_method_in_impl(method);
        }
    });

    visit_fn!(trait_method_in_impl for &(Docstring, TraitMethod));
    visit_fn!(trait_method for &(Docstring, TraitMethod));

    visit_fn!(expression for &Expression {
        match &*node.value {
            RawExpression::Bool(b) => self.visit_bool_literal(*b),
            RawExpression::String(s) => self.visit_string_literal(*s),
            RawExpression::Int(i) => self.visit_integer_literal(*i),
            RawExpression::Float(f) => self.visit_float_literal(*f),
            RawExpression::Char(c) => self.visit_char_literal(*c),
            RawExpression::Binary(l, op, r) => self.visit_binary_expression((l, op, r)),
            RawExpression::StaticName(s) => self.visit_static_name(&(*s).clone().with_span(node.span)),
            RawExpression::Imag(i) => self.visit_imaginary_literal(*i),
            RawExpression::PrefixOrPostfix(prefix, op, r) => self.visit_prefix_or_postfix((*prefix, op, r)),
            RawExpression::Call(g, l, r) => self.visit_call((g, l, r)),
            _ => todo!(),
        }
    });

    visit_fn!(prefix_or_postfix for (bool, &Token, &Expression) {
        self.visit_expression(node.2);
    });

    visit_fn!(call for (&Vec<Type>, &Expression, &Vec<Expression>) {
        for ty in node.0 {
            self.visit_type(ty);
        }

        self.visit_expression(node.1);

        self.visit_call_arguments(node.2);
    });

    visit_fn!(call_arguments for &Vec<Expression> {
        for argument in node {
            self.visit_expression(argument);
        }
    });

    visit_fn!(bool_literal for bool);
    visit_fn!(integer_literal for u64);
    visit_fn!(float_literal for f64);
    visit_fn!(imaginary_literal for f64);
    visit_fn!(string_literal for DefaultSymbol);
    visit_fn!(char_literal for char);

    visit_fn!(binary_expression for (&Expression, &Token, &Expression));

    visit_fn!(block for &Vec<Statement> {
        for statement in node {
            self.visit_statement(statement);
        }
    });

    visit_fn!(statement for &Statement {
        match node {
            Statement::Expression(e) => self.visit_expression_statement((true, e)),
            Statement::ExpressionWithoutSemicolon(e) => self.visit_expression_statement((false, e)),
            Statement::Return(r) => self.visit_return_statement(r),
            Statement::Defer(d) => self.visit_defer_statement(d),
            Statement::Var(mutable, name, ty, value) => self.visit_var_statement((mutable, name, ty, value)),
        }
    });

    visit_fn!(expression_statement for (bool, &Expression) {
        self.visit_expression(node.1);
    });

    visit_fn!(return_statement for &Expression {
        self.visit_expression(node);
    });

    visit_fn!(defer_statement for &Expression {
        self.visit_expression(node);
    });

    visit_fn!(var_statement for (&Option<Span>, &WithSpan<DefaultSymbol>, &Option<Type>, &Expression) {
        if let Some(ty) = node.2 {
            self.visit_type(ty);
        }

        self.visit_expression(node.3);
    });

    visit_fn!(op for &Token);
    visit_fn!(static_name for &WithSpan<Vec<DefaultSymbol>>);
}
