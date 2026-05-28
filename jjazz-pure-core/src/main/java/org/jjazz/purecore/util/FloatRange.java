package org.jjazz.purecore.util;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.Objects;

/**
 * A float range [from, to]. Immutable.
 */
public final class FloatRange {
    public final float from;
    public final float to;

    public FloatRange(float from, float to) {
        checkArgument(from <= to, "from=%s to=%s", from, to);
        this.from = from;
        this.to = to;
    }

    public float size() { return to - from; }

    public boolean contains(float value, boolean inclusive) {
        return inclusive ? value >= from && value <= to : value > from && value < to;
    }

    public boolean contains(FloatRange other, boolean inclusive) {
        if (inclusive) return other.from >= from && other.to <= to;
        return other.from > from && other.to < to;
    }

    public FloatRange getTransformed(float deltaFrom, float deltaTo) {
        return new FloatRange(from + deltaFrom, to + deltaTo);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof FloatRange fr)) return false;
        return Float.compare(from, fr.from) == 0 && Float.compare(to, fr.to) == 0;
    }

    @Override
    public int hashCode() { return Objects.hash(from, to); }

    @Override
    public String toString() { return "[" + from + ";" + to + "]"; }
}
