use crate::array::Value;

pub(crate) trait Command {
    fn execute(&self, values: &[Value]) -> Vec<u8>;
}
