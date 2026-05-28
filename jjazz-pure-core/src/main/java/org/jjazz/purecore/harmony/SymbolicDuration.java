package org.jjazz.purecore.harmony;

/**
 * Symbolic musical durations (whole, half, quarter, eighth, sixteenth notes
 * and their triplet/dotted variants).
 * <p>
 * Extracted from JJazzLab SymbolicDuration, stripped of NetBeans dependencies.
 */
public enum SymbolicDuration {
    UNKNOWN(0.0F, "unknown"),
    SIXTEENTH_TRIPLET(0.16667F, "1/16 triplet"),
    SIXTEENTH(0.25F, "1/16"),
    EIGHTH_TRIPLET(0.33333F, "1/8 triplet"),
    EIGHTH(0.5F, "1/8"),
    QUARTER_TRIPLET(0.66667F, "1/4 triplet"),
    EIGHTH_DOTTED(0.75F, "1/8 dotted"),
    QUARTER(1.0F, "1/4"),
    HALF_TRIPLET(1.33333F, "1/2 triplet"),
    QUARTER_DOTTED(1.5F, "1/4 dotted"),
    HALF(2.0F, "2"),
    WHOLE_TRIPLET(2.66667F, "4 triplet"),
    HALF_DOTTED(3.0F, "2 dotted"),
    WHOLE(4.0F, "4"),
    WHOLE_DOTTED(6.0F, "4 dotted");

    private final float duration;
    private final String name;

    SymbolicDuration(float d, String name) {
        if (d < 0) {
            throw new IllegalArgumentException("d=" + d);
        }
        this.duration = d;
        this.name = name;
    }

    public String getReadableName() { return name; }

    public float getDuration() { return duration; }

    public boolean isDotted() {
        return this == EIGHTH_DOTTED || this == QUARTER_DOTTED
            || this == HALF_DOTTED || this == WHOLE_DOTTED;
    }

    public boolean isTriplet() {
        return this == EIGHTH_TRIPLET || this == QUARTER_TRIPLET
            || this == HALF_TRIPLET || this == WHOLE_TRIPLET;
    }

    /**
     * Get the symbolic duration for specified beat duration (±0.01 beat tolerance).
     */
    public static SymbolicDuration getSymbolicDuration(float bd) {
        for (SymbolicDuration sd : values()) {
            if (Math.abs(bd - sd.getDuration()) < 0.01f) {
                return sd;
            }
        }
        return UNKNOWN;
    }

    /**
     * Get the closest symbolic duration for specified beat duration.
     */
    public static SymbolicDuration getClosestSymbolicDuration(float bd) {
        if (bd <= 0) return UNKNOWN;
        if (bd <= SIXTEENTH_TRIPLET.getDuration()) return SIXTEENTH_TRIPLET;

        SymbolicDuration res = WHOLE_DOTTED;
        var values = values();
        for (int i = 1; i < values.length - 1; i++) {
            var sd = values[i];
            var sdNext = values[i + 1];
            if (bd <= sdNext.getDuration()) {
                res = Math.abs(bd - sd.getDuration()) < Math.abs(bd - sdNext.getDuration()) ? sd : sdNext;
                break;
            }
        }
        return res;
    }
}
