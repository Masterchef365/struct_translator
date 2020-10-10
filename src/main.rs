use glsl::parser::Parse as _;
use glsl::syntax::{ShaderStage, StructFieldSpecifier};
use glsl::visitor::{Host, Visit, Visitor};
use glsl::transpiler::glsl as trans;
use anyhow::Result;

struct MyVisitor;

impl Visitor for MyVisitor {
    fn visit_struct_field_specifier(&mut self, field: &mut StructFieldSpecifier) -> Visit {
        let mut ty = String::new();
        let mut name = String::new();
        trans::show_type_specifier(&mut ty, &field.ty);
        for ident in &field.identifiers {
            name.clear();
            trans::show_arrayed_identifier(&mut name, ident);
            println!("    {}: {},", name, ty);
        }
        Visit::Parent
    }
}

fn main() -> Result<()> {
    let text = std::fs::read_to_string("particle.comp")?;
    let mut stage = ShaderStage::parse(text)?;
    let mut my_visitor = MyVisitor;
    println!("struct {{");
    stage.visit(&mut my_visitor);
    println!("}}");
    Ok(())
}
