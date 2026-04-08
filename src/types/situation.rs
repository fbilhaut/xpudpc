/// Operation type for [`XPlaneClient::situation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SituationOp {
    SaveSituation,
    LoadSituation,
    SaveMovie,
    LoadMovie,
}

impl From<SituationOp> for i32 {
    fn from(op: SituationOp) -> i32 {
        match op {
            SituationOp::SaveSituation => 0,
            SituationOp::LoadSituation => 1,
            SituationOp::SaveMovie => 2,
            SituationOp::LoadMovie => 3,
        }
    }
}
