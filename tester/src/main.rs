mod glsl_codegen;
mod shader_executor;
use glsl_codegen::make_test;
use struct_translator::*;
use glsl::parser::Parse;
use glsl::syntax::ShaderStage;
use anyhow::{Result, Context};

fn main() -> Result<()> {
    let mut runner = shader_executor::ShaderExecutor::new()?;
    let shader_src = std::fs::read("op.comp.spv")?;

    const GROUPS: usize = 4;
    const ENTRIES_PER_GROUP: usize = 16;
    const ENTRIES: usize = ENTRIES_PER_GROUP * GROUPS;
    type Int = i32;
    let mut buf: Vec<u8> = Vec::with_capacity(ENTRIES * std::mem::size_of::<Int>());
    for i in 0..ENTRIES as Int {
        buf.extend(i.to_le_bytes().iter());
    }

    runner.run_shader(&shader_src, &mut buf, GROUPS as _)?;

    for chunk in buf.chunks(4) {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(chunk);
        let i = Int::from_le_bytes(buf);
        println!("{}", i);
    }

    Ok(())
}

fn pmain() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let shader_path = args.next().context("Requires shader")?;
    let text = std::fs::read_to_string(shader_path)?;
    let mut stage = ShaderStage::parse(text)?;

    let fields = get_abstract_fields(&mut stage)?;
    //let naive = naive_layout_glsl_only(&fields);
    //summarize_layout(&naive);

    /*
    let new_glsl = abstract_to_struct(&fields, "Bob")?;
    glsl::transpiler::glsl::show_struct(&mut out, &new_glsl);
    */

    let code = make_test(&fields)?;
    let mut out = String::new();
    glsl::transpiler::glsl::show_translation_unit(&mut out, &code);
    println!("{}", out);

    Ok(())
}

#[derive(Debug)]
enum FieldGap {
    Field(AbstractField),
    Gap(u64),
}

/// Attempts to emulate glsls layout function
/// Will produce a set of fields and gaps which will attempt to match glsls layout 
fn naive_layout_glsl_only(fields: &[AbstractField]) -> Vec<FieldGap> {
    let mut output = Vec::new();
    let mut offset = 0;
    for field in fields {
        if let Some(gap) = compute_gap(offset, field.ty.align_gl()) {
            output.push(FieldGap::Gap(gap));
            offset += gap;
        }
        output.push(FieldGap::Field(field.clone()));
        offset += field.ty.size();
    }
    if let Some(gap) = compute_gap(offset, AbstractType::Vec4.align_gl()) {
        output.push(FieldGap::Gap(gap));
    }

    output
}

fn summarize_layout(fgs: &[FieldGap]) {
    let mut offset = 0;
    for fg in fgs {
        let size = match fg {
            FieldGap::Gap(gap) => *gap,
            FieldGap::Field(f) => f.ty.size(),
        };
        print!("{:2}-{:2}: ", offset, offset + size);
        match fg {
            FieldGap::Gap(_) => {
                println!("Gap: ({})", size);
            }
            FieldGap::Field(f) => {
                println!("{} ({})", f.name, size);
            }
        }
        offset += size;
    }
}

fn compute_gap(base: u64, align: u64) -> Option<u64> {
    assert!(align > 0);
    let remainder = (align + base) % align;
    if remainder > 0 {
        Some(align - remainder)
    } else {
        None
    }
}
