package org.jjazz.purecore.phrase;

import org.jjazz.purecore.harmony.*;
import org.jjazz.purecore.humanizer.Humanizer;
import org.jjazz.purecore.quantizer.Quantizer;
import org.jjazz.purecore.quantizer.Quantizer.Quantization;
import org.jjazz.purecore.util.FloatRange;
import org.jjazz.purecore.util.IntRange;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import static org.junit.jupiter.api.Assertions.*;

import java.text.ParseException;
import java.util.*;

class PhraseModelTest {

    // ============ NoteEvent Tests ============

    @Test @DisplayName("NoteEvent: basic creation")
    void testNoteEventBasic() {
        NoteEvent ne = new NoteEvent(60, 1.0f, 100, 2.5f);
        assertEquals(60, ne.getPitch());
        assertEquals(1.0f, ne.getDurationInBeats(), 0.001);
        assertEquals(100, ne.getVelocity());
        assertEquals(2.5f, ne.getPositionInBeats(), 0.001);
    }

    @Test @DisplayName("NoteEvent: setAll modifications")
    void testNoteEventSetAll() {
        NoteEvent ne = new NoteEvent(60, 1.0f, 100, 0f);
        NoteEvent modified = ne.setAll(62, -1, 80, 2.0f, null);
        assertEquals(62, modified.getPitch());
        assertEquals(1.0f, modified.getDurationInBeats(), 0.001); // unchanged
        assertEquals(80, modified.getVelocity());
        assertEquals(2.0f, modified.getPositionInBeats(), 0.001);
    }

    // ============ Phrase Tests ============

    @Test @DisplayName("Phrase: sorted by position")
    void testPhraseSorting() {
        Phrase p = new Phrase(0);
        p.add(new NoteEvent(67, 0.5f, 100, 2.0f)); // G4 at beat 2
        p.add(new NoteEvent(60, 1.0f, 100, 0f));   // C4 at beat 0
        p.add(new NoteEvent(64, 1.0f, 100, 1.0f)); // E4 at beat 1

        assertEquals(3, p.size());
        assertEquals(60, p.first().getPitch()); // earliest = C4
        assertEquals(67, p.last().getPitch());  // latest = G4
    }

    @Test @DisplayName("Phrase: replace note")
    void testPhraseReplace() {
        Phrase p = new Phrase(0);
        NoteEvent ne = new NoteEvent(60, 1.0f, 100, 0f);
        p.add(ne);
        NoteEvent ne2 = new NoteEvent(62, 1.0f, 80, 0f);
        p.replace(ne, ne2);
        assertEquals(1, p.size());
        assertEquals(62, p.first().getPitch());
        assertEquals(80, p.first().getVelocity());
    }

    @Test @DisplayName("Phrase: notes beat range")
    void testPhraseBeatRange() {
        Phrase p = new Phrase(0);
        p.add(new NoteEvent(60, 1.0f, 100, 2.0f));
        p.add(new NoteEvent(64, 2.0f, 100, 1.0f));
        FloatRange r = p.getNotesBeatRange();
        assertEquals(1.0f, r.from, 0.001);
        assertEquals(3.0f, r.to, 0.001); // latest end: pos 2 + dur 1 = 3, not 4
    }

    // ============ FloatRange / IntRange Tests ============

    @Test @DisplayName("FloatRange: contains")
    void testFloatRange() {
        FloatRange r = new FloatRange(1.0f, 4.0f);
        assertTrue(r.contains(2.0f, true));
        assertFalse(r.contains(1.0f, false));
        assertEquals(3.0f, r.size(), 0.001);
    }

    @Test @DisplayName("IntRange: contains")
    void testIntRange() {
        IntRange r = new IntRange(0, 3);
        assertTrue(r.contains(2));
        assertFalse(r.contains(4));
        assertEquals(4, r.size());
    }

    // ============ Humanizer Tests ============

    @Test @DisplayName("Humanizer: no-op with zero config")
    void testHumanizerZeroConfig() {
        Phrase p = new Phrase(0);
        NoteEvent ne = new NoteEvent(60, 1.0f, 100, 0f);
        p.add(ne);

        Humanizer h = new Humanizer(p, TimeSignature.FOUR_FOUR, 120);
        h.registerNotes(List.of(ne));
        h.setConfig(Humanizer.ZERO_CONFIG);
        h.humanize();

        // Should be unchanged
        assertEquals(1, p.size());
        NoteEvent result = p.first();
        assertEquals(60, result.getPitch());
        assertEquals(0f, result.getPositionInBeats(), 0.001);
        assertEquals(100, result.getVelocity());
    }

    @Test @DisplayName("Humanizer: applies timing and velocity changes")
    void testHumanizerWithConfig() {
        Phrase p = new Phrase(0);
        for (int i = 0; i < 20; i++) {
            p.add(new NoteEvent(60 + i, 0.5f, 100, i * 1.0f));
        }
        Humanizer h = new Humanizer(p, TimeSignature.FOUR_FOUR, 120);
        h.registerNotes(new ArrayList<>(p));
        h.setConfig(Humanizer.DEFAULT_CONFIG);
        h.humanize();

        // At least some notes should have changed
        boolean anyChanged = false;
        for (NoteEvent ne : p) {
            if (Math.abs(ne.getPositionInBeats() - Math.round(ne.getPositionInBeats())) > 0.001
                || ne.getVelocity() != 100) {
                anyChanged = true;
                break;
            }
        }
        assertTrue(anyChanged, "Humanizer should modify at least some notes");
    }

    // ============ Quantizer Tests ============

    @Test @DisplayName("Quantizer: OFF does nothing")
    void testQuantizerOff() {
        Position pos = new Position(1, 2.3f);
        Position q = Quantizer.quantize(Quantization.OFF, pos, TimeSignature.FOUR_FOUR, 1.0f, 10);
        assertEquals(2.3f, q.getBeat(), 0.001);
    }

    @Test @DisplayName("Quantizer: BEAT hard quantize")
    void testQuantizerBeat() {
        Position pos = new Position(1, 2.3f);
        Position q = Quantizer.quantize(Quantization.BEAT, pos, TimeSignature.FOUR_FOUR, 1.0f, 10);
        assertEquals(2.0f, q.getBeat(), 0.001);
        assertEquals(1, q.getBar());
    }

    @Test @DisplayName("Quantizer: HALF_BEAT")
    void testQuantizerHalfBeat() {
        Position pos = new Position(0, 1.3f);
        Position q = Quantizer.quantize(Quantization.HALF_BEAT, pos, TimeSignature.FOUR_FOUR, 1.0f, 10);
        assertEquals(1.5f, q.getBeat(), 0.001);
    }

    @Test @DisplayName("Quantizer: ONE_QUARTER_BEAT")
    void testQuantizerQuarterBeat() {
        Position pos = new Position(0, 0.6f);
        Position q = Quantizer.quantize(Quantization.ONE_QUARTER_BEAT, pos, TimeSignature.FOUR_FOUR, 1.0f, 10);
        assertEquals(0.5f, q.getBeat(), 0.001);
    }

    // ============ Integration Tests ============

    @Test @DisplayName("Integration: build a simple bass line Phrase")
    void testBuildBassLine() throws ParseException {
        ChordSymbol[] chords = {
            new ChordSymbol("Dm7"),
            new ChordSymbol("G7"),
            new ChordSymbol("Cmaj7")
        };

        Phrase bassLine = new Phrase(0);
        int rootOctave = 36; // C2 area

        for (int bar = 0; bar < chords.length; bar++) {
            ChordSymbol cs = chords[bar];
            int rootPitch = rootOctave + cs.getRootNote().getRelativePitch();
            // Simple pattern: root on beat 1, fifth on beat 3
            bassLine.add(new NoteEvent(rootPitch, 1.5f, 100, bar * 4));
            bassLine.add(new NoteEvent(
                rootPitch + 7, // fifth
                0.5f, 80, bar * 4 + 2f));
        }

        assertEquals(6, bassLine.size());

        // Check first note is D (38)
        NoteEvent first = bassLine.first();
        assertEquals(38, first.getPitch()); // D2

        // Verify sorting
        float lastPos = -1;
        for (NoteEvent ne : bassLine) {
            assertTrue(ne.getPositionInBeats() >= lastPos);
            lastPos = ne.getPositionInBeats();
        }
    }
}
