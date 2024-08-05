use oxc_ast::ast::{Declaration, Expression, JSXChild, JSXElement, JSXElementName};
use oxc_ast::AstBuilder;
use oxc_index::{define_index_type, IndexVec};
use oxc_span::Atom;
use oxc_allocator::Vec;

define_index_type! {
    pub struct TemplateId = usize;
}

pub enum SlotKind<'a> {
    Component(&'a JSXElement<'a>),
    Expression(&'a Expression<'a>),
    Spread(&'a Expression<'a>),
}

pub struct Slot<'a> {
    pub index: usize,
    pub kind: SlotKind<'a>,
}



pub struct Element<'a> {
    ast_builder: AstBuilder<'a>,
    expressions: Vec<'a, Expression<'a>>,
    declarations: Vec<'a, Declaration<'a>>,

    templates_stack: Vec<'a, Vec<'a, Atom<'a>>>,
    slots_stack: Vec<'a, Vec<'a, Slot<'a>>>,

    pub templates: IndexVec<TemplateId, Vec<'a, Atom<'a>>>,
    pub slots: IndexVec<TemplateId, Vec<'a, Slot<'a>>>,
}

impl<'a> Element<'a> {
    pub fn from_jsx_element(builder: AstBuilder<'a>, element: &'a JSXElement) -> Element<'a> {
        let mut result = Element {
            ast_builder: builder,
            templates_stack: builder.vec(),
            slots_stack: builder.vec(),
            declarations: builder.vec(),
            expressions: builder.vec(),

            templates: IndexVec::new(),
            slots: IndexVec::new(),
        };

        result.process_jsx(element);

        if let Some(head) = result.templates_stack.pop() {
            result.templates.push(head);
        }

        if let Some(head) = result.slots_stack.pop() {
            result.slots.push(head);
        }

        result
    }

    fn is_component_name(name: &Atom<'a>) -> bool {
        !name.eq(&name.to_lowercase())
    }

    fn process_children(&mut self, element: &'a JSXElement) {
        for child in &element.children {
            match child {
                JSXChild::Element(element) => {
                    self.process_jsx(element.as_ref())
                }
                JSXChild::Text(text) => {
                    if let Some(head) = self.templates_stack.last_mut() {
                        head.push(text.value.clone());
                    } else {
                        self.templates_stack.push(self.ast_builder.vec1(text.value.clone()));
                    }
                }
                JSXChild::Spread(_spread) => {
                    todo!();
                }
                JSXChild::ExpressionContainer(expr) => {
                    if let Some(head) = self.slots_stack.last_mut() {
                        if let Some(templates_head) = self.templates_stack.last() {
                            // TODO: make index as a dom position
                            let index = templates_head.len();
                            head.push(Slot { index, kind: SlotKind::Expression(expr.expression.as_expression().unwrap()) });
                        } else {
                            panic!("Lost template");
                        }
                    } else {
                        panic!("Lost template");
                    }
                }
                JSXChild::Fragment(_fragment) => {
                    // not possible
                }
            }
        }
    }

    fn process_jsx(&mut self, element: &'a JSXElement) {
        let opening_element = &element.opening_element;
        match &opening_element.name {
            JSXElementName::Identifier(name_identifier) => {
                let name = &name_identifier.name;
                if Self::is_component_name(name) {
                    if let Some(head) = self.slots_stack.last_mut() {
                        if let Some(templates_head) = self.templates_stack.last() {
                            let index = templates_head.len();
                            head.push(Slot { index, kind: SlotKind::Component(element) });
                        } else {
                            panic!("Lost template");
                        }
                    } else {
                        // top-level <Component>
                        todo!();
                    }

                    self.templates_stack.push(self.ast_builder.vec());
                    self.slots_stack.push(self.ast_builder.vec());

                    self.process_children(element);

                    if let Some(head) = self.templates_stack.pop() {
                        self.templates.push(head);
                    } else {
                        panic!("Lost template");
                    }

                    if let Some(head) = self.slots_stack.pop() {
                        self.slots.push(head);
                    } else {
                        panic!("Lost template");
                    }
                } else {
                    if let Some(head) = self.templates_stack.last_mut() {
                        head.push(self.ast_builder.atom("<"));
                        head.push(name.clone());
                        head.push(self.ast_builder.atom(">"));
                    } else {
                        let mut head = self.ast_builder.vec1(self.ast_builder.atom("<"));
                        head.push(name.clone());
                        head.push(self.ast_builder.atom(">"));
                        self.templates_stack.push(head);
                    }

                    if self.slots_stack.is_empty() {
                        self.slots_stack.push(self.ast_builder.vec());
                    }

                    self.process_children(element);

                    if let Some(head) = self.templates_stack.last_mut() {
                        head.push(self.ast_builder.atom("</"));
                        head.push(self.ast_builder.atom(name));
                        head.push(self.ast_builder.atom(">"));
                    } else {
                        panic!("Lost a template");
                    }

                }
            }
            _ => {}
        }
    }
}