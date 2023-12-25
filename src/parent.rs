use crate::child::{Child, ChildDTO};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

#[derive(Debug, Serialize)]
pub struct Parent {
    pub id: u64,
    #[serde(serialize_with = "serialize_rc_child")]
    pub child: Rc<RefCell<Child>>,
}

fn serialize_rc_child<S>(rc_child: &Rc<RefCell<Child>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    rc_child.borrow().serialize(serializer)
}

struct ParentVisitor;

impl<'de> Visitor<'de> for ParentVisitor {
    type Value = Parent;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Parent")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Parent, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let mut id = None;
        let mut child: Option<ChildDTO> = None;

        while let Some(key) = map.next_key()? {
            match key {
                "id" => {
                    if id.is_some() {
                        return Err(de::Error::duplicate_field("id"));
                    }
                    id = Some(map.next_value()?);
                }
                "child" => {
                    if child.is_some() {
                        return Err(de::Error::duplicate_field("child"));
                    }
                    child = Some(map.next_value()?);
                }
                _ => {
                    return Err(de::Error::unknown_field(key, &["id", "child"]));
                }
            }
        }

        let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
        let child = child.ok_or_else(|| de::Error::missing_field("child"))?;

        if child.parent_id != id {
            return Err(de::Error::custom("Parent id and child parent_id mismatch"));
        }

        let child = Rc::new(RefCell::new(Child {
            id: child.id,
            parent: Weak::new(),
        }));
        let parent = Rc::new(RefCell::new(Parent { id, child }));

        // Setting the parent weak reference
        parent.borrow().child.borrow_mut().parent = Rc::downgrade(&parent);

        Ok(parent)
    }
}

impl<'de> Deserialize<'de> for Parent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ParentVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::rc::Weak;

    #[test]
    fn serialize() {
        let child = Rc::new(RefCell::new(Child {
            id: 200,
            parent: Weak::new(),
        }));
        let parent = Rc::new(RefCell::new(Parent { id: 1, child }));

        // Setting the parent weak reference
        parent.borrow().child.borrow_mut().parent = Rc::downgrade(&parent);

        // Serializing the parent
        let parent_borrow = parent.borrow();
        let parent_json = serde_json::to_string(&*parent_borrow).unwrap();
        assert_eq!(parent_json, r#"{"id":1,"child":{"id":200,"parent":1}}"#);
    }
}
