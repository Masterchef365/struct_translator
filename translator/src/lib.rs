mod abstract_data;
mod extraction;
mod glsl_layout;
pub use glsl_layout::*;
pub use extraction::*;
pub use abstract_data::*;
use glsl::syntax::TypeSpecifierNonArray;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unsupported data type {:?}", ty)]
    UnsupportedType {
        ty: TypeSpecifierNonArray,
    },
    #[error("Currently, we do not support qualifiers")]
    QualifiersUnsupported,
    #[error("Currently, we do not support arrays")]
    ArraysUnsupported,
}

pub type Result<T> = std::result::Result<T, Error>;
