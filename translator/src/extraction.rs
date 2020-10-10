use glsl::syntax::StructFieldSpecifier;
use glsl::visitor::{Host, Visit, Visitor};
use crate::abstract_data::AbstractField;
use crate::Result;

pub fn get_abstract_fields<H: Host>(structure: &mut H) -> Result<Vec<AbstractField>> {
    let mut abstract_fields = Vec::new();
    let mut extractor = FieldExtractor::new();

    structure.visit(&mut extractor);
    let glsl_fields = extractor.finish();

    for field in &glsl_fields {
        for sub in AbstractField::extract_fields(field)? {
            abstract_fields.push(sub?);
        }
    }

    Ok(abstract_fields)
}

struct FieldExtractor(Vec<StructFieldSpecifier>);

impl FieldExtractor {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn finish(self) -> Vec<StructFieldSpecifier> {
        self.0
    }
}

impl Visitor for FieldExtractor {
    fn visit_struct_field_specifier(&mut self, field: &mut StructFieldSpecifier) -> Visit {
        self.0.push(field.clone());
        Visit::Parent
    }
}
