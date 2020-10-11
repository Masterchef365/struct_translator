use crate::abstract_data::*;

#[derive(Debug)]
pub enum FieldGap {
    Field(AbstractField),
    Gap(u64),
}

impl FieldGap {
    pub fn size(&self) -> u64 {
        match self {
            FieldGap::Gap(g) => *g,
            FieldGap::Field(f) => f.ty.size(),
        }
    }
}

/// Attempts to emulate glsls layout function
/// Will produce a set of fields and gaps which will attempt to match glsls layout 
pub fn naive_layout_glsl_only(fields: &[AbstractField]) -> Vec<FieldGap> {
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

pub fn summarize_layout(fgs: &[FieldGap]) {
    let mut offset = 0;
    for fg in fgs {
        let size = match fg {
            FieldGap::Gap(gap) => *gap,
            FieldGap::Field(f) => f.ty.size(),
        };
        print!("{:2}-{:2}: ", offset, offset + size);
        match fg {
            FieldGap::Gap(_) => {
                println!("<gap> ({})", size);
            }
            FieldGap::Field(f) => {
                println!("{} ({})", f.name, size);
            }
        }
        offset += size;
    }
}

pub fn compute_gap(base: u64, align: u64) -> Option<u64> {
    assert!(align > 0);
    let remainder = (align + base) % align;
    if remainder > 0 {
        Some(align - remainder)
    } else {
        None
    }
}
