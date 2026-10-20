#[derive(Clone)]
pub(crate) struct Actor {
    pub actor: String,
}

impl Default for Actor {
    fn default() -> Self {
        Self {
            actor: "private-operator".into(),
        }
    }
}
