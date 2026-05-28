package org.jjazz.purecore.util;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.Objects;

/**
 * An integer range [from, to]. Immutable.
 */
public final class IntRange {
    public final int from;
    public final int to;

    public IntRange(int from, int to) {
        checkArgument(from <= to, "from=%s to=%s", from, to);
        this.from = from;
        this.to = to;
    }

    public boolean isEmpty() { return from == to; }

    public int size() { return to - from + 1; }

    public boolean contains(int value) { return value >= from && value <= to; }

    public boolean contains(IntRange other) { return other.from >= from && other.to <= to; }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof IntRange ir)) return false;
        return from == ir.from && to == ir.to;
    }

    @Override
    public int hashCode() { return Objects.hash(from, to); }

    @Override
    public String toString() { return "[" + from + ";" + to + "]"; }
}
