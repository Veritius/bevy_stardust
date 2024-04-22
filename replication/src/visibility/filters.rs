use std::{any::TypeId, sync::Arc};

/// Storage for how replicated data is considered visible to a peer.
pub struct VisibilityGroupFilters {
    conditions: VisibilityCondition,
}

impl Default for VisibilityGroupFilters {
    fn default() -> Self {
        Self {
            conditions: VisibilityCondition::default()
        }
    }
}

impl VisibilityCondition {
    fn change_in_place(&mut self, funct: impl FnOnce(VisibilityCondition) -> VisibilityCondition) {
        let mut swap = VisibilityCondition::default();
        std::mem::swap(self, &mut swap);
        *self = funct(swap);
    }

    pub fn any(&mut self, condition: VisibilityCondition) {
        self.change_in_place(|this| {
            VisibilityCondition::Any(vec![
                this, condition,
            ])
        });
    }

    pub fn and(&mut self, condition: VisibilityCondition) {
        self.change_in_place(|this| {
            VisibilityCondition::And(vec![
                this, condition
            ])
        })
    }
}

impl Default for VisibilityCondition {
    #[inline]
    fn default() -> Self {
        Self::Any(vec![])
    }
}

pub enum VisibilityCondition {
    Member(TypeId),
    Funct(Arc<dyn Fn(TypeId) -> bool>),
    Any(Vec<VisibilityCondition>),
    And(Vec<VisibilityCondition>),
    Not(Box<VisibilityCondition>),
    Xor {
        a: Box<VisibilityCondition>,
        b: Box<VisibilityCondition>,
    }
}