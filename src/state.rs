use serde::{Deserialize, Serialize};

pub const STORAGE_KEY: &str = "scorecounter:v1";
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Counter {
    pub id: String,
    pub name: String,
    pub score: i32,
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoredData {
    pub schema_version: u32,
    pub counters: Vec<Counter>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Store {
    pub counters: Vec<Counter>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            counters: Vec::new(),
        }
    }
}

impl Store {
    pub fn from_storage(data: StoredData) -> Self {
        Self {
            counters: data.counters,
        }
    }

    pub fn to_storage(&self) -> StoredData {
        StoredData {
            schema_version: SCHEMA_VERSION,
            counters: self.counters.clone(),
        }
    }

    pub fn upsert(&mut self, counter: Counter) {
        match self.counters.iter_mut().find(|c| c.id == counter.id) {
            Some(existing) => {
                *existing = counter;
            }
            None => self.counters.push(counter),
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.counters.retain(|c| c.id != id);
    }

    pub fn adjust_score(&mut self, id: &str, delta: i32) -> Option<i32> {
        let counter = self.counters.iter_mut().find(|c| c.id == id)?;
        counter.score += delta;
        Some(counter.score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_counter(id: &str, score: i32) -> Counter {
        Counter {
            id: id.to_string(),
            name: "Player".to_string(),
            score,
            color: "#ffffff".to_string(),
        }
    }

    #[test]
    fn adjust_score_updates_value() {
        let mut store = Store {
            counters: vec![sample_counter("a", 0)],
        };
        let result = store.adjust_score("a", 5);
        assert_eq!(result, Some(5));
        assert_eq!(store.counters[0].score, 5);
    }

    #[test]
    fn remove_clears_counter() {
        let mut store = Store {
            counters: vec![sample_counter("a", 0), sample_counter("b", 1)],
        };
        store.remove("a");
        assert_eq!(store.counters.len(), 1);
        assert_eq!(store.counters[0].id, "b");
    }

    #[test]
    fn storage_round_trip() {
        let counter = sample_counter("x", -3);
        let store = Store {
            counters: vec![counter.clone()],
        };
        let payload = store.to_storage();
        let restored = Store::from_storage(payload);
        assert_eq!(restored.counters, vec![counter]);
    }
}
