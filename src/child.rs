use crate::parent::Parent;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Weak};

#[derive(Deserialize)]
pub struct ChildDTO {
    pub id: u64,
    pub parent_id: u64,
}

#[derive(Debug, Serialize)]
pub struct Child {
    pub id: u64,
    #[serde(serialize_with = "serialize_weak_parent")]
    pub parent: Weak<RefCell<Parent>>,
}

fn serialize_weak_parent<S>(
    weak_parent: &Weak<RefCell<Parent>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(parent) = weak_parent.upgrade() {
        parent.borrow().id.serialize(serializer)
    } else {
        Err(serde::ser::Error::custom("Parent is missing"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::rc::Rc;

    #[test]
    fn serialize() {
        let child = Rc::new(RefCell::new(Child {
            id: 200,
            parent: Weak::new(),
        }));
        let parent = Rc::new(RefCell::new(Parent { id: 1, child }));

        // Setting the parent weak reference
        parent.borrow().child.borrow_mut().parent = Rc::downgrade(&parent);

        // Serializing the child
        let parent_borrow = parent.borrow();
        let child_borrow = parent_borrow.child.borrow();
        let child_json = serde_json::to_string(&*child_borrow).unwrap();
        assert_eq!(child_json, r#"{"id":200,"parent":1}"#);
    }
}
