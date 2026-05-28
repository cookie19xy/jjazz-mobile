package org.jjazz.purecore.harmony;

/**
 * Predefined standard scales (major, minor, modes, pentatonic, etc.).
 * <p>
 * Each scale is defined by its degrees starting from ROOT.
 */
public final class StandardScales {

    private StandardScales() {}

    public static final Scale MAJOR = new Scale("Major",
        Degree.ROOT, Degree.NINTH, Degree.THIRD, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH);

    public static final Scale DORIAN = new Scale("Dorian",
        Degree.ROOT, Degree.NINTH, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH_FLAT);

    public static final Scale PHRYGIAN = new Scale("Phrygian",
        Degree.ROOT, Degree.NINTH_FLAT, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH_FLAT);

    public static final Scale LYDIAN = new Scale("Lydian",
        Degree.ROOT, Degree.NINTH, Degree.THIRD, Degree.ELEVENTH_SHARP,
        Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH);

    public static final Scale MIXOLYDIAN = new Scale("Mixolydian",
        Degree.ROOT, Degree.NINTH, Degree.THIRD, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH_FLAT);

    public static final Scale AEOLIAN = new Scale("Aeolian",
        Degree.ROOT, Degree.NINTH, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH_FLAT);

    public static final Scale LOCRIAN = new Scale("Locrian",
        Degree.ROOT, Degree.NINTH_FLAT, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH_FLAT, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH_FLAT);

    public static final Scale HARMONIC_MINOR = new Scale("Harmonic Minor",
        Degree.ROOT, Degree.NINTH, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH);

    public static final Scale MELODIC_MINOR = new Scale("Melodic Minor",
        Degree.ROOT, Degree.NINTH, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH);

    public static final Scale MAJOR_PENTATONIC = new Scale("Major Pentatonic",
        Degree.ROOT, Degree.NINTH, Degree.THIRD, Degree.FIFTH, Degree.SIXTH_OR_THIRTEENTH);

    public static final Scale MINOR_PENTATONIC = new Scale("Minor Pentatonic",
        Degree.ROOT, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH, Degree.FIFTH, Degree.SEVENTH_FLAT);

    public static final Scale BLUES = new Scale("Blues",
        Degree.ROOT, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH, Degree.FIFTH_FLAT,
        Degree.FIFTH, Degree.SEVENTH_FLAT);

    public static final Scale WHOLE_TONE = new Scale("Whole Tone",
        Degree.ROOT, Degree.NINTH, Degree.THIRD, Degree.ELEVENTH_SHARP,
        Degree.FIFTH_SHARP, Degree.SEVENTH_FLAT);

    public static final Scale DIMINISHED = new Scale("Diminished",
        Degree.ROOT, Degree.NINTH, Degree.THIRD_FLAT, Degree.FOURTH_OR_ELEVENTH,
        Degree.FIFTH_FLAT, Degree.FIFTH_SHARP, Degree.SIXTH_OR_THIRTEENTH, Degree.SEVENTH);
}
