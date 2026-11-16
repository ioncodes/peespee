pub mod pbp;
pub mod prx;

pub use pbp::Pbp;
pub use prx::Prx;

pub enum Type {
    Pbp,
    Prx,
}

pub fn detect(buffer: &[u8]) -> Type {
    if &buffer[0..4] == b"\0PBP" {
        Type::Pbp
    } else {
        Type::Prx
    }
}