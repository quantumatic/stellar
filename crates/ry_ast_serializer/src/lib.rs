// use std::sync::Arc;

// use ry_ast::{
//     span::At,
//     visitor::{Visitor, VisitorMut},
//     *,
// };
// use string_interner::{usize, StringInterner};

// pub struct ASTSerializer<'a> {
//     content: String,
//     indent: usize,
//     string_interner: &'a StringInterner,
//     resolve_docstrings: bool,
// }

// impl<'a> ASTSerializer<'a> {
//     pub fn new(string_interner: &'a StringInterner, resolve_docstrings: bool) -> Self {
//         Self {
//             content: "".to_owned(),
//             indent: 0,
//             string_interner,
//             resolve_docstrings,
//         }
//     }

//     fn indent(&mut self) {
//         self.content += &"  ".repeat(self.indent);
//     }

//     fn new_line(&mut self) {
//         self.content += "\n";

//         self.indent();
//     }

//     fn write_ident(&mut self, symbol: usize) {
//         self.content
//             .push_str(self.string_interner.resolve(symbol).unwrap());
//     }

//     pub fn serialize(&mut self, ast: &ProgramUnit) -> &str {
//         self.visit_program_unit(ast);
//         &self.content
//     }

//     fn write_docstring(&mut self, global: bool, docstring: &Docstring) {
//         for comment in docstring {
//             self.content += "//";
//             self.content += if global { "!" } else { "/" };
//             self.content += comment;
//             self.new_line();
//         }
//     }
// }

// pub trait Serializer {}

// // impl<'a> Visitor for ASTSerializer<'a> {
// //     fn visit_program_unit(&mut self, node: &ProgramUnit) {
// //         self.write_docstring(true, &node.docstring);

// //         for item in &node.items {
// //             self.visit_item(item);
// //         }
// //     }
// // }
