package org.jjazz.purecore.musicgen;

import org.jjazz.purecore.harmony.*;
import org.jjazz.purecore.phrase.*;
import org.jjazz.purecore.humanizer.Humanizer;
import org.jjazz.purecore.quantizer.Quantizer;
import org.jjazz.purecore.quantizer.Quantizer.Quantization;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import static org.junit.jupiter.api.Assertions.*;

import java.text.ParseException;
import java.util.*;

/**
 * End-to-end verification: chord symbols → phrase generation → humanize → quantize.
 */
class EndToEndTest {

    @Test @DisplayName("E2E: Dm7 G7 Cmaj7 → simple bass line")
    void testSimpleBassGeneration() throws ParseException {
        // 1. Parse chord symbols
        ChordSymbol dm7 = new ChordSymbol("Dm7");
        ChordSymbol g7 = new ChordSymbol("G7");
        ChordSymbol cmaj7 = new ChordSymbol("Cmaj7");

        assertEquals("Dm7", dm7.getName());
        assertEquals("G7", g7.getName());
        assertEquals("CM7", cmaj7.getName()); // canonized to CM7

        // 2. Build a simple bass line Phrase (4/4, 3 bars, root on beat 1, fifth on beat 3)
        Phrase bassPhrase = new Phrase(0);
        ChordSymbol[] chords = {dm7, g7, cmaj7};

        for (int bar = 0; bar < chords.length; bar++) {
            ChordSymbol cs = chords[bar];
            int rootPitch = 36 + cs.getRootNote().getRelativePitch(); // C2=36
            float barStart = bar * 4f;

            // Root note on beat 1 (duration 1.5 beats)
            bassPhrase.add(new NoteEvent(rootPitch, 1.5f, 100, barStart));
            // Fifth on beat 3 (duration 1 beat)
            bassPhrase.add(new NoteEvent(rootPitch + 7, 1.0f, 85, barStart + 2f));
        }

        assertEquals(6, bassPhrase.size());

        // 3. Humanize
        Humanizer humanizer = new Humanizer(bassPhrase, TimeSignature.FOUR_FOUR, 120);
        humanizer.registerNotes(new ArrayList<>(bassPhrase));
        humanizer.setConfig(Humanizer.DEFAULT_CONFIG);
        humanizer.humanize();

        // Should still have 6 notes after humanizing (might change but shouldn't lose notes)
        assertTrue(bassPhrase.size() >= 4, "Humanizer shouldn't remove most notes");

        // 4. Verify positions are within bounds
        for (NoteEvent ne : bassPhrase) {
            assertTrue(ne.getPositionInBeats() >= 0, "Note shouldn't start before bar 0");
            assertTrue(ne.getPositionInBeats() + ne.getDurationInBeats() <= 12.5f,
                "Note shouldn't extend past bar 3");
        }
    }

    @Test @DisplayName("E2E: verify chord type attributes survive round-trip")
    void testChordTypeRoundTrip() throws ParseException {
        // Major 7th
        ChordSymbol cmaj7 = new ChordSymbol("Cmaj7");
        assertTrue(cmaj7.getChordType().isMajor());
        assertTrue(cmaj7.getChordType().isSeventh());
        assertTrue(cmaj7.getChordType().isSeventhMajor());

        // Minor 7th
        ChordSymbol dm7 = new ChordSymbol("Dm7");
        assertTrue(dm7.getChordType().isMinor());
        assertTrue(dm7.getChordType().isSeventhMinor());

        // Half diminished
        ChordSymbol bm7b5 = new ChordSymbol("Bm7b5");
        assertTrue(bm7b5.getChordType().isFifthFlat());
        assertTrue(bm7b5.getChordType().isSeventhMinor());

        // Diminished 7th
        ChordSymbol bdim7 = new ChordSymbol("Bdim7");
        assertEquals("dim7", bdim7.getChordType().getName());
        // dim7 has bb7 = 6th, not a seventh
        assertFalse(bdim7.getChordType().isSeventh());
    }

    @Test @DisplayName("E2E: generate phrase, quantize, verify timing")
    void testQuantizePipeline() throws ParseException {
        Phrase p = new Phrase(0);

        // Add notes at slightly off-grid positions
        p.add(new NoteEvent(60, 0.5f, 100, 0.05f));   // should go to 0.0
        p.add(new NoteEvent(62, 0.5f, 100, 1.08f));   // should go to 1.0
        p.add(new NoteEvent(64, 0.5f, 100, 2.02f));   // should go to 2.0
        p.add(new NoteEvent(65, 0.5f, 100, 3.15f));   // should go to 3.0

        // Quantize hard
        List<NoteEvent> quantized = new ArrayList<>();
        for (NoteEvent ne : p) {
            Position pos = Position.fromAbsoluteBeat(ne.getPositionInBeats(), TimeSignature.FOUR_FOUR);
            Position qPos = Quantizer.quantize(Quantization.BEAT, pos, TimeSignature.FOUR_FOUR, 1.0f, 10);
            float newBeat = qPos.toAbsoluteBeat(TimeSignature.FOUR_FOUR);
            NoteEvent qNe = ne.setAll(-1, -1, -1, newBeat, null);
            quantized.add(qNe);
        }

        // After hard quantize, all should be on exact beats
        float[] expected = {0f, 1f, 2f, 3f};
        for (int i = 0; i < quantized.size(); i++) {
            assertEquals(expected[i], quantized.get(i).getPositionInBeats(), 0.01f,
                "Note " + i + " should be on beat " + expected[i]);
        }
    }
}
