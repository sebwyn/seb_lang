enum Kind {
    String,
    Int,
}

struct Value {
    kind: Kind,
    bytes: String,
}

struct Object {
    name: String,
    keys: HashMap<String, Value>,
}