use crate::FileHolder;
use std::{
    borrow::Borrow,
    collections::{hash_map::Drain, HashMap},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum DirtyState {
    New,
    Edit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyHandler<R: std::hash::Hash + Eq + Clone, A> {
    resources_to_reserialize: HashMap<R, DirtyState>,
    resources_to_remove: HashMap<R, DirtyState>,
    associated_values: Option<HashMap<R, Vec<A>>>,
}

impl<R: std::hash::Hash + Eq + Clone, A> DirtyHandler<R, A> {
    pub fn new_assoc() -> Self {
        Self {
            resources_to_reserialize: Default::default(),
            resources_to_remove: Default::default(),
            associated_values: Some(Default::default()),
        }
    }

    pub fn new() -> Self {
        Self {
            resources_to_reserialize: Default::default(),
            resources_to_remove: Default::default(),
            associated_values: None,
        }
    }

    pub fn add(&mut self, new_value: R) {
        match self.resources_to_remove.remove(&new_value) {
            Some(DirtyState::New) => {
                // do nothing
            }
            Some(DirtyState::Edit) => {
                self.resources_to_reserialize
                    .insert(new_value, DirtyState::Edit);
            }
            None => {
                self.resources_to_reserialize
                    .insert(new_value, DirtyState::New);
            }
        }
    }

    pub fn replace_associated<'a, F>(&'a mut self, new_value: R, f: F)
    where
        F: Fn(DirtyValueHolder<'a, A>),
    {
        let dirty_state = match self.resources_to_reserialize.remove(&new_value) {
            Some(DirtyState::New) => DirtyState::New,
            Some(DirtyState::Edit) => DirtyState::Edit,
            None => {
                let inner = self
                    .associated_values
                    .as_mut()
                    .unwrap()
                    .entry(new_value.clone())
                    .or_default();
                f(DirtyValueHolder(inner));
                DirtyState::Edit
            }
        };

        self.resources_to_reserialize.insert(new_value, dirty_state);
    }

    pub fn replace(&mut self, new_value: R) {
        let dirty_state = match self.resources_to_reserialize.remove(&new_value) {
            Some(DirtyState::New) => DirtyState::New,
            Some(DirtyState::Edit) | None => DirtyState::Edit,
        };

        self.resources_to_reserialize.insert(new_value, dirty_state);
    }

    pub fn remove<Q: ?Sized>(&mut self, value: &Q)
    where
        R: Borrow<Q>,
        Q: std::hash::Hash + Eq + ToOwned<Owned = R>,
    {
        match self.resources_to_reserialize.remove(value) {
            Some(DirtyState::New) => {
                // do nothing... let it die
            }
            Some(DirtyState::Edit) | None => {
                self.resources_to_remove
                    .insert(value.to_owned(), DirtyState::Edit);

                if let Some(inner) = &mut self.associated_values {
                    inner.remove(value);
                }
            }
        }
    }

    pub fn drain_all(&mut self) -> DirtyDrain<'_, R, A> {
        DirtyDrain {
            resources_to_reserialize: self.resources_to_reserialize.drain(),
            resources_to_remove: self.resources_to_remove.drain(),
            associated_values: self.associated_values.as_mut().map(|v| v.drain()),
        }
    }

    #[allow(dead_code)]
    pub fn resources_to_reserialize(&self) -> &HashMap<R, DirtyState> {
        &self.resources_to_reserialize
    }

    #[allow(dead_code)]
    pub fn resources_to_remove(&self) -> &HashMap<R, DirtyState> {
        &self.resources_to_remove
    }
}

pub struct DirtyValueHolder<'a, A>(&'a mut Vec<A>);
impl<'a> FileHolder for DirtyValueHolder<'a, std::path::PathBuf> {
    fn push(&mut self, f: std::path::PathBuf) {
        self.0.push(f)
    }
}

pub struct DirtyDrain<'a, R: std::hash::Hash + Eq + Clone, A> {
    pub resources_to_reserialize: Drain<'a, R, DirtyState>,
    pub resources_to_remove: Drain<'a, R, DirtyState>,
    pub associated_values: Option<Drain<'a, R, Vec<A>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    fn dirty_handler() -> DirtyHandler<String, usize> {
        DirtyHandler::<String, usize>::new_assoc()
    }

    fn a() -> String {
        "a".to_string()
    }

    #[test]
    fn add() {
        let mut dirty_handler = dirty_handler();

        dirty_handler.add("a".to_string());
        dirty_handler.add("b".to_string());

        assert_eq!(
            dirty_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::New,
                "b".to_string() => DirtyState::New
            }
        );
        assert_eq!(dirty_handler.resources_to_remove, HashMap::default());
    }

    #[test]
    fn replace() {
        let mut dirty_handler = dirty_handler();

        dirty_handler.add("a".to_string());
        dirty_handler.resources_to_reserialize.clear();
        dirty_handler.replace_associated("a".to_string(), |v| v.0.push(0));

        assert_eq!(
            dirty_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::Edit,
            }
        );
        assert_eq!(
            dirty_handler.associated_values,
            Some(hashmap! {
                "a".to_string() => vec![0]
            })
        );
    }

    #[test]
    fn remove() {
        let mut dirty_handler = dirty_handler();

        dirty_handler.add(a());
        dirty_handler.resources_to_reserialize.clear();

        assert!(dirty_handler.resources_to_reserialize.is_empty());
        assert!(dirty_handler.resources_to_remove.is_empty());
        dirty_handler.remove("a");

        assert_eq!(dirty_handler.resources_to_reserialize, hashmap! {});
        assert_eq!(
            dirty_handler.resources_to_remove,
            hashmap! {
                a() => DirtyState::Edit,
            }
        );
    }

    #[test]
    fn add_remove_simple_symmetry() {
        let mut dirty_handler = dirty_handler();

        dirty_handler.add(a());
        dirty_handler.remove("a");

        assert!(dirty_handler.resources_to_reserialize.is_empty());
        assert!(dirty_handler.resources_to_remove.is_empty());
    }

    #[test]
    fn remove_add_simple_symmetry() {
        let mut dirty_handler = dirty_handler();

        dirty_handler.add(a());
        dirty_handler.resources_to_reserialize.clear();

        // we removed it!
        dirty_handler.remove("a");

        // reset the thing...
        dirty_handler.add(a());

        assert_eq!(
            dirty_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::Edit,
            }
        );
        assert!(dirty_handler.resources_to_remove.is_empty());
    }

    #[test]
    fn remove_add_complex_symmetry() {
        let mut dirty_handler = dirty_handler();

        dirty_handler.add(a());
        dirty_handler.resources_to_reserialize.clear();

        // we removed it!
        dirty_handler.remove("a");
        dirty_handler.add(a());
        assert_eq!(
            dirty_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::Edit,
            }
        );
        assert!(dirty_handler.resources_to_remove.is_empty());

        // and now a complex case...
        dirty_handler.resources_to_reserialize.clear();
        dirty_handler.remove("a");
        dirty_handler.add(a());
        assert_eq!(
            dirty_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::Edit,
            }
        );
        assert!(dirty_handler.resources_to_remove.is_empty());
    }

    #[test]
    fn remove_add_remove_symmetry() {
        let mut dirty_handler = dirty_handler();

        dirty_handler.add(a());
        dirty_handler.resources_to_reserialize.clear();

        // we removed it!
        dirty_handler.remove("a");
        dirty_handler.add(a());

        dirty_handler.remove("a");

        assert!(dirty_handler.resources_to_reserialize.is_empty(),);
        assert_eq!(
            dirty_handler.resources_to_remove,
            hashmap! {
                "a".to_string() => DirtyState::Edit,
            }
        );

        assert!(dirty_handler.associated_values.as_ref().unwrap().is_empty());
    }

    #[test]
    fn add_remove_add_symmetry() {
        let mut dummy_handler = dirty_handler();

        // we removed it!
        dummy_handler.add(a());
        dummy_handler.remove("a");
        dummy_handler.add(a());

        assert_eq!(
            dummy_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::New
            }
        );
        assert_eq!(dummy_handler.resources_to_remove, hashmap! {});

        assert!(dummy_handler.associated_values.as_ref().unwrap().is_empty());
    }

    #[test]
    fn replace_remove() {
        let mut dirty_handler = dirty_handler();

        // add resource...
        dirty_handler.add(a());
        dirty_handler.resources_to_reserialize.clear();

        // replace it..
        dirty_handler.replace_associated(a(), |v| v.0.push(0));

        // and then remove it
        dirty_handler.remove("a");

        assert_eq!(dirty_handler.resources_to_reserialize, hashmap! {});
        assert_eq!(
            dirty_handler.resources_to_remove,
            hashmap! {
                "a".to_string() => DirtyState::Edit,
            }
        );

        // aaaaand we keep the files...
        assert_eq!(
            dirty_handler.associated_values,
            Some(hashmap! {
                "a".to_string() => vec![0]
            })
        );
    }

    #[test]
    fn replace_remove_add() {
        let mut dirty_handler = dirty_handler();

        // add resource...
        dirty_handler.add(a());
        dirty_handler.resources_to_reserialize.clear();

        // replace it..
        dirty_handler.replace_associated(a(), |v| v.0.push(0));

        // and then remove it
        dirty_handler.remove("a");

        assert_eq!(dirty_handler.resources_to_reserialize, hashmap! {});
        assert_eq!(
            dirty_handler.resources_to_remove,
            hashmap! {
                "a".to_string() => DirtyState::Edit,
            }
        );

        // aaaaand we keep the files...
        assert_eq!(
            dirty_handler.associated_values,
            Some(hashmap! {
                "a".to_string() => vec![0]
            })
        );
    }
}
