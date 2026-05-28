package org.jjazz.purecore.harmony;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * The possible degrees that make a chord (root, third, fifth, seventh, extensions).
 * <p>
 * Extracted from JJazzLab Degree, stripped of NetBeans dependencies.
 */
public enum Degree {
    ROOT(Natural.ROOT, 0),
    NINTH_FLAT(Natural.NINTH, -1),
    NINTH(Natural.NINTH, 0),
    NINTH_SHARP(Natural.NINTH, 1),
    THIRD_FLAT(Natural.THIRD, -1),
    THIRD(Natural.THIRD, 0),
    FOURTH_OR_ELEVENTH(Natural.ELEVENTH, 0),
    ELEVENTH_SHARP(Natural.ELEVENTH, 1),
    FIFTH_FLAT(Natural.FIFTH, -1),
    FIFTH(Natural.FIFTH, 0),
    FIFTH_SHARP(Natural.FIFTH, 1),
    THIRTEENTH_FLAT(Natural.SIXTH, -1),
    SIXTH_OR_THIRTEENTH(Natural.SIXTH, 0),
    SEVENTH_FLAT(Natural.SEVENTH, -1),
    SEVENTH(Natural.SEVENTH, 0);

    /**
     * The natural degrees.
     */
    public enum Natural {
        ROOT(1, 0), NINTH(9, 2), THIRD(3, 4), ELEVENTH(11, 5), FIFTH(5, 7), SIXTH(13, 9), SEVENTH(7, 11);

        private final int intValue;
        private final int pitch;

        Natural(int value, int pitch) {
            this.intValue = value;
            this.pitch = pitch;
        }

        public int getIntValue() { return intValue; }
        public int getPitch() { return pitch; }
        public String toStringShort() { return String.valueOf(intValue); }

        public static Natural getFromIntValue(int intValue) {
            for (Natural b : values()) {
                if (b.intValue == intValue) return b;
            }
            return null;
        }

        public static Natural get(int relPitch) {
            for (Natural b : values()) {
                if (b.pitch == relPitch) return b;
            }
            return null;
        }
    }

    private final Natural natural;
    private final int accidental;

    Degree(Natural n, int defaultAccidental, Degree... incompatibleDegrees) {
        this.natural = n;
        this.accidental = defaultAccidental;
    }

    public int getPitch() { return natural.getPitch() + accidental; }
    public Natural getNatural() { return natural; }
    public int getAccidental() { return accidental; }

    public String toStringShort() {
        if (accidental == -1) return "b" + natural.toStringShort();
        if (accidental == +1) return "#" + natural.toStringShort();
        return natural.toStringShort();
    }

    public static Degree getDegree(Natural n, int alt) {
        Objects.requireNonNull(n);
        checkArgument(alt >= -1 && alt <= 1);
        for (Degree d : values()) {
            if (d.getNatural() == n && d.getAccidental() == alt) return d;
        }
        return null;
    }

    public static List<Degree> getDegrees(int relPitch) {
        checkArgument(relPitch >= 0 && relPitch <= 11);
        ArrayList<Degree> res = new ArrayList<>();
        for (Degree d : values()) {
            if (d.getPitch() == relPitch) res.add(d);
        }
        return res;
    }

    public static Degree getDegreeMostProbable(int relPitch) {
        checkArgument(relPitch >= 0 && relPitch <= 11);
        return switch (relPitch) {
            case 0 -> ROOT;
            case 1 -> NINTH_FLAT;
            case 2 -> NINTH;
            case 3 -> THIRD_FLAT;
            case 4 -> THIRD;
            case 5 -> FOURTH_OR_ELEVENTH;
            case 6 -> FIFTH_FLAT;
            case 7 -> FIFTH;
            case 8 -> FIFTH_SHARP;
            case 9 -> SIXTH_OR_THIRTEENTH;
            case 10 -> SEVENTH_FLAT;
            case 11 -> SEVENTH;
            default -> throw new IllegalArgumentException("relPitch=" + relPitch);
        };
    }

    public boolean equalsSixthMajorSeventh(Degree d) {
        return this == d
            || (this == SIXTH_OR_THIRTEENTH && d == SEVENTH)
            || (d == SIXTH_OR_THIRTEENTH && this == SEVENTH);
    }
}
