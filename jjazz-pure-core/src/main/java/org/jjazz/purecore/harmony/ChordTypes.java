package org.jjazz.purecore.harmony;

/**
 * Built-in chord type definitions.
 * <p>
 * This replaces the original JJazzLab ChordTypeDatabase SPI.
 * All standard chord types are registered by name for lookup.
 * <p>
 * The chord type constructor parameters are:
 * (base, extension, family, i9, i3, i11, i5, i13, i7)
 * where each i value is -1 (flat), 0 (natural), +1 (sharp), or 9 (NOT_PRESENT).
 */
public final class ChordTypes {

    // Prevent instantiation
    private ChordTypes() {}

    // The static initializer in ChordType itself registers each instance.
    // These constructor calls below create and auto-register all types.

    // === Triads ===
    public static final ChordType MAJOR      = new ChordType("",  "",  ChordType.Family.MAJOR,      9, 0, 9, 0, 9, 9);
    public static final ChordType MINOR      = new ChordType("m", "",  ChordType.Family.MINOR,      9, -1, 9, 0, 9, 9);
    public static final ChordType DIM        = new ChordType("dim","", ChordType.Family.DIMINISHED, 9, -1, 9, -1, 9, 9);
    public static final ChordType AUG        = new ChordType("aug","", ChordType.Family.MAJOR,      9, 0, 9, 1, 9, 9);
    public static final ChordType SUS4       = new ChordType("sus","", ChordType.Family.SUS,        9, 9, 0, 0, 9, 9);
    public static final ChordType SUS2       = new ChordType("2",  "",  ChordType.Family.SUS,       0, 9, 9, 0, 9, 9);

    // === Sevenths ===
    public static final ChordType MAJ7       = new ChordType("M7", "",  ChordType.Family.MAJOR,      9, 0, 9, 0, 9, 0);
    public static final ChordType DOM7       = new ChordType("7",  "",  ChordType.Family.SEVENTH,    9, 0, 9, 0, 9, -1);
    public static final ChordType MIN7       = new ChordType("m7", "",  ChordType.Family.MINOR,      9, -1, 9, 0, 9, -1);
    public static final ChordType MIN_MAJ7   = new ChordType("mM7","",  ChordType.Family.MINOR,      9, -1, 9, 0, 9, 0);
    public static final ChordType DIM7       = new ChordType("dim7","", ChordType.Family.DIMINISHED, 9, -1, 9, -1, 0, 9); // bb7 = major 6th
    public static final ChordType HALF_DIM7  = new ChordType("m7b5","", ChordType.Family.DIMINISHED, 9, -1, 9, -1, 9, -1);
    public static final ChordType AUG7       = new ChordType("7aug","", ChordType.Family.SEVENTH,    9, 0, 9, 1, 9, -1);
    public static final ChordType AUG_MAJ7   = new ChordType("M7aug","",ChordType.Family.MAJOR,      9, 0, 9, 1, 9, 0);
    public static final ChordType SUS7       = new ChordType("7sus","", ChordType.Family.SUS,        9, 9, 0, 0, 9, -1);
    public static final ChordType SUS4_MAJ7  = new ChordType("M7sus","",ChordType.Family.SUS,        9, 9, 0, 0, 9, 0);

    // === Sixths ===
    public static final ChordType MAJ6       = new ChordType("6",  "",  ChordType.Family.MAJOR,      9, 0, 9, 0, 0, 9);
    public static final ChordType MIN6       = new ChordType("m6", "",  ChordType.Family.MINOR,      9, -1, 9, 0, 0, 9);

    // === Ninths ===
    public static final ChordType MAJ9       = new ChordType("M9", "",  ChordType.Family.MAJOR,      0, 0, 9, 0, 9, 0);
    public static final ChordType DOM9       = new ChordType("9",  "",  ChordType.Family.SEVENTH,    0, 0, 9, 0, 9, -1);
    public static final ChordType MIN9       = new ChordType("m9", "",  ChordType.Family.MINOR,      0, -1, 9, 0, 9, -1);

    // === Altered Dominants ===
    public static final ChordType DOM7_FLAT9     = new ChordType("7", "b9",  ChordType.Family.SEVENTH, -1, 0, 9, 0, 9, -1);
    public static final ChordType DOM7_SHARP9    = new ChordType("7", "#9",  ChordType.Family.SEVENTH, 1, 0, 9, 0, 9, -1);
    public static final ChordType DOM7_FLAT5     = new ChordType("7", "b5",  ChordType.Family.SEVENTH, 9, 0, 9, -1, 9, -1);
    public static final ChordType DOM7_SHARP5    = new ChordType("7", "#5",  ChordType.Family.SEVENTH, 9, 0, 9, 1, 9, -1);
    public static final ChordType DOM7_SHARP11   = new ChordType("7", "#11", ChordType.Family.SEVENTH, 9, 0, 1, 0, 9, -1);
    public static final ChordType DOM7_FLAT9_SHARP5  = new ChordType("7", "b9#5", ChordType.Family.SEVENTH, -1, 0, 9, 1, 9, -1);
    public static final ChordType DOM7_SHARP9_SHARP5 = new ChordType("7", "#9#5", ChordType.Family.SEVENTH, 1, 0, 9, 1, 9, -1);
    public static final ChordType DOM7_FLAT9_FLAT5   = new ChordType("7", "b9b5", ChordType.Family.SEVENTH, -1, 0, 9, -1, 9, -1);

    // === Major 7 Altered ===
    public static final ChordType MAJ7_SHARP11  = new ChordType("M7","#11",ChordType.Family.MAJOR, 9, 0, 1, 0, 9, 0);
    public static final ChordType MAJ9_SHARP11  = new ChordType("M9","#11",ChordType.Family.MAJOR, 0, 0, 1, 0, 9, 0);

    // === Thirteenths ===
    public static final ChordType DOM13      = new ChordType("13", "",  ChordType.Family.SEVENTH, 0, 0, 9, 0, 0, -1);
    public static final ChordType MAJ13      = new ChordType("M13","",  ChordType.Family.MAJOR,   0, 0, 9, 0, 0, 0);
    public static final ChordType MIN13      = new ChordType("m13","",  ChordType.Family.MINOR,   0, -1, 9, 0, 0, -1);

    // === Elevenths ===
    public static final ChordType DOM11      = new ChordType("11", "",  ChordType.Family.SEVENTH, 0, 0, 0, 0, 9, -1);
    public static final ChordType MIN11      = new ChordType("m11","",  ChordType.Family.MINOR,   0, -1, 0, 0, 9, -1);

    // === alt ===
    public static final ChordType DOM7_ALT   = new ChordType("7", "alt", ChordType.Family.SEVENTH, -1, 0, 9, -1, 9, -1);

    // Note: more exotic types can be added as needed.
    // The original JJazzLab defines ~100+ chord types. These are the most commonly used ones.
}
