package org.jjazz.purecore.harmony;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.Objects;

/**
 * Musical time signature (e.g. 4/4, 3/4, 6/8).
 * Extracted from JJazzLab TimeSignature.
 */
public final class TimeSignature {

    public static final TimeSignature FOUR_FOUR = new TimeSignature(4, 4);
    public static final TimeSignature THREE_FOUR = new TimeSignature(3, 4);
    public static final TimeSignature TWO_FOUR = new TimeSignature(2, 4);
    public static final TimeSignature FIVE_FOUR = new TimeSignature(5, 4);
    public static final TimeSignature SIX_FOUR = new TimeSignature(6, 4);
    public static final TimeSignature SEVEN_FOUR = new TimeSignature(7, 4);
    public static final TimeSignature SIX_EIGHT = new TimeSignature(6, 8);
    public static final TimeSignature TWELVE_EIGHT = new TimeSignature(12, 8);

    private final int numerator;
    private final int denominator;
    private final float halfBarBeat;
    private final float naturalBeat;

    public TimeSignature(int numerator, int denominator) {
        checkArgument(numerator > 0, "numerator=%s", numerator);
        checkArgument(denominator > 0, "denominator=%s", denominator);
        this.numerator = numerator;
        this.denominator = denominator;

        // A "natural beat" is 1 beat = 1/denominator of a whole note in 4/4 time
        // In 4/4: 1 beat = quarter note. In 6/8: 1 beat = dotted quarter (3 eighth notes)
        if (denominator == 8) {
            naturalBeat = 1.5f;
        } else {
            naturalBeat = 1f;
        }
        halfBarBeat = (numerator * naturalBeat) / 2f;
    }

    public int getNumerator() { return numerator; }
    public int getDenominator() { return denominator; }

    /**
     * Duration of one natural beat (e.g. 1.0 for 4/4, 1.5 for 6/8).
     */
    public float getNaturalBeat() { return naturalBeat; }

    /**
     * Beat position of the middle of the bar.
     */
    public float getHalfBarBeat(boolean swing) {
        // Note: swing handling simplified in pure core
        return halfBarBeat;
    }

    /**
     * Total number of natural beats per bar.
     */
    public float getNbNaturalBeats() {
        return numerator * naturalBeat;
    }

    /**
     * Check if the given beat position is valid within this time signature.
     */
    public boolean checkBeat(float beat) {
        return beat >= 0 && beat < getNbNaturalBeats();
    }

    /**
     * Get the bar index that contains the given absolute beat position.
     */
    public int getBarIndex(float absoluteBeat) {
        return (int) (absoluteBeat / getNbNaturalBeats());
    }

    @Override
    public String toString() {
        return numerator + "/" + denominator;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof TimeSignature ts)) return false;
        return numerator == ts.numerator && denominator == ts.denominator;
    }

    @Override
    public int hashCode() {
        return Objects.hash(numerator, denominator);
    }
}
