use struct_translator::*;
use glsl::parser::Parse;
use glsl::syntax::ShaderStage;
use anyhow::Result;

fn main() -> Result<()> {
    let text = std::fs::read_to_string("particle2.comp")?;
    let mut stage = ShaderStage::parse(text)?;

    let fields = get_abstract_fields(&mut stage)?;
    let naive = naive_layout_glsl_only(&fields);
    summarize_layout(&naive);

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
