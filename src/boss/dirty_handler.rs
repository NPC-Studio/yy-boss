use std::{
    borrow::Borrow,
    collections::{hash_map::Drain, HashMap},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum DirtyState {
    New,
    Edit,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DirtyHandler<R: std::hash::Hash + Eq + Clone, A> {
    resources_to_reserialize: HashMap<R, DirtyState>,
    resources_to_remove: HashMap<R, DirtyState>,
    associated_values: Option<HashMap<R, Vec<A>>>,
}

impl<R: std::hash::Hash + Eq + Clone, A> DirtyHandler<R, A> {
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
        F: Fn(DirtyAssociatedValue<'a, A>),
    {
        let dirty_state = match self.resources_to_reserialize.remove(&new_value) {
            Some(DirtyState::New) => DirtyState::New,
            Some(DirtyState::Edit) | None => {
                let inner = self
                    .associated_values
                    .as_mut()
                    .unwrap()
                    .entry(new_value.clone())
                    .or_default();
                f(DirtyAssociatedValue(inner));
                DirtyState::Edit
            }
        };

        self.resources_to_reserialize.insert(new_value, dirty_state);
    }
    
    pub fn replace<'a>(&'a mut self, new_value: R) {
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
}

pub struct DirtyAssociatedValue<'a, A>(&'a mut Vec<A>);

impl<'a, A> DirtyAssociatedValue<'a, A> {
    pub fn push(&mut self, a: A) {
        self.0.push(a);
    }
}

pub struct DirtyDrain<'a, R: std::hash::Hash + Eq + Clone, A> {
    pub resources_to_reserialize: Drain<'a, R, DirtyState>,
    pub resources_to_remove: Drain<'a, R, DirtyState>,
    pub associated_values: Option<Drain<'a, R, Vec<A>>>,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::dummy::DummyResource;
//     use maplit::hashmap;

//     #[test]
//     fn add() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();

//         assert!(dummy_handler.set(DummyResource::new("a", 0), 0).is_none());
//         assert!(dummy_handler.set(DummyResource::new("b", 0), 0).is_none());

//         assert_eq!(
//             dummy_handler.resources_to_reserialize,
//             hashmap! {
//                 "a".to_string() => DirtyState::New,
//                 "b".to_string() => DirtyState::New
//             }
//         );
//         assert_eq!(dummy_handler.resources_to_remove, HashMap::default());

//         assert_eq!(
//             dummy_handler.set(DummyResource::new("a", 1), 0),
//             Some(YyResourceData {
//                 yy_resource: DummyResource::new("a", 0),
//                 associated_data: Some(0)
//             })
//         );
//     }

//     #[test]
//     fn replace() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();

//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         dummy_handler.set(DummyResource::new("b", 0), 0);
//         dummy_handler.resources_to_reserialize.clear();

//         assert_eq!(
//             dummy_handler.set(DummyResource::new("a", 1), 0),
//             Some(YyResourceData {
//                 yy_resource: DummyResource::new("a", 0),
//                 associated_data: Some(0)
//             })
//         );
//         assert_eq!(
//             dummy_handler.resources_to_reserialize,
//             hashmap! {
//                 "a".to_string() => DirtyState::Edit,
//             }
//         );
//         assert_eq!(
//             dummy_handler.associated_files_to_cleanup,
//             hashmap! {
//                 "a".to_string() => vec![Path::new("a/0.txt").to_owned()]
//             }
//         );
//         assert_eq!(
//             dummy_handler.associated_folders_to_cleanup,
//             hashmap! {
//                 "a".to_string() => vec![Path::new("a/0").to_owned()]
//             }
//         );
//     }

//     #[test]
//     fn remove() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
//         let tcu = TrailingCommaUtility::new();

//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         dummy_handler.resources_to_reserialize.clear();

//         assert!(dummy_handler.resources_to_reserialize.is_empty());
//         assert!(dummy_handler.resources_to_remove.is_empty());

//         assert_eq!(
//             dummy_handler.remove("a", &tcu),
//             Some((DummyResource::new("a", 0), Some(0)))
//         );
//         assert_eq!(dummy_handler.remove("a", &tcu), None);

//         assert_eq!(dummy_handler.resources_to_reserialize, hashmap! {});
//         assert_eq!(
//             dummy_handler.resources_to_remove,
//             hashmap! {
//                 "a".to_string() => DirtyState::Edit,
//             }
//         );
//     }

//     #[test]
//     fn add_remove_simple_symmetry() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
//         let tcu = TrailingCommaUtility::new();

//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         assert!(dummy_handler.remove("a", &tcu).is_some());

//         assert!(dummy_handler.resources_to_reserialize.is_empty());
//         assert!(dummy_handler.resources_to_remove.is_empty());
//     }

//     #[test]
//     fn remove_add_simple_symmetry() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
//         let tcu = TrailingCommaUtility::new();

//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         dummy_handler.resources_to_reserialize.clear();

//         // we removed it!
//         assert_eq!(
//             dummy_handler.remove("a", &tcu),
//             Some((DummyResource::new("a", 0), Some(0)))
//         );

//         // reset the thing...
//         dummy_handler.set(DummyResource::new("a", 0), 0);

//         assert_eq!(
//             dummy_handler.resources_to_reserialize,
//             hashmap! {
//                 "a".to_string() => DirtyState::Edit,
//             }
//         );
//         assert!(dummy_handler.resources_to_remove.is_empty());
//     }

//     #[test]
//     fn remove_add_complex_symmetry() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
//         let tcu = TrailingCommaUtility::new();

//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         dummy_handler.resources_to_reserialize.clear();

//         // we removed it!
//         dummy_handler.remove("a", &tcu);
//         dummy_handler.set(DummyResource::new("a", 1), 0);
//         assert_eq!(
//             dummy_handler.resources_to_reserialize,
//             hashmap! {
//                 "a".to_string() => DirtyState::Edit,
//             }
//         );
//         assert!(dummy_handler.resources_to_remove.is_empty());

//         // and now a complex case...
//         dummy_handler.resources_to_reserialize.clear();
//         dummy_handler.remove("a", &tcu);
//         dummy_handler.set(DummyResource::new("a", 1), 1);
//         assert_eq!(
//             dummy_handler.resources_to_reserialize,
//             hashmap! {
//                 "a".to_string() => DirtyState::Edit,
//             }
//         );
//         assert!(dummy_handler.resources_to_remove.is_empty());
//     }

//     #[test]
//     fn remove_add_remove_symmetry() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
//         let tcu = TrailingCommaUtility::new();

//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         dummy_handler.resources_to_reserialize.clear();

//         // we removed it!
//         dummy_handler.remove("a", &tcu);
//         dummy_handler.set(DummyResource::new("a", 0), 0);

//         dummy_handler.remove("a", &tcu);

//         assert!(dummy_handler.resources_to_reserialize.is_empty(),);
//         assert_eq!(
//             dummy_handler.resources_to_remove,
//             hashmap! {
//                 "a".to_string() => DirtyState::Edit,
//             }
//         );

//         assert!(dummy_handler.associated_files_to_cleanup.is_empty());
//         assert!(dummy_handler.associated_folders_to_cleanup.is_empty());
//     }

//     #[test]
//     fn add_remove_add_symmetry() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
//         let tcu = TrailingCommaUtility::new();

//         // we removed it!
//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         dummy_handler.remove("a", &tcu);
//         dummy_handler.set(DummyResource::new("a", 0), 0);

//         assert_eq!(
//             dummy_handler.resources_to_reserialize,
//             hashmap! {
//                 "a".to_string() => DirtyState::New
//             }
//         );
//         assert_eq!(dummy_handler.resources_to_remove, hashmap! {});

//         assert!(dummy_handler.associated_files_to_cleanup.is_empty());
//         assert!(dummy_handler.associated_folders_to_cleanup.is_empty());
//     }

//     #[test]
//     fn replace_associated_files() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();

//         // we removed it!
//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         dummy_handler.set(DummyResource::new("a", 0), 0);

//         assert_eq!(
//             dummy_handler.resources_to_reserialize,
//             hashmap! {
//                 "a".to_string() => DirtyState::New,
//             }
//         );
//         assert_eq!(dummy_handler.resources_to_remove, hashmap! {});

//         // notice how there's nothing to remove yet here...
//         assert_eq!(dummy_handler.associated_files_to_cleanup, hashmap![]);
//         assert_eq!(dummy_handler.associated_folders_to_cleanup, hashmap![]);
//     }

//     #[test]
//     fn replace_remove() {
//         let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
//         let tcu = TrailingCommaUtility::new();

//         // add resource...
//         dummy_handler.set(DummyResource::new("a", 0), 0);
//         dummy_handler.resources_to_reserialize.clear();

//         // replace it..
//         dummy_handler.set(DummyResource::new("a", 0), 0);

//         // and then remove it
//         dummy_handler.remove("a", &tcu);

//         assert_eq!(dummy_handler.resources_to_reserialize, hashmap! {});
//         assert_eq!(
//             dummy_handler.resources_to_remove,
//             hashmap! {
//                 "a".to_string() => DirtyState::Edit,
//             }
//         );

//         // aaaaand no files to cleanup!
//         assert_eq!(dummy_handler.associated_files_to_cleanup, hashmap![]);
//         assert_eq!(dummy_handler.associated_folders_to_cleanup, hashmap![]);
//     }
// }
