package org.jjazz.purecore.harmony;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import static org.junit.jupiter.api.Assertions.*;

import java.text.ParseException;
import java.util.List;

/**
 * Unit tests for the Harmony model layer.
 */
class HarmonyModelTest {

    // ======================== Note Tests ========================

    @Test @DisplayName("Note: pitch construction and accessors")
    void testNotePitch() {
        Note c4 = new Note(60);
        assertEquals(60, c4.getPitch());
        assertEquals(0, c4.getRelativePitch()); // C
        assertEquals(5, c4.getOctave()); // 60/12 = 5
        assertEquals(100, c4.getVelocity());
        assertEquals("C4", c4.toPianoOctaveString());
    }

    @Test @DisplayName("Note: relative pitch")
    void testNoteRelativePitch() {
        assertEquals(0, new Note(0).getRelativePitch());   // C
        assertEquals(1, new Note(1).getRelativePitch());   // Db
        assertEquals(3, new Note(3).getRelativePitch());   // Eb
        assertEquals(7, new Note(7).getRelativePitch());   // G
        assertEquals(11, new Note(11).getRelativePitch()); // B
        assertEquals(0, new Note(12).getRelativePitch());  // C (octave up)
    }

    @Test @DisplayName("Note: transposition")
    void testNoteTranspose() {
        Note c4 = new Note(60);
        Note d4 = c4.getTransposed(2);
        assertEquals(62, d4.getPitch());
        assertEquals("D4", d4.toPianoOctaveString());
    }

    @Test @DisplayName("Note: string parsing")
    void testNoteParsing() throws ParseException {
        Note n1 = new Note("C");
        assertEquals(48, n1.getPitch()); // C!4 = C4 = 48+0 = 48, wait... actually OCTAVE_STD=4, so 4*12+0=48
        
        Note n2 = new Note("C!4");
        assertEquals(48, n2.getPitch());
        
        Note n3 = new Note("Eb!3");
        assertEquals(36 + 3, n3.getPitch());
        
        Note n4 = new Note("F#");
        assertEquals(4 * 12 + 6, n4.getPitch());
    }

    @Test @DisplayName("Note: closest pitch")
    void testNoteClosestPitch() {
        Note c4 = new Note(60); // C4
        assertEquals(59, c4.getClosestPitch(11)); // B - should be B3(59), not B4(71)
        assertEquals(61, c4.getClosestPitch(1));  // Db - should be Db4(61)
    }

    @Test @DisplayName("Note: equals and hashCode")
    void testNoteEquality() {
        Note n1 = new Note(60, 1.0f, 100);
        Note n2 = new Note(60, 1.0f, 100);
        Note n3 = new Note(60, 2.0f, 100);
        assertEquals(n1, n2);
        assertNotEquals(n1, n3);
        assertEquals(n1.hashCode(), n2.hashCode());
    }

    // ======================== Degree Tests ========================

    @Test @DisplayName("Degree: pitch values")
    void testDegreePitch() {
        assertEquals(0, Degree.ROOT.getPitch());
        assertEquals(4, Degree.THIRD.getPitch());
        assertEquals(3, Degree.THIRD_FLAT.getPitch());
        assertEquals(7, Degree.FIFTH.getPitch());
        assertEquals(11, Degree.SEVENTH.getPitch());
        assertEquals(10, Degree.SEVENTH_FLAT.getPitch());
    }

    @Test @DisplayName("Degree: getDegreeMostProbable")
    void testDegreeMostProbable() {
        assertEquals(Degree.ROOT, Degree.getDegreeMostProbable(0));
        assertEquals(Degree.THIRD_FLAT, Degree.getDegreeMostProbable(3));
        assertEquals(Degree.FIFTH, Degree.getDegreeMostProbable(7));
        assertEquals(Degree.SIXTH_OR_THIRTEENTH, Degree.getDegreeMostProbable(9));
    }

    // ======================== ChordType Tests ========================

    @Test @DisplayName("ChordType: major triad")
    void testChordTypeMajor() {
        ChordType maj = ChordTypes.MAJOR;
        assertEquals("", maj.getName());
        assertEquals(ChordType.Family.MAJOR, maj.getFamily());
        assertEquals(3, maj.getNbDegrees());
        
        List<Degree> degs = maj.getDegrees();
        assertEquals(Degree.ROOT, degs.get(0));
        assertEquals(Degree.THIRD, degs.get(1));
        assertEquals(Degree.FIFTH, degs.get(2));
    }

    @Test @DisplayName("ChordType: minor seventh")
    void testChordTypeMin7() {
        ChordType m7 = ChordTypes.MIN7;
        assertEquals("m7", m7.getName());
        assertEquals(ChordType.Family.MINOR, m7.getFamily());
        assertEquals(4, m7.getNbDegrees());
        
        assertTrue(m7.isMinor());
        assertTrue(m7.isSeventh());
        assertTrue(m7.isSeventhMinor());
    }

    @Test @DisplayName("ChordType: dominant 7th")
    void testChordTypeDom7() {
        ChordType dom7 = ChordTypes.DOM7;
        assertEquals("7", dom7.getName());
        assertTrue(dom7.isMajor());
        assertTrue(dom7.isSeventh());
        assertTrue(dom7.isSeventhMinor());
    }

    @Test @DisplayName("ChordType: lookup by name")
    void testChordTypeLookup() {
        assertEquals(ChordTypes.MAJOR, ChordType.getByName(""));
        assertEquals(ChordTypes.MIN7, ChordType.getByName("m7"));
        assertEquals(ChordTypes.DOM7, ChordType.getByName("7"));
        assertEquals(ChordTypes.HALF_DIM7, ChordType.getByName("m7b5"));
    }

    @Test @DisplayName("ChordType: simplify")
    void testChordTypeSimplify() {
        ChordType m9 = ChordTypes.MIN9; // 5 degrees
        assertEquals(5, m9.getNbDegrees());
        
        ChordType simplified = m9.getSimplified(4);
        assertEquals(4, simplified.getNbDegrees());
        // Should reduce to m7
        assertEquals(ChordTypes.MIN7, simplified);
    }

    // ======================== ChordSymbol Tests ========================

    @Test @DisplayName("ChordSymbol: basic parsing")
    void testChordSymbolParsing() throws ParseException {
        ChordSymbol cs = new ChordSymbol("Dm7");
        assertEquals("D", cs.getRootNote().toRelativeNoteString());
        assertEquals("m7", cs.getChordType().getName());
        assertEquals("Dm7", cs.getName());
    }

    @Test @DisplayName("ChordSymbol: slash chord")
    void testChordSymbolSlash() throws ParseException {
        ChordSymbol cs = new ChordSymbol("Cm7/G");
        assertEquals("C", cs.getRootNote().toRelativeNoteString());
        assertEquals("G", cs.getBassNote().toRelativeNoteString());
        assertTrue(cs.isSlashChord());
    }

    @Test @DisplayName("ChordSymbol: altered chords")
    void testChordSymbolAltered() throws ParseException {
        ChordSymbol cs1 = new ChordSymbol("F7b9");
        assertEquals("F", cs1.getRootNote().toRelativeNoteString());
        assertEquals("7b9", cs1.getChordType().getName());

        ChordSymbol cs2 = new ChordSymbol("Bb7#11");
        assertEquals("Bb", cs2.getRootNote().toRelativeNoteString());
    }

    // ======================== Scale Tests ========================

    @Test @DisplayName("Scale: major scale degrees")
    void testScaleMajor() {
        Scale major = StandardScales.MAJOR;
        assertEquals("Major", major.getName());
        assertEquals(7, major.size());
        
        List<Note> cMajorNotes = major.getNotes(60); // C4
        assertEquals(7, cMajorNotes.size());
        assertEquals(60, cMajorNotes.get(0).getPitch()); // C
        assertEquals(62, cMajorNotes.get(1).getPitch()); // D
        assertEquals(64, cMajorNotes.get(2).getPitch()); // E
        assertEquals(65, cMajorNotes.get(3).getPitch()); // F
        assertEquals(67, cMajorNotes.get(4).getPitch()); // G
        assertEquals(69, cMajorNotes.get(5).getPitch()); // A
        assertEquals(71, cMajorNotes.get(6).getPitch()); // B
    }

    @Test @DisplayName("Scale: pentatonic")
    void testScalePentatonic() {
        Scale majorPent = StandardScales.MAJOR_PENTATONIC;
        assertEquals(5, majorPent.size());
        
        List<Note> notes = majorPent.getNotes(60);
        // C D E G A
        assertEquals(60, notes.get(0).getPitch()); // C
        assertEquals(62, notes.get(1).getPitch()); // D
        assertEquals(64, notes.get(2).getPitch()); // E
        assertEquals(67, notes.get(3).getPitch()); // G
        assertEquals(69, notes.get(4).getPitch()); // A
    }

    // ======================== Position Tests ========================

    @Test @DisplayName("Position: bar and beat")
    void testPosition() {
        Position p = new Position(2, 3.5f);
        assertEquals(2, p.getBar());
        assertEquals(3.5f, p.getBeat(), 0.001);
        assertEquals(3, p.getBeatInt());
        assertEquals(0.5f, p.getBeatFractionalPart(), 0.001);
    }

    @Test @DisplayName("Position: absolute beat conversion")
    void testPositionAbsoluteBeat() {
        TimeSignature ts = TimeSignature.FOUR_FOUR;
        Position p = new Position(2, 2.5f);
        float absBeat = p.toAbsoluteBeat(ts);
        assertEquals(2 * 4 + 2.5f, absBeat, 0.001);
    }

    // ======================== TimeSignature Tests ========================

    @Test @DisplayName("TimeSignature: basic properties")
    void testTimeSignature() {
        assertEquals(4, TimeSignature.FOUR_FOUR.getNumerator());
        assertEquals(4, TimeSignature.FOUR_FOUR.getDenominator());
        assertEquals(1.0f, TimeSignature.FOUR_FOUR.getNaturalBeat(), 0.001);
        assertEquals(4.0f, TimeSignature.FOUR_FOUR.getNbNaturalBeats(), 0.001);
        
        assertEquals(1.5f, TimeSignature.SIX_EIGHT.getNaturalBeat(), 0.001);
        assertEquals(9.0f, TimeSignature.SIX_EIGHT.getNbNaturalBeats(), 0.001);
    }

    // ======================== Chord Tests ========================

    @Test @DisplayName("Chord: add and sort notes")
    void testChordAdd() {
        Chord c = new Chord();
        c.add(new Note(67)); // G4
        c.add(new Note(60)); // C4
        c.add(new Note(64)); // E4
        
        assertEquals(3, c.size());
        assertEquals(60, c.getNote(0).getPitch()); // C4 (lowest)
        assertEquals(64, c.getNote(1).getPitch()); // E4
        assertEquals(67, c.getNote(2).getPitch()); // G4
    }

    @Test @DisplayName("Chord: relative equality")
    void testChordRelativeEquality() {
        Chord c1 = new Chord(List.of(new Note(60), new Note(64), new Note(67))); // C E G
        Chord c2 = new Chord(List.of(new Note(55), new Note(59), new Note(62))); // G B D
        assertTrue(c1.equalsRelative(c2)); // Both are major triads in root position
    }

    // ======================== Integration: ChordSymbol + Chord ========================

    @Test @DisplayName("Integration: Cm7 chord notes")
    void testCm7ChordNotes() throws ParseException {
        ChordSymbol cm7 = new ChordSymbol("Cm7");
        Chord chord = cm7.getChord(48); // C3 = 48
        // Cm7 = C Eb G Bb
        assertEquals(4, chord.size());
        assertEquals(48, chord.getNote(0).getPitch()); // C3
        assertEquals(51, chord.getNote(1).getPitch()); // Eb3
        assertEquals(55, chord.getNote(2).getPitch()); // G3
        assertEquals(58, chord.getNote(3).getPitch()); // Bb3
    }

    @Test @DisplayName("Integration: Dm7b5 chord degrees")
    void testHalfDimChord() throws ParseException {
        ChordSymbol dm7b5 = new ChordSymbol("Dm7b5");
        assertEquals("D", dm7b5.getRootNote().toRelativeNoteString());
        assertEquals("m7b5", dm7b5.getChordType().getName());
        assertTrue(dm7b5.getChordType().isFifthFlat());
    }
}
