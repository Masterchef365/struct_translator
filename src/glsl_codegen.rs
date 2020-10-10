use crate::abstract_data::*;
use crate::Result;
use glsl::syntax::{
    ArrayedIdentifier, Identifier, NonEmpty, StructFieldSpecifier, StructSpecifier, TypeName,
    TypeSpecifier, TranslationUnit, ShaderStage,
};
use glsl::parser::Parse;

const PRELUDE: &str = "
layout(std430, binding = 0) buffer Collection {
    TestBuffer buf[];
};
";

pub fn abstract_to_field(field: &AbstractField) -> Result<StructFieldSpecifier> {
    let ty = TypeSpecifier {
        ty: field.ty.into(),
        array_specifier: None,
    };
    let identifier = ArrayedIdentifier {
        ident: Identifier::new(field.name.clone()).expect("Non-ascii ident"),
        array_spec: None,
    };
    Ok(StructFieldSpecifier {
        qualifier: None,
        identifiers: NonEmpty(vec![identifier]),
        ty,
    })
}

pub fn abstract_to_struct(fields: &[AbstractField], name: &str) -> Result<StructSpecifier> {
    let fields = fields
        .iter()
        .map(abstract_to_field)
        .collect::<Result<Vec<_>>>()?;

    if fields.is_empty() {
        panic!("No fields");
    }

    let fields = NonEmpty(fields);

    let name = Some(TypeName::new(name).expect("Non-ascii name"));
    Ok(StructSpecifier { name, fields })
}

pub fn make_test(fields: &[AbstractField]) -> Result<TranslationUnit> {
    let prelude = ShaderStage::parse(PRELUDE).unwrap();
    Ok(prelude)
}
