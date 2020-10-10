use crate::Result;
use glsl::syntax::{StructFieldSpecifier, TypeSpecifierNonArray};
use std::convert::{TryFrom, TryInto};

#[derive(Copy, Clone, Debug)]
pub enum AbstractType {
    Float,
    Vec2,
    Vec3,
    Vec4,
}

#[derive(Clone, Debug)]
pub struct AbstractField {
    pub name: String,
    pub ty: AbstractType,
}

impl AbstractField {
    pub fn extract_fields<'a>(
        field: &'a StructFieldSpecifier,
    ) -> Result<impl Iterator<Item = Result<Self>> + 'a> {
        if field.qualifier.is_some() {
            return Err(crate::Error::QualifiersUnsupported);
        }

        if field.ty.array_specifier.is_some() {
            return Err(crate::Error::ArraysUnsupported);
        }

        let ty: AbstractType = field.ty.ty.clone().try_into()?;

        Ok(field.identifiers.0.iter().map(move |ident| {
            if ident.array_spec.is_some() {
                return Err(crate::Error::ArraysUnsupported);
            }
            let name = ident.ident.0.clone();
            Ok(Self { name, ty })
        }))
    }
}

const FLOAT_ALIGN: u64 = 4;
const FLOAT_SIZE: u64 = 4;

impl AbstractType {
    pub fn align_c(&self) -> u64 {
        todo!()
    }

    pub fn align_gl(&self) -> u64 {
        match self {
            AbstractType::Float => FLOAT_ALIGN,
            AbstractType::Vec2 => FLOAT_ALIGN * 2,
            AbstractType::Vec3 => FLOAT_ALIGN * 4,
            AbstractType::Vec4 => FLOAT_ALIGN * 4,
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            AbstractType::Float => FLOAT_SIZE,
            AbstractType::Vec2 => FLOAT_SIZE * 2,
            AbstractType::Vec3 => FLOAT_SIZE * 3,
            AbstractType::Vec4 => FLOAT_SIZE * 4,
        }
    }
}

impl TryFrom<TypeSpecifierNonArray> for AbstractType {
    type Error = crate::Error;
    fn try_from(ty: TypeSpecifierNonArray) -> Result<Self> {
        match ty {
            TypeSpecifierNonArray::Float => Ok(Self::Float),
            TypeSpecifierNonArray::Vec2 => Ok(Self::Vec2),
            TypeSpecifierNonArray::Vec3 => Ok(Self::Vec3),
            TypeSpecifierNonArray::Vec4 => Ok(Self::Vec4),
            _ => Err(crate::Error::UnsupportedType { ty }),
        }
    }
}

impl Into<TypeSpecifierNonArray> for AbstractType {
    fn into(self) -> TypeSpecifierNonArray {
        match self {
            Self::Float => TypeSpecifierNonArray::Float,
            Self::Vec2 => TypeSpecifierNonArray::Vec2,
            Self::Vec3 => TypeSpecifierNonArray::Vec3,
            Self::Vec4 => TypeSpecifierNonArray::Vec4,
        }
    }
}
