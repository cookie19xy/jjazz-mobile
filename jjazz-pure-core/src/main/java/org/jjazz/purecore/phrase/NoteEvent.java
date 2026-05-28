package org.jjazz.purecore.phrase;

import static com.google.common.base.Preconditions.checkArgument;
import org.jjazz.purecore.harmony.Note;
import org.jjazz.purecore.util.FloatRange;
import java.util.Objects;

/**
 * A Note with a position in beats. Extracted from JJazzLab NoteEvent.
 */
public class NoteEvent extends Note {

    private final float position;

    public NoteEvent(int pitch, float duration, int velocity, float posInBeats, Accidental acc) {
        super(pitch, duration, velocity, acc);
        if (posInBeats < 0) throw new IllegalArgumentException("posInBeats=" + posInBeats);
        this.position = posInBeats;
    }

    public NoteEvent(int pitch, float duration, int velocity, float posInBeats) {
        this(pitch, duration, velocity, posInBeats, Accidental.FLAT);
    }

    public NoteEvent(Note n, float posInBeats) {
        this(n.getPitch(), n.getDurationInBeats(), n.getVelocity(), posInBeats, n.getAccidental());
    }

    public float getPositionInBeats() { return position; }

    public FloatRange getBeatRange() {
        return new FloatRange(position, position + getDurationInBeats());
    }

    public boolean isBefore(NoteEvent other) { return position < other.position; }

    /** Create a new NoteEvent with some fields modified. Pass negative/null to keep original. */
    public NoteEvent setAll(int pitch, float duration, int velocity, float posInBeats, Accidental acc) {
        return new NoteEvent(
            pitch < 0 ? getPitch() : pitch,
            duration < 0 ? getDurationInBeats() : duration,
            velocity < 0 ? getVelocity() : velocity,
            posInBeats < 0 ? getPositionInBeats() : posInBeats,
            acc == null ? getAccidental() : acc
        );
    }

    public NoteEvent setPitch(int newPitch) { return setAll(newPitch, -1, -1, -1, null); }
    public NoteEvent setDuration(float newDur) { return setAll(-1, newDur, -1, -1, null); }
    public NoteEvent setVelocity(int newVel) { return setAll(-1, -1, newVel, -1, null); }
    public NoteEvent setPosition(float newPos) { return setAll(-1, -1, -1, newPos, null); }

    @Override
    public NoteEvent clone() {
        return new NoteEvent(getPitch(), getDurationInBeats(), getVelocity(), position, getAccidental());
    }

    @Override
    public int compareTo(Note n) {
        if (n == this) return 0;
        if (n instanceof NoteEvent ne) {
            int res = Float.compare(position, ne.position);
            if (res == 0) res = super.compareTo(n);
            return res;
        }
        return super.compareTo(n);
    }

    @Override
    public boolean equals(Object o) { return this == o; }

    @Override
    public int hashCode() { return System.identityHashCode(this); }

    @Override
    public String toString() {
        return String.format("[%s, p=%.3f, d=%.3f, v=%d]",
            toPianoOctaveString(), position, getDurationInBeats(), getVelocity());
    }

    public static String saveAsString(NoteEvent ne) {
        return Note.saveAsString(ne, true) + ":" + ne.position;
    }
}
