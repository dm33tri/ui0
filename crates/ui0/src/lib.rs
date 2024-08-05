use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{Expression, JSXElementName, Program},
    AstBuilder,
};
use oxc_codegen::Codegen;
use oxc_index::{Idx, IndexVec};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{SemanticBuilder, SymbolId, SymbolTable};
use oxc_span::{SourceType, SPAN};
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};
use crate::jsx::SlotKind;

mod jsx;

struct Components<'a> {
    i: usize,
    symbols: SymbolTable,
    builder: AstBuilder<'a>,
    program: Program<'a>,
    source_type: SourceType,

    components: IndexVec<SymbolId, String>,
}

impl<'a> Components<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        let builder = AstBuilder::new(&allocator);
        let source_type = SourceType::default()
            .with_jsx(true)
            .with_typescript(true)
            .with_module(true);
        let program = builder.program(SPAN, source_type, None, builder.vec(), builder.vec());
        let symbols = SymbolTable::default();
        Components {
            i: 0,
            symbols,
            builder,
            source_type,
            program,
            components: IndexVec::new()
        }
    }

    pub fn add(&mut self, src: &'a str) {
        let allocator = self.builder.allocator;
        let mut parser = Parser::new(&allocator, src, self.source_type);
        let ParserReturn {
            mut program,
            errors: _errors,
            trivias: _trivias,
            panicked: _panicked,
        } = parser.parse();

        let (symbols, scopes) = SemanticBuilder::new(src, self.source_type)
            .build(&program)
            .semantic
            .into_symbol_table_and_scope_tree();
        let (_symbols, _scopes) = traverse_mut(self, allocator, &mut program, symbols, scopes);

        for statement in program.body {
            self.program.body.push(statement);
        }
    }

    pub fn print(&self) -> String {
        Codegen::<false>::new().build(&self.program).source_text
    }
}




impl<'a> Traverse<'a> for Components<'a> {
    fn enter_expression(&mut self, n: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match n {
            Expression::JSXElement(element) => {
                let element = jsx::Element::from_jsx_element(self.builder, element);
                for i in 0..element.templates.len() {

                }
                for (template, slots) in std::iter::zip(element.templates, element.slots) {
                    let mut s = String::new();
                    let mut j: usize = 0;
                    for i in 0..template.len() {
                        s += template[i].as_str();
                        if slots.len() > j && slots[j].index == i + 1 {
                            match slots[j].kind {
                                SlotKind::Component(component) => {
                                    if let JSXElementName::Identifier(name) = &component.opening_element.name {
                                        let name = name.name.as_str();
                                        s += format!("<{}>", name).as_str();
                                    }
                                }
                                SlotKind::Expression(_) => {
                                    s += "{}";
                                }
                                SlotKind::Spread(_) => {
                                    s += "{...}"
                                }
                            }
                            j += 1;
                        }
                    }
                    println!("Element {}: {}", self.i, s);
                }
                self.i = self.i + 1;

                // let mut templates: Vec<String> = vec!();
                // let mut values = self.builder.vec::<ArrayExpressionElement>();
                // let opening_element = &element.opening_element;
                // if let JSXElementName::Identifier(name_identifier) = &opening_element.name {

                // let a = ctx.find_ancestor(|a| {
                //     if let Ancestor::FunctionBody(_) = a {
                //         Found(a)
                //     } else {
                //         Continue
                //     }
                // });
                // if let Some(Ancestor::FunctionBody(a)) = a {
                //     if let Some(id) = a.id() {
                //         println!("Function {} ({}) is a component, child {}", id.name, id.symbol_id.get().unwrap().index(), name_identifier.name);
                //     }
                // }

                // let r = self.builder.identifier_reference(name_identifier.span, name_identifier.name.clone());
                // let e = self.builder.expression_from_identifier_reference(r);
                // values.push(self.builder.array_expression_element_expression(e));

                // *n = self.builder.expression_array(e.span, values.into(), None);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_component() {
        let allocator = Allocator::default();
        // let source = "function x() { console.log('test'); return (<Test>Hello, world!</Test>) }; const z = () => <Hello />";
        // let source2 = "function y() { return (<span>Hello, world 2!</span>) }";
        let source3 = "function A() { \
            if (true) {\
                return <div><p>Hello, world 3!</p></div>;\
            }\
            return (\
                <span>\
                    <i>Hello, world {number}!</i>\
                </span>\
            );\
        }\
        function a() {\
            return 3;\
        }\
        function B(props) {\
            return <div>Hello, Test!<Test><div>Test</div></Test><span>Hello, Test 2!</span></div>;\
        }\
        const Test = (props) => {\
            return <div onClick={() => <b />}>{() => {\
                return <div>TestX</div>\
            }}</div>\
        }";
        let mut c = Components::new(&allocator);
        // c.add(source);
        // c.add(source2);
        c.add(source3);

        println!("{:?}", c.print());
    }
}
