use crate::error::Result;
pub trait WithIndex {
    fn get_index(self) -> (usize, usize);
}
pub trait Deserialize {
    type Value: WithIndex;
    fn deserialize(bytes: &[u8]) -> Result<Self::Value>;
}
