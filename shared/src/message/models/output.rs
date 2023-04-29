use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Write};

use serde::{Deserialize, Serialize};
use toml::value::Index;





#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputStore {
    pub outputs: HashMap<OutputKey, VecDeque<OutputLine>>,
    current_idx: u128
}
impl OutputStore {
    pub fn new() -> Self {
        OutputStore {
            outputs: HashMap::new(),
            current_idx: 1
        }
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) -> &OutputLine {
        if !self.outputs.contains_key(key) {
            self.outputs.insert(key.clone(), VecDeque::new());
        }
        let deque = self.outputs.get_mut(key).unwrap();
        deque.push_back(OutputLine {
            value: line,
            index: self.current_idx
        });
        self.current_idx += 1;
        // TODO move to a config field
        if deque.len() > 8096 {
            deque.pop_front();
        }

        deque.iter().last().unwrap()
    }

    pub fn query_lines(&self, num_lines: usize, max_idx: Option<u128>) -> Vec<(&OutputKey, &str)> {
        let max_idx = max_idx.unwrap_or(self.current_idx);
        let mut result: Vec<(&OutputKey, &str)> = Vec::with_capacity(num_lines);
        let mut bucket_indices: HashMap<OutputKey, Option<usize>> = self.outputs.iter()
            .map(|(key, lines)| {
                if lines.len() == 0 {
                    // If the bucket has 0 lines, then there's nothing we could ever return
                    (key.clone(), None)
                } else if lines.iter().last().map(|OutputLine { index, ..}| index <= &max_idx).unwrap_or(false) {
                    // If all lines have an index lower than the given max index, then the starting index is the length
                    // of the bucket
                    (key.clone(), (lines.len() - 1).into())
                } else {
                    // Otherwise find the partition point for elements at most the given index, and select the last
                    // index of the first partition
                    (
                        key.clone(),
                        (lines.partition_point(|OutputLine { index, .. }| {
                            index <= &max_idx
                        }) - 1).into()
                    )
                }
            }).collect();

        // Loop until the result vec is fully populated, or we run out of lines.
        while result.len() < num_lines && bucket_indices.iter().any(|(_, value)| value.is_some()) {
            // Figure out the bucket with the next line
            let next_bucket = self.outputs.iter().max_by_key(|(key, lines)| {
                if let Some(cur_idx) = bucket_indices.get(key).unwrap() {
                    lines.get(*cur_idx).unwrap().index
                } else {
                    0
                }
            }).unwrap().0;
            let cur_idx = bucket_indices.get(next_bucket).unwrap().unwrap();

            // Push the relevant line into the returned results
            result.push((next_bucket, &self.outputs.get(next_bucket).unwrap().get(cur_idx).unwrap().value));
            // Update the current index for the bucket
            if cur_idx > 0 {
                *bucket_indices.get_mut(next_bucket).unwrap() = Some(cur_idx - 1);
            } else {
                *bucket_indices.get_mut(next_bucket).unwrap() = None;
            }
        }

        result.into_iter().rev().collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct OutputKey {
    pub name: String,
    pub service_ref: String,
    pub kind: OutputKind
}
impl OutputKey {
    pub const STD: &'static str = "std";
    pub const CTRL: &'static str = "ctrl";

    pub fn new(name: String, service_ref: String, kind: OutputKind) -> Self {
        OutputKey {
            name,
            service_ref,
            kind
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum OutputKind {
    Compile,
    Run
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputLine {
    pub value: String,
    pub index: u128
}
