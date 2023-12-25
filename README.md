# Problem:

I have a `Parent` and a `Child` that has references to each other like so:

```rust
struct Parent {
    id: u64,
    child: Rc<RefCell<Child>>,
}
struct Child {
    id: u64,
    parent: Weak<RefCell<Parent>>,
}
```
I am trying to set up Deserialization for the `Parent` with serde. The JSON representation should be like so:
```json
{
    "id": 1,
    "child": {
        "id": 100,
        "parent_id": 1,
    }
}
```

However I can't seem to figure out how to actually do this.
The problem I face in the end is that I no longer have ownership of the `Parent` because I need to pass it to the `Rc<RefCell>`.

## Details

Since I need to deserialize in such a way that the `parent_id` field becomes a `parent` refering to the actual `Parent` I start by having an intermediate struct called `ChildDTO` that can easily deserialize.
```rust
#[derive(Deserialize)]
pub struct ChildDTO {
    pub id: u64,
    pub parent_id: u64,
}
```
Next I can implement Deserialization for the `Parent`.
However at the last step of the deserialization when I have to set up the references I am forced to give up ownership of the parent and so I can't return a `Parent`.
```rust
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

Ok(parent) // I want to return a Parent, but now I am stuck returning an Rc<RefCell<Parent>>
```
