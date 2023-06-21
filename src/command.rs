use crate::array::Value;

pub(crate) trait Command {
    fn execute(&self, values: &Vec<Value>) -> Vec<u8>;
}
