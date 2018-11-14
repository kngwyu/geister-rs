use guister_core::player::Player;

pub trait GpuProto: Player {}

pub fn start_client<C: GpuProto>(client: &mut C) {}
