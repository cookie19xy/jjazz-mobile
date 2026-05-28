use crate::harmony::ChordSymbol;

/// A sequence of chord symbols, one per bar (simplified).
#[derive(Debug, Clone)]
pub struct ChordSequence {
    pub chords: Vec<ChordSymbol>,
}

impl ChordSequence {
    pub fn new() -> Self { Self { chords: Vec::new() } }
    pub fn push(&mut self, cs: ChordSymbol) { self.chords.push(cs); }
    pub fn len(&self) -> usize { self.chords.len() }
    pub fn get(&self, bar: usize) -> Option<&ChordSymbol> { self.chords.get(bar) }
}
