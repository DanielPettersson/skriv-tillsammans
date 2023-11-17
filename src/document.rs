use std::ops::Range;

use cola::{Deletion, EncodedReplica, Replica, ReplicaId};
use eframe::egui::TextBuffer;
use serde::{Deserialize, Serialize};

pub struct Document {
    buffer: String,
    crdt: Replica,
    insert_listener: Option<Box<dyn FnMut(&Insertion) + Send>>,
    delete_listener: Option<Box<dyn FnMut(&Deletion) + Send>>,
}

#[derive(Serialize, Deserialize)]
pub struct EncodedDocument {
    buffer: String,
    encoded_replica: EncodedReplica,
}

#[derive(Serialize, Deserialize)]
pub struct Insertion {
    pub text: String,
    pub crdt: cola::Insertion,
}

impl Document {
    pub fn new<S: Into<String>>(text: S, replica_id: ReplicaId) -> Self {
        let buffer = text.into();
        let crdt = Replica::new(replica_id, buffer.len());
        Document {
            buffer,
            crdt,
            insert_listener: None,
            delete_listener: None,
        }
    }

    pub fn fork(&self, new_replica_id: ReplicaId) -> Self {
        let crdt = self.crdt.fork(new_replica_id);
        Document {
            buffer: self.buffer.clone(),
            crdt,
            insert_listener: None,
            delete_listener: None,
        }
    }

    pub fn encode(&self) -> String {
        serde_json::to_string(&EncodedDocument {
            buffer: self.buffer.clone(),
            encoded_replica: self.crdt.encode(),
        })
        .unwrap()
    }

    pub fn decode(new_replica_id: ReplicaId, data: &str) -> Self {
        let encoded_document: EncodedDocument = serde_json::from_str(data).unwrap();
        let replica = Replica::decode(new_replica_id, &encoded_document.encoded_replica).unwrap();
        Document {
            buffer: encoded_document.buffer,
            crdt: replica,
            insert_listener: None,
            delete_listener: None,
        }
    }

    pub fn insert_listener(&mut self, listener: impl FnMut(&Insertion) + 'static + Send) {
        self.insert_listener = Some(Box::new(listener));
    }

    pub fn delete_listener(&mut self, listener: impl FnMut(&Deletion) + 'static + Send) {
        self.delete_listener = Some(Box::new(listener));
    }

    pub fn insert<S: Into<String>>(&mut self, insert_at: usize, text: S) -> Insertion {
        let text = text.into();
        self.buffer.insert_str(insert_at, &text);
        let crdt = self.crdt.inserted(insert_at, text.len());
        let insertion = Insertion { text, crdt };

        if let Some(l) = &mut self.insert_listener {
            l(&insertion)
        }

        insertion
    }

    pub fn delete(&mut self, range: Range<usize>) -> Deletion {
        self.buffer.replace_range(range.clone(), "");
        let deletion = self.crdt.deleted(range);

        if let Some(l) = &mut self.delete_listener {
            l(&deletion)
        }

        deletion
    }

    pub fn integrate_insertion(&mut self, insertion: &Insertion) {
        if let Some(offset) = self.crdt.integrate_insertion(&insertion.crdt) {
            self.buffer.insert_str(offset, &insertion.text);
        }
    }

    pub fn integrate_deletion(&mut self, deletion: &Deletion) {
        let ranges = self.crdt.integrate_deletion(&deletion);
        for range in ranges.into_iter().rev() {
            self.buffer.replace_range(range, "");
        }
    }
}

impl TextBuffer for Document {
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_str(&self) -> &str {
        self.buffer.as_ref()
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        self.insert(char_index, text);
        text.chars().count()
    }

    fn delete_char_range(&mut self, char_range: Range<usize>) {
        self.delete(char_range);
    }
}
