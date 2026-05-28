package org.jjazz.purecore.harmony;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import java.text.ParseException;
import java.util.Objects;

/**
 * A musical note with pitch, duration, velocity, and accidental display.
 * Immutable. Extracted from JJazzLab Note, stripped of ResUtil/NetBeans deps.
 */
public class Note implements Comparable<Note>, Cloneable {

    public static final int VELOCITY_MIN = 0;
    public static final int VELOCITY_STD = 100;
    public static final int VELOCITY_MAX = 127;
    public static final int PITCH_MIN = 0;
    public static final int PITCH_STD = 60; // C4 (middle C)
    public static final int PITCH_MAX = 127;
    public static final int OCTAVE_MIN = 0;
    public static final int OCTAVE_STD = 4;
    public static final int OCTAVE_MAX = 10;

    public enum Accidental { FLAT, SHARP }

    public static final String[] NOTES_FLAT = {"C","Db","D","Eb","E","F","Gb","G","Ab","A","Bb","B"};
    public static final String[] NOTES_SHARP = {"C","C#","D","D#","E","F","F#","G","G#","A","A#","B"};

    private final int pitch;
    private final SymbolicDuration symbolicDuration;
    private final float beatDuration;
    private final Accidental accidental;
    private final int velocity;
    private String pianoOctaveString; // cached

    public Note() {
        this(PITCH_STD);
    }

    public Note(int p) {
        this(p, SymbolicDuration.QUARTER, VELOCITY_STD, Accidental.FLAT);
    }

    public Note(int p, float bd) {
        this(p, bd, VELOCITY_STD);
    }

    public Note(int p, float bd, int v) {
        this(p, bd, v, Accidental.FLAT);
    }

    public Note(int p, SymbolicDuration sd, int v, Accidental alt) {
        Objects.requireNonNull(sd);
        Objects.requireNonNull(alt);
        checkArgument(checkPitch(p), "p=%s", p);
        checkArgument(checkVelocity(v), "v=%s", v);
        this.pitch = p;
        this.beatDuration = sd.getDuration();
        this.symbolicDuration = sd;
        this.accidental = alt;
        this.velocity = v;
    }

    public Note(int p, float bd, int v, Accidental alt) {
        Objects.requireNonNull(alt);
        checkArgument(checkPitch(p), "p=%s", p);
        checkArgument(bd > 0, "bd=%s", bd);
        checkArgument(checkVelocity(v), "v=%s", v);
        this.pitch = p;
        this.beatDuration = bd;
        this.symbolicDuration = SymbolicDuration.getSymbolicDuration(bd);
        this.accidental = alt;
        this.velocity = v;
    }

    public Note(Note n, int newPitch) {
        this(newPitch, n.beatDuration, n.velocity, n.accidental);
    }

    public Note(Note n, Accidental alt) {
        this(n.pitch, n.beatDuration, n.velocity, alt);
    }

    /**
     * Parse a note string like "C", "Db", "A#", "C!4", "Eb!3".
     * Octave range is [0-10].
     */
    public Note(String s) throws ParseException {
        Objects.requireNonNull(s);
        String str = s.trim();
        Accidental alt = Accidental.FLAT;

        if (str.isEmpty()) {
            throw new ParseException("Empty note string", 0);
        }

        String degreeStr = str.substring(0, 1);
        if (str.length() > 1 && (str.charAt(1) == 'b' || str.charAt(1) == '#')) {
            degreeStr = str.substring(0, 2);
        }

        String octaveStr = null;
        int octaveIndex = str.indexOf("!");
        if (octaveIndex == str.length() - 1) {
            throw new ParseException("Invalid note: " + str, str.length() - 1);
        }
        if (octaveIndex != -1) {
            octaveStr = str.substring(octaveIndex + 1);
        }

        int relPitch = -1;
        // Handle special cases first
        if (degreeStr.compareTo("Cb") == 0)      { relPitch = 11; alt = Accidental.FLAT; }
        else if (degreeStr.compareToIgnoreCase("B#") == 0) { relPitch = 0; alt = Accidental.SHARP; }
        else if (degreeStr.compareToIgnoreCase("E#") == 0) { relPitch = 5; alt = Accidental.SHARP; }
        else if (degreeStr.compareToIgnoreCase("Fb") == 0) { relPitch = 4; alt = Accidental.FLAT; }
        else {
            for (int i = 0; i < NOTES_FLAT.length; i++) {
                if (degreeStr.compareToIgnoreCase(NOTES_FLAT[i]) == 0) { relPitch = i; alt = Accidental.FLAT; break; }
                if (degreeStr.compareToIgnoreCase(NOTES_SHARP[i]) == 0) { relPitch = i; alt = Accidental.SHARP; break; }
            }
        }

        if (relPitch == -1) {
            throw new ParseException("Invalid note: " + str, 0);
        }

        int octave = OCTAVE_STD;
        if (octaveStr != null) {
            try {
                octave = Integer.parseInt(octaveStr);
            } catch (NumberFormatException e) {
                throw new ParseException("Invalid note: " + str + " : " + e.getLocalizedMessage(), 0);
            }
        }
        if (!checkOctave(octave)) {
            throw new ParseException("Invalid note: " + str, 0);
        }

        this.pitch = octave * 12 + relPitch;
        this.beatDuration = SymbolicDuration.QUARTER.getDuration();
        this.symbolicDuration = SymbolicDuration.QUARTER;
        this.accidental = alt;
        this.velocity = VELOCITY_STD;
    }

    @Override
    public Note clone() {
        return new Note(this.pitch, this.beatDuration, this.velocity, this.accidental);
    }

    public int getPitch() { return pitch; }
    public int getRelativePitch() { return pitch % 12; }
    public float getDurationInBeats() { return beatDuration; }
    public SymbolicDuration getSymbolicDuration() { return symbolicDuration; }
    public int getOctave() { return pitch / 12; }
    public Accidental getAccidental() { return accidental; }

    public int getRelativeAscInterval(Note relNote) {
        int delta = relNote.getRelativePitch() - getRelativePitch();
        return delta < 0 ? delta + 12 : delta;
    }

    public int getRelativeDescInterval(Note relNote) {
        int delta = getRelativePitch() - relNote.getRelativePitch();
        return delta < 0 ? delta + 12 : delta;
    }

    /**
     * Shortest pitch delta from this note's relative pitch to relPitch. [-5, +6].
     */
    public int getRelativePitchDelta(int relPitch) {
        checkArgument(relPitch >= 0 && relPitch <= 11);
        int pitchDelta = relPitch - getRelativePitch();
        if (pitchDelta > 6) pitchDelta -= 12;
        else if (pitchDelta < -5) pitchDelta += 12;
        return pitchDelta;
    }

    public Note getTransposed(int t) {
        return new Note(this, pitch + t);
    }

    public Note getCentered(int lowPitch, int highPitch) {
        checkArgument(lowPitch <= highPitch - 12);
        int newPitch = pitch;
        while (newPitch < lowPitch) newPitch += 12;
        while (newPitch > highPitch) newPitch -= 12;
        return new Note(this, newPitch);
    }

    public Note getTransposed(int pitchShift, int pitchLimit) {
        checkArgument(pitchLimit >= 13 && pitchLimit <= 119);
        int newPitch = this.pitch + pitchShift;
        if (pitchShift > 0) while (newPitch > pitchLimit) newPitch -= 12;
        else if (pitchShift < 0) while (newPitch < pitchLimit) newPitch += 12;
        return new Note(this, newPitch);
    }

    public Note getTransposedWithinOctave(int t) {
        int rp = getRelativePitch() + t;
        if (t != 0 && (t % 12) != 0) {
            rp = rp < 0 ? rp + 12 : rp;
            rp = rp > 11 ? rp - 12 : rp;
            rp = getOctave() * 12 + rp;
        }
        return new Note(this, rp);
    }

    public int getLowerPitch(int relPitch, boolean acceptEquals) {
        checkArgument(relPitch >= 0 && relPitch <= 11);
        int p = getOctave() * 12 + relPitch;
        if ((relPitch == getRelativePitch() && !acceptEquals) || relPitch > getRelativePitch()) {
            p = (getOctave() - 1) * 12 + relPitch;
        }
        return Math.max(p, 0);
    }

    public int getUpperPitch(int relPitch, boolean inclusive) {
        checkArgument(relPitch >= 0 && relPitch <= 11);
        int p = getOctave() * 12 + relPitch;
        if ((relPitch == getRelativePitch() && !inclusive) || relPitch < getRelativePitch()) {
            p = (getOctave() + 1) * 12 + relPitch;
        }
        return Math.min(p, 127);
    }

    public int getClosestPitch(int relPitch) {
        int up = getUpperPitch(relPitch, true);
        int low = getLowerPitch(relPitch, true);
        return (up - getPitch() > getPitch() - low) ? low : up;
    }

    public int getVelocity() { return velocity; }

    public boolean equalsRelativePitch(Note n) {
        return getRelativePitch() == n.getRelativePitch();
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (!(obj instanceof Note other)) return false;
        return this.pitch == other.pitch
            && Float.floatToIntBits(this.beatDuration) == Float.floatToIntBits(other.beatDuration)
            && this.velocity == other.velocity;
    }

    @Override
    public int hashCode() {
        int hash = 7;
        hash = 67 * hash + this.pitch;
        hash = 67 * hash + Float.floatToIntBits(this.beatDuration);
        hash = 67 * hash + this.velocity;
        return hash;
    }

    @Override
    public int compareTo(Note n) {
        int res = Integer.compare(pitch, n.pitch);
        if (res == 0) {
            res = Float.compare(beatDuration, n.beatDuration);
            if (res == 0) res = Integer.compare(velocity, n.velocity);
        }
        return res;
    }

    @Override
    public String toString() {
        return toPianoOctaveString();
    }

    public String toPianoOctaveBeatString() {
        return toPianoOctaveString() + ":" + beatDuration;
    }

    public String toRelativeNoteString() {
        return toRelativeNoteString(accidental);
    }

    public String toRelativeNoteString(Accidental acc) {
        return acc == Accidental.FLAT ? NOTES_FLAT[getRelativePitch()] : NOTES_SHARP[getRelativePitch()];
    }

    public String toPianoOctaveString() {
        if (pianoOctaveString == null) {
            pianoOctaveString = toRelativeNoteString() + (getOctave() - 1);
        }
        return pianoOctaveString;
    }

    public boolean isFlat() { return accidental == Accidental.FLAT; }

    public boolean isWhiteKey() { return isWhiteKey(this.pitch); }

    public int getWhiteKeyPitch() {
        int res = getPitch();
        if (!isWhiteKey()) {
            res = isFlat() ? res + 1 : res - 1;
            res = Math.max(0, Math.min(127, res));
        }
        return res;
    }

    // --- Static helpers ---

    public static boolean checkPitch(int p) { return p >= PITCH_MIN && p <= PITCH_MAX; }
    public static boolean checkVelocity(int v) { return v >= VELOCITY_MIN && v <= VELOCITY_MAX; }
    public static boolean checkOctave(int o) { return o >= OCTAVE_MIN && o <= OCTAVE_MAX; }

    public static boolean isWhiteKey(int pitch) {
        pitch = pitch % 12;
        return pitch != 1 && pitch != 3 && pitch != 6 && pitch != 8 && pitch != 10;
    }

    public static int getNormalizedRelPitch(int absPitch) {
        if (absPitch >= 0) return absPitch % 12;
        else return (12 - (-absPitch % 12)) % 12;
    }

    public static int limitPitch(int pitch, int lowPitch, int highPitch) {
        checkArgument(lowPitch <= highPitch - 11);
        int newPitch = pitch;
        while (newPitch < lowPitch) newPitch += 12;
        while (newPitch > highPitch) newPitch -= 12;
        return newPitch;
    }

    public static Note[] getChromaticNotesArray(int pitchFrom, int pitchTo) {
        checkArgument(pitchFrom >= 0 && pitchTo >= 0 && pitchFrom <= pitchTo);
        Note[] notes = new Note[pitchTo - pitchFrom + 1];
        for (int i = 0; i < notes.length; i++) notes[i] = new Note(pitchFrom + i);
        return notes;
    }

    public static Note parsePianoOctaveString(String s) throws ParseException {
        if (s == null || s.length() < 2) throw new ParseException("Invalid string s=" + s, 0);
        int index = (s.charAt(1) == '#' || s.charAt(1) == 'b') ? 2 : 1;
        String strNote = s.substring(0, index);
        String strOctave = s.substring(index);
        int octave = Integer.parseInt(strOctave) + 1;
        return new Note(strNote + "!" + octave);
    }

    public static Note parsePianoOctaveBeatString(String s) throws ParseException {
        Objects.requireNonNull(s);
        s = s.strip();
        int index = s.indexOf(":");
        if (index == -1) throw new ParseException("Invalid PianoOctaveBeat string: " + s, 0);
        Note n = parsePianoOctaveString(s.substring(0, index));
        String strDur = s.substring(index + 1);
        float dur;
        try { dur = Float.parseFloat(strDur); }
        catch (NumberFormatException ex) { throw new ParseException("Invalid duration: " + s, index + 1); }
        return new Note(n.getPitch(), dur);
    }

    public static String saveAsString(Note n, boolean skipAccidental) {
        checkNotNull(n);
        if (skipAccidental) return n.pitch + "," + n.velocity + "," + n.beatDuration;
        else return n.pitch + "," + n.accidental + "," + n.velocity + "," + n.beatDuration;
    }

    public static Note loadAsString(String s) throws ParseException {
        checkNotNull(s);
        String[] strs = s.split(",");
        if (strs.length == 4 || strs.length == 3) {
            try {
                int p = Integer.parseInt(strs[0]);
                Accidental alt = strs.length == 3 ? Accidental.FLAT : Accidental.valueOf(strs[1]);
                int v = Integer.parseInt(strs[strs.length == 3 ? 1 : 2]);
                float bd = Float.parseFloat(strs[strs.length == 3 ? 2 : 3]);
                return new Note(p, bd, v, alt);
            } catch (IllegalArgumentException ex) { /* fall through */ }
        }
        throw new ParseException("Note.loadAsString() Invalid Note string s=" + s, 0);
    }
}
