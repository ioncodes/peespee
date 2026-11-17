pub mod pbp;
pub mod prx;

pub use pbp::Pbp;
pub use prx::Prx;

enum Type {
    Pbp,
    Prx,
}

fn detect(buffer: &[u8]) -> Type {
    if &buffer[0..4] == b"\0PBP" {
        Type::Pbp
    } else {
        Type::Prx
    }
}

pub fn executable(buffer: &[u8]) -> Prx {
    match detect(buffer) {
        Type::Pbp => Pbp::new(buffer).prx(),
        Type::Prx => Prx::parse(buffer).expect("failed to parse executable"),
    }
}
