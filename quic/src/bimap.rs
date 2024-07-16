use std::hash::Hash;

use bevy::utils::HashMap;

pub(crate) struct BiHashMap<Left, Right> {
    left_to_right: HashMap<Left, Right>,
    right_to_left: HashMap<Right, Left>,
}

impl<Left, Right> Default for BiHashMap<Left, Right> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<Left, Right> BiHashMap<Left, Right> {
    pub fn new() -> Self {
        Self {
            left_to_right: HashMap::new(),
            right_to_left: HashMap::new(),
        }
    }
}

impl<Left, Right> BiHashMap<Left, Right>
where
    Left: Copy + Eq + Hash,
    Right: Copy + Eq + Hash,
{
    pub fn insert(&mut self, left: Left, right: Right) {
        self.left_to_right.insert(left, right);
        self.right_to_left.insert(right, left);
    }

    pub fn get_left(&self, left: &Left) -> Option<&Right> {
        self.left_to_right.get(left)
    }

    pub fn get_right(&self, right: &Right) -> Option<&Left> {
        self.right_to_left.get(right)
    }

    pub fn remove_left(&mut self, left: &Left) {
        if let Some(right) = self.left_to_right.remove(left) {
            self.right_to_left.remove(&right);
        }
    }

    pub fn remove_right(&mut self, right: &Right) {
        if let Some(left) = self.right_to_left.remove(right) {
            self.left_to_right.remove(&left);
        }
    }
}