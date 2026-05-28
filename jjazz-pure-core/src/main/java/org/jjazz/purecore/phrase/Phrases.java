package org.jjazz.purecore.phrase;

import java.util.*;

/**
 * Utility methods for Phrase and NoteEvent manipulation.
 */
public final class Phrases {

    private Phrases() {}

    /**
     * Fix overlapping same-pitch notes by shortening earlier notes.
     * @return Map of original notes to their replacements (or null if removed).
     */
    public static Map<NoteEvent, NoteEvent> fixOverlappedNotes(Phrase phrase) {
        Map<NoteEvent, NoteEvent> result = new HashMap<>();
        // Group by pitch
        Map<Integer, List<NoteEvent>> byPitch = new TreeMap<>();
        for (NoteEvent ne : phrase) {
            byPitch.computeIfAbsent(ne.getPitch(), k -> new ArrayList<>()).add(ne);
        }
        for (var entry : byPitch.entrySet()) {
            List<NoteEvent> nes = entry.getValue();
            for (int i = 0; i < nes.size() - 1; i++) {
                NoteEvent ne = nes.get(i);
                NoteEvent next = nes.get(i + 1);
                if (ne.getPositionInBeats() + ne.getDurationInBeats() > next.getPositionInBeats()) {
                    // Overlap - shorten
                    float newDur = next.getPositionInBeats() - ne.getPositionInBeats();
                    if (newDur > 0.02f) {
                        NoteEvent fixed = ne.setDuration(newDur);
                        result.put(ne, fixed);
                        nes.set(i, fixed);
                    } else {
                        result.put(ne, null);
                        nes.remove(i);
                        i--;
                    }
                }
            }
        }
        // Apply changes
        for (var entry : result.entrySet()) {
            NoteEvent old = entry.getKey();
            NoteEvent replacement = entry.getValue();
            phrase.remove(old);
            if (replacement != null) phrase.add(replacement);
        }
        return result;
    }

    /** Get notes that cross a given position. */
    public static List<NoteEvent> getCrossingNotes(Phrase phrase, float posInBeats, boolean inclusive) {
        List<NoteEvent> res = new ArrayList<>();
        for (NoteEvent ne : phrase) {
            float end = ne.getPositionInBeats() + ne.getDurationInBeats();
            if (inclusive ? end >= posInBeats && ne.getPositionInBeats() <= posInBeats
                          : end > posInBeats && ne.getPositionInBeats() < posInBeats) {
                res.add(ne);
            }
        }
        return res;
    }
}
