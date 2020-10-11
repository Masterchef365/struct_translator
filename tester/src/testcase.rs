use crate::glsl_codegen::*;
use anyhow::Result;
use glsl::parser::Parse;
use glsl::syntax::{ShaderStage, TranslationUnit};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use struct_translator::*;

pub struct TestCase {
    pub glsl_code: String,
    pub initial: Vec<u8>,
    pub expected: Vec<u8>,
    pub invocations: u32,
}

fn add_contiguous_floats(
    initial: &mut Vec<u8>,
    expected: &mut Vec<u8>,
    count: usize,
    gid: u32,
    rng: &mut impl Rng,
) {
    let vi = rng.gen_range(-100.0, 100.0);
    let ve = vi * gid as f32;
    for _ in 0..count {
        initial.extend_from_slice(&vi.to_le_bytes()[..]);
        expected.extend_from_slice(&ve.to_le_bytes()[..]);
    }
}

fn add_test_value(
    ty: &AbstractType,
    initial: &mut Vec<u8>,
    expected: &mut Vec<u8>,
    gid: u32,
    rng: &mut impl Rng,
) {
    match ty {
        AbstractType::Float => add_contiguous_floats(initial, expected, 1, gid, rng),
        AbstractType::Vec2 => add_contiguous_floats(initial, expected, 2, gid, rng),
        AbstractType::Vec3 => add_contiguous_floats(initial, expected, 3, gid, rng),
        AbstractType::Vec4 => add_contiguous_floats(initial, expected, 4, gid, rng),
    }
}

impl TestCase {
    pub fn new(fields: &[AbstractField], invocations: u32, seed: u64) -> Result<Self> {
        let mut initial = Vec::new();
        let mut expected = Vec::new();
        let naive_layout = naive_layout_glsl_only(fields);

        let mut rng = SmallRng::seed_from_u64(seed);

        for gid in 0..invocations * LOCAL_SIZE {
            for fg in &naive_layout {
                match fg {
                    FieldGap::Gap(g) => initial.extend((0..*g).map(|_| 0)),
                    FieldGap::Field(f) => {
                        add_test_value(&f.ty, &mut initial, &mut expected, gid, &mut rng)
                    }
                }
            }
        }

        let glsl_code = make_test(fields)?;

        Ok(Self {
            glsl_code,
            initial,
            expected,
            invocations,
        })
    }
}
