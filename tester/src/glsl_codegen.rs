use anyhow::Result;
use glsl::parser::Parse;
use glsl::syntax::{
    ArrayedIdentifier, Declaration, Expr, ExternalDeclaration, FullySpecifiedType, Identifier,
    InitDeclaratorList, NonEmpty, ShaderStage, SingleDeclaration, Statement, StructFieldSpecifier,
    StructSpecifier, TranslationUnit, TypeName, TypeSpecifier, TypeSpecifierNonArray,
};
use std::fmt::Write;
use struct_translator::*;

pub const LOCAL_SIZE: u32 = 16;

const PRELUDE: &str = "
#version 450
layout (local_size_x = 16) in;
uint gid = gl_GlobalInvocationID.x;
";

const BINDS: &str = "
layout(std140, binding = 0) buffer Collection {
    TestStruct data[];
};
";

fn glsl_typename(ty: &AbstractType) -> &str {
    match ty {
        AbstractType::Float => "float",
        AbstractType::Vec2 => "vec2",
        AbstractType::Vec3 => "vec3",
        AbstractType::Vec4 => "vec4",
    }
}

pub fn make_test(fields: &[AbstractField]) -> Result<String> {
    let mut output = String::new();
    // Prelude
    output.push_str(PRELUDE);

    // Structure
    output.push_str("struct TestStruct {\n");
    for field in fields {
        write!(
            &mut output,
            "    {} {};\n",
            glsl_typename(&field.ty),
            field.name,
        )?;
    }
    output.push_str("};\n");

    // Bindings
    output.push_str(BINDS);

    // Test pattern
    output.push_str("void main() {\n");
    for field in fields {
        write!(&mut output, "    data[gid].{} *= float(gid);\n", field.name)?;
    }
    output.push_str("}\n");

    Ok(output)
}

/*
fn abstract_to_field(field: &AbstractField) -> Result<StructFieldSpecifier> {
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

fn abstract_to_struct(fields: &[AbstractField], name: &str) -> Result<StructSpecifier> {
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
    // Header, local size, gid
    let mut glsl_code = ShaderStage::parse(PRELUDE).unwrap();

    // Structure
    let struct_ = abstract_to_struct(&fields, "TestStruct")?;
    let decl = decl_struct(struct_);
    glsl_code.push(decl);

    // SSBO
    let mut binds = ShaderStage::parse(BINDS).unwrap();
    (glsl_code.0).0.append(&mut (binds.0).0);

    todo!()
}
*/
