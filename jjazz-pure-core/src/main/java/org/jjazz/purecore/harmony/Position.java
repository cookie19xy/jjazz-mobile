package org.jjazz.purecore.harmony;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.Objects;

/**
 * A musical position defined by a bar index and a beat position within that bar.
 * Immutable. Extracted from JJazzLab Position.
 */
public final class Position implements Comparable<Position> {

    private final int bar;
    private final float beat;

    /** Position at bar 0, beat 0. */
    public Position() {
        this(0, 0f);
    }

    /** Position at given bar, beat 0. */
    public Position(int bar) {
        this(bar, 0f);
    }

    public Position(int bar, float beat) {
        checkArgument(bar >= 0, "bar=%s", bar);
        checkArgument(beat >= 0, "beat=%s", beat);
        this.bar = bar;
        this.beat = beat;
    }

    /** Copy constructor. */
    public Position(Position pos) {
        this(pos.bar, pos.beat);
    }

    public int getBar() { return bar; }

    /**
     * The beat position within the bar (0.0 = start of bar).
     */
    public float getBeat() { return beat; }

    /**
     * Integer part of the beat.
     */
    public int getBeatInt() { return (int) beat; }

    /**
     * Fractional part of the beat [0.0, 1.0).
     */
    public float getBeatFractionalPart() { return beat - (int) beat; }

    /**
     * Convert to absolute beat position in the given time signature.
     */
    public float toAbsoluteBeat(TimeSignature ts) {
        return bar * ts.getNbNaturalBeats() + beat;
    }

    /**
     * Create a Position from an absolute beat position and time signature.
     */
    public static Position fromAbsoluteBeat(float absoluteBeat, TimeSignature ts) {
        float nbBeatsPerBar = ts.getNbNaturalBeats();
        int bar = (int) (absoluteBeat / nbBeatsPerBar);
        float beat = absoluteBeat - bar * nbBeatsPerBar;
        return new Position(bar, beat);
    }

    @Override
    public int compareTo(Position o) {
        int cmp = Integer.compare(bar, o.bar);
        if (cmp == 0) cmp = Float.compare(beat, o.beat);
        return cmp;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof Position p)) return false;
        return bar == p.bar && Float.compare(beat, p.beat) == 0;
    }

    @Override
    public int hashCode() {
        return Objects.hash(bar, beat);
    }

    @Override
    public String toString() {
        return "bar:" + bar + ":" + String.format("%.3f", beat);
    }
}
