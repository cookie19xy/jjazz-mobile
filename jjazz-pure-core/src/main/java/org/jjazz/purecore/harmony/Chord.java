package org.jjazz.purecore.harmony;

import java.util.*;
import java.util.stream.Collectors;

/**
 * A chord is an ordered list of notes kept sorted by ascending pitch.
 * Extracted from JJazzLab Chord, no external dependencies beyond Note.
 */
public class Chord implements Cloneable {

    private final List<Note> notes = new ArrayList<>();

    public Chord() {}

    public Chord(List<? extends Note> newNotes) {
        for (Note note : newNotes) add(note);
    }

    public int size() { return notes.size(); }

    public void add(Note note) {
        int index = Collections.binarySearch(notes, note,
            (n1, n2) -> Integer.compare(n1.getPitch(), n2.getPitch()));
        if (index < 0) notes.add(-(index + 1), note);
    }

    public Note removePitch(int p) {
        int i = indexOfPitch(p);
        return (i == -1) ? null : removeNote(i);
    }

    @Override
    public Chord clone() {
        Chord c = new Chord();
        for (Note n : notes) c.add(n);
        return c;
    }

    public void clear() { notes.clear(); }

    public List<Note> getNotes() { return List.copyOf(notes); }

    public List<Integer> getPitches() {
        return notes.stream().map(Note::getPitch).toList();
    }

    public Chord getRelativePitchChord() {
        Chord c = new Chord();
        for (Note n : notes) c.add(new Note(n.getRelativePitch()));
        return c;
    }

    /**
     * Build a parallel chord using relative pitches and preserving voicing intervals.
     */
    public Chord computeParallelChord(List<Integer> relPitches, boolean startBelow) {
        Chord result = new Chord();
        if (relPitches.size() != getRelativePitchChord().size()) {
            throw new IllegalArgumentException("this=" + this + " relPitches=" + relPitches);
        }
        if (size() == 0) return result;

        Note n0 = getNote(0);
        int destRelPitch = relPitches.get(0);
        int destPitch = startBelow ? n0.getLowerPitch(destRelPitch, true) : n0.getUpperPitch(destRelPitch, true);
        Note lastNote = new Note(destPitch);
        result.add(lastNote);

        if (size() > 1) {
            HashMap<Integer, Integer> mapSave = new HashMap<>();
            mapSave.put(n0.getRelativePitch(), destRelPitch);
            List<Integer> skipOctaves = computeSkipOctaves();
            int destPitchIndex = 1;

            for (int i = 1; i < skipOctaves.size(); i++) {
                Note n = getNote(i);
                destRelPitch = mapSave.getOrDefault(n.getRelativePitch(), relPitches.get(destPitchIndex++));
                for (int j = 0; j <= skipOctaves.get(i); j++) {
                    destPitch = lastNote.getUpperPitch(destRelPitch, false);
                    lastNote = new Note(destPitch);
                }
                result.add(lastNote);
                mapSave.put(n.getRelativePitch(), destRelPitch);
            }
        }
        return result;
    }

    public Note getNote(int index) {
        if (index < 0 || index >= notes.size())
            throw new IllegalArgumentException("index=" + index + " notes=" + notes);
        return notes.get(index);
    }

    public Note removeNote(int index) {
        if (index < 0 || index >= notes.size())
            throw new IllegalArgumentException("i=" + index);
        return notes.remove(index);
    }

    public int indexOfPitch(int p) {
        if (!Note.checkPitch(p)) throw new IllegalArgumentException("pitch=" + p);
        for (int i = 0; i < notes.size(); i++) {
            int pitch = notes.get(i).getPitch();
            if (pitch > p) break;
            if (pitch == p) return i;
        }
        return -1;
    }

    public int indexOfRelativePitch(int p) {
        p = p % 12;
        for (int i = 0; i < notes.size(); i++) {
            if (notes.get(i).getRelativePitch() == p) return i;
        }
        return -1;
    }

    public int getMaxPitch() { return notes.isEmpty() ? 0 : notes.get(notes.size() - 1).getPitch(); }
    public int getMinPitch() { return notes.isEmpty() ? 0 : notes.get(0).getPitch(); }

    public void centerChordOctave(int lowPitch, int maxPitch) {
        int cCentralOctave = (getMaxPitch() + getMinPitch()) / (2 * 12);
        int nbTransposeOctave = (lowPitch + maxPitch) / (2 * 12) - cCentralOctave;
        transpose(nbTransposeOctave * 12);
    }

    public void transpose(int t) {
        Note[] oldNotes = notes.toArray(new Note[0]);
        clear();
        for (Note n : oldNotes) add(n.getTransposed(t));
    }

    public int computeDistance(Chord c) {
        if (c == null || c.size() != size())
            throw new IllegalArgumentException("c=" + c + " this=" + this);
        int dist = 0;
        for (int i = 0; i < size(); i++)
            dist += Math.abs(c.getNote(i).getPitch() - notes.get(i).getPitch());
        return dist;
    }

    public void normalize() {
        int lowest = getMinPitch();
        transpose((lowest / 12) * -12);
    }

    @Override
    public boolean equals(Object obj) {
        if (obj instanceof Chord c) return notes.equals(c.notes);
        throw new ClassCastException("obj=" + obj);
    }

    public boolean equalsRelative(Chord c) {
        if (c.size() != size()) return false;
        if (size() <= 1) return true;
        for (int i = 1; i < size(); i++) {
            if (c.getNote(i).getPitch() - c.getNote(i - 1).getPitch()
                != getNote(i).getPitch() - getNote(i - 1).getPitch())
                return false;
        }
        return true;
    }

    @Override
    public int hashCode() {
        String s = notes.stream().map(n -> String.valueOf(n.getPitch())).collect(Collectors.joining());
        return s.hashCode();
    }

    @Override
    public String toString() {
        return "Chord" + notes.toString();
    }

    public String toRelativeNoteString(Note.Accidental acc) {
        StringJoiner joiner = new StringJoiner(",", "[", "]");
        notes.forEach(n -> joiner.add(acc == null ? n.toRelativeNoteString() : n.toRelativeNoteString(acc)));
        return joiner.toString();
    }

    public String toAbsoluteNoteString() {
        StringBuilder sb = new StringBuilder("[");
        for (Note n : notes) sb.append(n.toPianoOctaveString()).append(",");
        if (!notes.isEmpty()) sb.setLength(sb.length() - 1);
        sb.append("]");
        return sb.toString();
    }

    private List<Integer> computeSkipOctaves() {
        ArrayList<Integer> res = new ArrayList<>();
        int lastPitch = -1;
        for (Note ne : notes) {
            if (res.isEmpty()) {
                res.add(0);
            } else {
                int pitchDelta = ne.getPitch() - lastPitch;
                res.add(pitchDelta / 12);
            }
            lastPitch = ne.getPitch();
        }
        return res;
    }
}
