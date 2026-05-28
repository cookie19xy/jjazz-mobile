package org.jjazz.purecore.harmony;

import java.util.*;

/**
 * A musical scale defined by ascending degrees starting from ROOT.
 * Example: MajorPentatonic starting on Eb = Eb, F, G, Bb, C.
 * <p>
 * Extracted from JJazzLab Scale.
 */
public class Scale {

    private final String name;
    private final List<Degree> degrees;
    private final List<Note> notes0; // notes starting on C0
    private final List<Integer> intervals;

    public Scale(String name, Degree... degs) {
        if (name == null || name.isEmpty() || degs == null || degs.length == 0 || degs[0] != Degree.ROOT) {
            throw new IllegalArgumentException("name=" + name + " degrees=" + Arrays.toString(degs));
        }
        this.name = name;
        this.degrees = new ArrayList<>();
        this.notes0 = new ArrayList<>();
        this.intervals = new ArrayList<>();

        degrees.add(Degree.ROOT);
        notes0.add(new Note(0));

        int lastPitch = 0;
        for (int i = 1; i < degs.length; i++) {
            int newPitch = degs[i].getPitch();
            if (newPitch <= lastPitch)
                throw new IllegalArgumentException("Degrees must be unique and ascending. name=" + name + " degs=" + Arrays.toString(degs));
            degrees.add(degs[i]);
            notes0.add(new Note(newPitch));
            intervals.add(newPitch - lastPitch);
            lastPitch = newPitch;
        }
    }

    public String getName() { return name; }

    public List<Degree> getDegrees() { return Collections.unmodifiableList(degrees); }

    /**
     * Get the degree matching relPitch, or null.
     */
    public Degree getDegree(int relPitch) {
        for (Degree d : degrees) {
            if (d.getPitch() == relPitch) return d;
        }
        return null;
    }

    /**
     * Get all degrees derived from a natural (same Natural, different accidentals).
     */
    public List<Degree> getDegrees(Degree.Natural natural) {
        List<Degree> res = new ArrayList<>();
        for (Degree d : degrees) {
            if (d.getNatural() == natural) res.add(d);
        }
        return res;
    }

    /**
     * Get notes of this scale starting from the given root pitch.
     */
    public List<Note> getNotes(Note root) {
        List<Note> res = new ArrayList<>();
        int rootPitch = root.getPitch();
        for (Note n0 : notes0) {
            res.add(new Note(rootPitch + n0.getPitch()));
        }
        return res;
    }

    /**
     * Get notes of this scale over one octave starting from rootPitch.
     */
    public List<Note> getNotes(int rootPitch) {
        return getNotes(new Note(rootPitch));
    }

    public List<Integer> getIntervals() { return Collections.unmodifiableList(intervals); }

    /** Get the note at the given scale degree index (0-based). */
    public Note getNote(int rootPitch, int degreeIndex) {
        return new Note(rootPitch + notes0.get(degreeIndex % notes0.size()).getPitch());
    }

    public int size() { return degrees.size(); }

    @Override
    public String toString() {
        return name + ":" + degrees;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof Scale scale)) return false;
        return degrees.equals(scale.degrees);
    }

    @Override
    public int hashCode() { return degrees.hashCode(); }
}
