use crate::abstract_data::*;
use crate::Result;
use glsl::parser::Parse;
use glsl::syntax::{
    ArrayedIdentifier, Declaration, ExternalDeclaration, FullySpecifiedType, Identifier,
    InitDeclaratorList, NonEmpty, ShaderStage, SingleDeclaration, StructFieldSpecifier,
    StructSpecifier, TranslationUnit, TypeName, TypeSpecifier, TypeSpecifierNonArray,
};

const PRELUDE: &str = "
#version 450
layout (local_size_x = 64) in;
uint gid = gl_GlobalInvocationID.x;
";

const BINDS: &str = "
layout(std140, binding = 0) buffer Collection {
    TestStruct buf[];
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

fn decl_struct(struct_: StructSpecifier) -> ExternalDeclaration {
    let ty = TypeSpecifierNonArray::Struct(struct_);
    let ty = TypeSpecifier {
        ty,
        array_specifier: None,
    };
    let ty = FullySpecifiedType {
        ty,
        qualifier: None,
    };
    let head = SingleDeclaration {
        ty,
        name: None,
        array_specifier: None,
        initializer: None,
    };
    let decl = InitDeclaratorList { head, tail: vec![] };
    let decl = Declaration::InitDeclaratorList(decl);
    ExternalDeclaration::Declaration(decl)
}

pub fn make_test(fields: &[AbstractField]) -> Result<TranslationUnit> {
    let mut output = ShaderStage::parse(PRELUDE).unwrap();
    let struct_ = abstract_to_struct(&fields, "TestStruct")?;
    let decl = decl_struct(struct_);
    output.push(decl);
    let mut binds = ShaderStage::parse(BINDS).unwrap();
    (output.0).0.append(&mut (binds.0).0);

    Ok(output)
}
