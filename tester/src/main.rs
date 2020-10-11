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
            println!("{:02}: {:02X} {:02X}", idx, expect, test);
        }
    }

    Ok(())
}
