package org.jjazz.purecore.harmony;

import static com.google.common.base.Preconditions.checkArgument;
import java.text.ParseException;
import java.util.Objects;

/**
 * A jazz chord symbol like "Cm7", "F7b9", "Dbmaj7/F".
 * Immutable. Extracted from JJazzLab ChordSymbol, stripped of SPI deps.
 */
public class ChordSymbol implements Cloneable {

    private final String originalName;
    private final String name;
    private final Note rootNote;
    private final Note bassNote;
    private final ChordType chordType;

    /** Default "C" chord. */
    public ChordSymbol() {
        this(new Note(0), ChordTypes.MAJOR);
    }

    public ChordSymbol(Note root, ChordType ct) {
        this(root, root, ct);
    }

    public ChordSymbol(Note root, Note bass, ChordType ct) {
        Objects.requireNonNull(root);
        Objects.requireNonNull(bass);
        Objects.requireNonNull(ct);
        checkArgument(root.getRelativePitch() == bass.getRelativePitch() || bass != root,
            "root=%s bass=%s ct=%s", root, bass, ct);

        this.rootNote = root;
        this.bassNote = bass;
        this.chordType = ct;
        this.name = root.toRelativeNoteString() + ct.getName();
        this.originalName = this.name;
    }

    /**
     * Parse a chord symbol string.
     * Examples: "C", "Dm7", "F#7b9", "Bbmaj7/F", "G!3m7" (G in octave 3, m7)
     */
    public ChordSymbol(String s) throws ParseException {
        Objects.requireNonNull(s);
        String str = s.trim();
        if (str.isEmpty()) throw new ParseException("Empty chord string", 0);

        // Find root note
        int rootEnd = 1;
        if (str.length() > 1 && (str.charAt(1) == 'b' || str.charAt(1) == '#')) {
            rootEnd = 2;
        }

        // Check for octave specification (!n)
        String rootStr = str.substring(0, rootEnd);
        int octaveIndex = str.indexOf("!");
        if (octaveIndex != -1 && octaveIndex < rootEnd) {
            int end = str.indexOf("!", rootEnd);
            if (end == -1) end = str.length();
            rootStr = str.substring(0, end);
            rootEnd = end;
        }

        Note root = new Note(rootStr);

        // Find bass note (after '/')
        int slashIndex = str.indexOf('/', rootEnd);
        String bassStr = null;
        if (slashIndex != -1) {
            bassStr = str.substring(slashIndex + 1).trim();
        }

        // Extract chord type name (between root and slash/bass)
        String ctStr;
        if (bassStr != null) {
            ctStr = str.substring(rootEnd, slashIndex).trim();
        } else {
            ctStr = str.substring(rootEnd).trim();
        }

        // Handle empty chord type = major triad
        if (ctStr.isEmpty()) {
            ctStr = "";
        }

        ChordType ct = findChordType(ctStr);
        if (ct == null) {
            throw new ParseException("Unknown chord type: '" + ctStr + "' in '" + str + "'", rootEnd);
        }

        this.rootNote = root;
        this.chordType = ct;
        this.originalName = str;

        if (bassStr != null) {
            this.bassNote = new Note(bassStr);
            this.name = root.toRelativeNoteString() + ct.getName() + "/" + bassNote.toRelativeNoteString();
        } else {
            this.bassNote = root;
            this.name = root.toRelativeNoteString() + ct.getName();
        }
    }

    public String getOriginalName() { return originalName; }
    public String getName() { return name; }
    public Note getRootNote() { return rootNote; }
    public Note getBassNote() { return bassNote; }
    public ChordType getChordType() { return chordType; }

    /** Whether this chord has a different bass note (slash chord). */
    public boolean isSlashChord() {
        return !bassNote.equalsRelativePitch(rootNote);
    }

    /**
     * Get the notes of this chord at a given absolute root pitch.
     */
    public Chord getChord(int rootPitch) {
        Chord c = chordType.getChord();
        Chord result = new Chord();
        for (Note n : c.getNotes()) {
            result.add(new Note(rootPitch + n.getPitch()));
        }
        if (isSlashChord()) {
            // Add bass note below the chord
            int bassPitch = rootNote.getLowerPitch(bassNote.getRelativePitch(), true);
            result.add(new Note(bassPitch));
        }
        return result;
    }

    @Override
    public ChordSymbol clone() {
        try {
            return (ChordSymbol) super.clone();
        } catch (CloneNotSupportedException e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    public String toString() {
        return name;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof ChordSymbol cs)) return false;
        return rootNote.equals(cs.rootNote)
            && bassNote.equalsRelativePitch(cs.bassNote)
            && chordType.equals(cs.chordType);
    }

    @Override
    public int hashCode() {
        return Objects.hash(rootNote, bassNote.getRelativePitch(), chordType);
    }

    /**
     * Find a chord type by name. Handles aliases and variations.
     */
    private static ChordType findChordType(String name) {
        ChordType ct = ChordType.getByName(name);
        if (ct != null) return ct;

        // Try common aliases
        return switch (name.toLowerCase()) {
            case "maj", "maj7", "M7", "ma7", "△", "△7" -> ChordTypes.MAJ7;
            case "m", "min", "mi", "-" -> ChordTypes.MINOR;
            case "dim", "°", "o" -> ChordTypes.DIM;
            case "aug", "+" -> ChordTypes.AUG;
            case "sus", "sus4" -> ChordTypes.SUS4;
            case "sus2" -> ChordTypes.SUS2;
            case "7" -> ChordTypes.DOM7;
            case "m7", "min7", "mi7", "-7" -> ChordTypes.MIN7;
            case "dim7", "°7" -> ChordTypes.DIM7;
            case "m7b5", "ø", "halfdim" -> ChordTypes.HALF_DIM7;
            case "mmaj7", "m7+", "minmaj7" -> ChordTypes.MIN_MAJ7;
            case "6" -> ChordTypes.MAJ6;
            case "m6", "min6", "-6" -> ChordTypes.MIN6;
            case "9" -> ChordTypes.DOM9;
            case "maj9", "M9", "ma9" -> ChordTypes.MAJ9;
            case "m9", "min9", "-9" -> ChordTypes.MIN9;
            case "7b9" -> ChordTypes.DOM7_FLAT9;
            case "7#9" -> ChordTypes.DOM7_SHARP9;
            case "7b5" -> ChordTypes.DOM7_FLAT5;
            case "7#5", "7+5" -> ChordTypes.DOM7_SHARP5;
            case "7#11" -> ChordTypes.DOM7_SHARP11;
            case "7alt", "alt" -> ChordTypes.DOM7_ALT;
            case "13" -> ChordTypes.DOM13;
            case "maj13", "m13" -> ChordTypes.MAJ13;
            case "7sus", "7sus4" -> ChordTypes.SUS7;
            default -> null;
        };
    }
}
