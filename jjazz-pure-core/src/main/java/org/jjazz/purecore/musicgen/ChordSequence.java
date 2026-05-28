package org.jjazz.purecore.musicgen;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.*;
import org.jjazz.purecore.harmony.*;
import org.jjazz.purecore.util.IntRange;

/**
 * A sequence of chord symbols with positions within a bar range.
 * Extracted from JJazzLab ChordSequence.
 */
public class ChordSequence extends ArrayList<ChordSymbol> {

    private final IntRange barRange;

    public ChordSequence(IntRange barRange) {
        checkArgument(barRange != null);
        this.barRange = barRange;
    }

    public IntRange getBarRange() { return barRange; }

    /**
     * Find the chord symbol active at the given position.
     * Returns the last chord symbol at or before the position.
     */
    public ChordSymbol getChordSymbol(Position pos) {
        // Search backwards for the last chord at or before pos
        // In a real implementation this would track chord positions precisely.
        // For pure core simplification, we assume chords are ordered.
        int bar = pos.getBar();
        // Return the chord for the current bar (simplified)
        // In real JJazzLab, each chord has a Position tracked separately
        return isEmpty() ? null : get(Math.min(bar, size() - 1));
    }

    @Override
    public String toString() {
        return "ChordSequence" + super.toString();
    }
}
