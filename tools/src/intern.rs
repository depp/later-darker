use std::collections::HashSet;
use std::sync::Arc;

/// A string interning table.
pub struct Table(HashSet<Arc<str>>);

impl Table {
    pub fn new() -> Self {
        Table(HashSet::new())
    }

    pub fn add(&mut self, str: &str) -> Arc<str> {
        match self.0.get(str) {
            None => {
                let k: Arc<str> = Arc::from(str);
                self.0.insert(k.clone());
                k
            }
            Some(k) => k.clone(),
        }
    }
}
