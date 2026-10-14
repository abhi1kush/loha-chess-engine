//! Transposition table. Entries are always replaced, matching the C engine.

use crate::defs::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct HashEntry {
    pub pos_key: u64,
    pub mov: u32,
    pub score: i32,
    pub depth: i32,
    pub flags: u8,
}

#[derive(Clone, Debug)]
pub struct HashTable {
    entries: Vec<HashEntry>,
    pub new_write: i32,
    pub overwrite: i32,
    pub hit: i32,
    pub cut: i32,
}

impl HashTable {
    pub fn with_entries(count: usize) -> Self {
        let count = count.max(1);
        Self {
            entries: vec![HashEntry::default(); count],
            new_write: 0,
            overwrite: 0,
            hit: 0,
            cut: 0,
        }
    }

    pub fn with_mb(mb: usize) -> Self {
        let mb = mb.clamp(4, MAX_HASH_MB);
        let mut count = (mb * 1024 * 1024) / std::mem::size_of::<HashEntry>();
        if count > 2 {
            count -= 2;
        }
        println!("HashTable init complete with {count} entries ");
        Self::with_entries(count)
    }

    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = HashEntry::default();
        }
        self.new_write = 0;
    }

    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    fn index(&self, key: u64) -> usize {
        (key % self.entries.len() as u64) as usize
    }

    pub fn store(&mut self, key: u64, mut score: i32, mov: u32, flags: u8, depth: i32, ply: i32) {
        let i = self.index(key);
        if self.entries[i].pos_key == 0 {
            self.new_write += 1;
        } else {
            self.overwrite += 1;
        }
        if score > IS_MATE {
            score += ply;
        } else if score < -IS_MATE {
            score -= ply;
        }
        self.entries[i] = HashEntry {
            pos_key: key,
            mov,
            score,
            depth,
            flags,
        };
    }

    /// Returns `(cutoff, move, score)`. The move is filled whenever the key matches,
    /// which is what the C probe did through its out-parameter. Cutoff is true only
    /// when the stored bound is deep enough to end the node.
    pub fn probe(
        &mut self,
        key: u64,
        ply: i32,
        alpha: i32,
        beta: i32,
        depth: i32,
    ) -> (bool, u32, i32) {
        let i = self.index(key);
        let entry = self.entries[i];
        if entry.pos_key != key {
            return (false, NOMOVE, 0);
        }
        let mov = entry.mov;
        if entry.depth < depth {
            return (false, mov, 0);
        }
        self.hit += 1;
        let mut score = entry.score;
        if score > IS_MATE {
            score -= ply;
        } else if score < -IS_MATE {
            score += ply;
        }
        match entry.flags {
            HF_ALPHA if score <= alpha => (true, mov, alpha),
            HF_BETA if score >= beta => (true, mov, beta),
            HF_EXACT => (true, mov, score),
            _ => (false, mov, score),
        }
    }

    pub fn probe_pv(&self, key: u64) -> u32 {
        let entry = self.entries[self.index(key)];
        if entry.pos_key == key {
            entry.mov
        } else {
            NOMOVE
        }
    }
}
