mod glsl_codegen;
mod shader_executor;
mod testcase;
use testcase::TestCase;
use struct_translator::*;
use glsl::parser::Parse;
use glsl::syntax::ShaderStage;
use anyhow::{Result, Context};
use shaderc::{Compiler, ShaderKind};
use shader_executor::ShaderExecutor;

fn main() -> Result<()> {
    let fields = [
        AbstractField {
            name: "wonk".into(),
            ty: AbstractType::Vec2,
        },
        AbstractField {
            name: "clonk".into(),
            ty: AbstractType::Float,
        },
    ];

    summarize_layout(&naive_layout_glsl_only(&fields));

    const INVOCATIONS: u32 = 1;
    let mut test = TestCase::new(&fields, INVOCATIONS, 0)?;
    println!("{}", test.glsl_code);
    //println!("{:?}", test.initial);
    //println!("{:?}", test.expected);

    let mut compiler = Compiler::new().context("Couldn't find a compiler")?;

    let spirv = compiler
        .compile_into_spirv(
            &test.glsl_code,
            ShaderKind::Compute,
            "test_shader.comp",
            "main",
            None,
        )
        .context("Failed to compile shader!")?;
    let mut runner = ShaderExecutor::new().context("Failed to init runner")?;

    runner.run_shader(spirv.as_binary_u8(), &mut test.initial, INVOCATIONS)?;

    if test.initial == test.expected {
        println!("Test OK");
    } else {
        for (idx, (test, expect)) in test.initial.iter().zip(test.expected.iter()).enumerate() {
            println!("{:02}: {:02X} {:02X}", idx, test, expect);
        }
    }

    Ok(())
}

/*
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
*/
