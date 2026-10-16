//! Which keys a save changed, so the log can say what moved without ever
//! printing a setting value.

use serde_json::Value;

/// Keys whose value differs between `previous` and `new` (added, removed or
/// changed), sorted. Non-objects count as having no keys; `previous` is
/// `None` on the first save of a scope.
#[must_use]
pub fn changed_keys(previous: Option<&Value>, new: &Value) -> Vec<String> {
    let empty = serde_json::Map::new();
    let old = previous.and_then(Value::as_object).unwrap_or(&empty);
    let new = new.as_object().unwrap_or(&empty);
    let mut keys: Vec<String> = old
        .keys()
        .chain(new.keys())
        .filter(|key| old.get(*key) != new.get(*key))
        .cloned()
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

#[cfg(test)]
mod tests {
    use super::changed_keys;
    use serde_json::json;

    #[test]
    fn reports_only_keys_whose_value_changed() {
        let old = json!({"theme": "dark", "clock": "h24"});
        let new = json!({"theme": "light", "clock": "h24"});
        assert_eq!(changed_keys(Some(&old), &new), ["theme"]);
    }

    #[test]
    fn reports_added_and_removed_keys_sorted() {
        let old = json!({"b": 1, "z": 2});
        let new = json!({"a": 1, "b": 1});
        assert_eq!(changed_keys(Some(&old), &new), ["a", "z"]);
    }

    #[test]
    fn first_save_changes_every_key() {
        assert_eq!(changed_keys(None, &json!({"x": 1, "a": 2})), ["a", "x"]);
    }

    #[test]
    fn identical_documents_change_nothing() {
        let doc = json!({"a": 1});
        assert!(changed_keys(Some(&doc), &doc).is_empty());
    }

    #[test]
    fn non_objects_have_no_keys() {
        assert!(changed_keys(Some(&json!(3)), &json!("x")).is_empty());
    }
}
