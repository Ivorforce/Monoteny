use crate::program::computation_tree::ExpressionID;
use crate::program::functions::ParameterKey;


#[derive(Clone)]
pub struct Struct {
    pub keys: Vec<ParameterKey>,
    pub values: Vec<ExpressionID>
}
