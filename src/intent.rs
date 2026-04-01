pub struct IntentModel {
    pub layers: Vec<Vec<String>>,
}
pub fn infer_intent(graph: &ModuleGraph) -> IntentModel;
