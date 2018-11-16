use geister_core::{board::GhostID, player::PlayerID};

#[derive(Clone, Copy, Debug, Default)]
pub struct MetaData {
    escape: u8,
    noescape: u8,
    noout: bool,
}

impl MetaData {
    
}

#[derive(Clone, Debug)]
pub struct MetaDataList([MetaData; 16]);

impl MetaDataList {
    pub fn new() -> Self {
        MetaDataList([MetaData::default(); 16])
    }
    pub fn get(&self, ghost: GhostID, owner: PlayerID) -> &MetaData {
        let idx = ghost.as_u8() + if owner == PlayerID::P1 { 0 } else { 8 };
        &self.0[usize::from(idx)]
    }
}





