use crate::linker::LinkError;
use crate::program::computation_tree::{ExpressionForest, ExpressionID, ExpressionOperation};
use crate::program::generics::TypeForest;
use crate::program::types::TypeUnit;

pub struct AmbiguousExpression {
    pub expression_id: ExpressionID,

    pub candidates: Vec<Box<dyn Fn(&mut TypeForest, ExpressionID) -> Result<ExpressionOperation, LinkError>>>
}

impl AmbiguousExpression {
    pub fn reduce(&mut self, expressions: &mut ExpressionForest) -> bool {
        let callbacks: Vec<Box<dyn Fn(&mut TypeForest, ExpressionID) -> Result<ExpressionOperation, LinkError>>> = self.candidates.drain(..).collect();

        for callback in callbacks {
            let mut type_forest_copy = expressions.type_forest.clone();

            let result = callback(type_forest_copy.as_mut(), self.expression_id);
            if !(result.is_err()) {
                self.candidates.push(callback);
            }
        }

        if self.candidates.len() == 0 {
            todo!("Properly output the error")
        }
        else if self.candidates.len() == 1 {
            let candidate = self.candidates.drain(..).next().unwrap();
            let result = candidate(&mut expressions.type_forest, self.expression_id).unwrap();
            expressions.operations.insert(self.expression_id, result);
            return true;
        }

        // Not done yet
        return false
    }
}
