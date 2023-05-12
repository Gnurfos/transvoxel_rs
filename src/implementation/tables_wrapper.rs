use super::aux_tables::{RegularCellVoxelIndex, TransitionCellGridPointIndex};

pub struct RegularReuseIndex(pub usize);
pub struct TransitionReuseIndex(pub usize);

pub struct RegularVertexData(pub u16);

impl RegularVertexData {
    pub fn reuse_dx(&self) -> isize {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        -(((reuse_info & 0x10) >> 4) as i16) as isize
    }
    pub fn reuse_dy(&self) -> isize {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        -(((reuse_info & 0x20) >> 5) as i16) as isize
    }
    pub fn reuse_dz(&self) -> isize {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        -(((reuse_info & 0x40) >> 6) as i16) as isize
    }
    pub fn new_vertex(&self) -> bool {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        (reuse_info & 0x80) != 0
    }
    pub fn reuse_index(&self) -> RegularReuseIndex {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        RegularReuseIndex((reuse_info & 0x0F) as usize)
    }
    pub fn voxel_a_index(&self) -> RegularCellVoxelIndex {
        let edge_location = self.0 & 0xFF;
        RegularCellVoxelIndex(((edge_location & 0xF0) >> 4) as usize)
    }
    pub fn voxel_b_index(&self) -> RegularCellVoxelIndex {
        let edge_location = self.0 & 0xFF;
        RegularCellVoxelIndex((edge_location & 0xF) as usize)
    }
}

/// The low byte contains the indices for the two endpoints of the edge on which the vertex lies.
/// The high byte contains the vertex reuse data.

pub struct TransitionVertexData(pub u16);

impl TransitionVertexData {
    pub fn reuse(&self) -> bool {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        (reuse_info & 0x30) != 0
    }
    pub fn reuse_du(&self) -> isize {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        -(((reuse_info & 0x10) >> 4) as i16) as isize
    }
    pub fn reuse_dv(&self) -> isize {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        -(((reuse_info & 0x20) >> 5) as i16) as isize
    }
    pub fn _new_interior(&self) -> bool {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        (reuse_info & 0x40) != 0
    }
    pub fn new_reusable(&self) -> bool {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        (reuse_info & 0x80) != 0
    }
    pub fn reuse_index(&self) -> TransitionReuseIndex {
        let reuse_info = (self.0 & 0xFF00) >> 8;
        TransitionReuseIndex((reuse_info & 0x0F) as usize)
    }
    pub fn grid_point_a_index(&self) -> TransitionCellGridPointIndex {
        let edge_location = self.0 & 0xFF;
        TransitionCellGridPointIndex(((edge_location & 0xF0) >> 4) as usize)
    }
    pub fn grid_point_b_index(&self) -> TransitionCellGridPointIndex {
        let edge_location = self.0 & 0xFF;
        TransitionCellGridPointIndex((edge_location & 0xF) as usize)
    }
}
