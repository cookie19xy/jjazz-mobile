package org.jjazz.purecore.harmony;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.*;
import java.util.stream.Collectors;
import static org.jjazz.purecore.harmony.Degree.*;

/**
 * Represents a chord type like "m7", its aliases and its degrees.
 * Immutable. Extracted from JJazzLab, stripped of ChordTypeDatabase/SPI/Scale deps.
 */
public final class ChordType {

    /** Constant for "degree not present". */
    public static final int NOT_PRESENT = 9;

    public enum Family {
        MAJOR, SEVENTH, MINOR, DIMINISHED, SUS;

        @Override
        public String toString() {
            // Simple English names
            return switch (this) {
                case MAJOR -> "Major";
                case SEVENTH -> "Seventh";
                case MINOR -> "Minor";
                case DIMINISHED -> "Diminished";
                case SUS -> "Sus";
            };
        }
    }

    public enum DegreeIndex {
        ROOT,
        THIRD_OR_FOURTH,
        FIFTH,
        SIXTH_OR_SEVENTH,
        EXTENSION1,  // 9, 11, or 13
        EXTENSION2,  // 11 or 13
        EXTENSION3;  // 13

        public boolean isExtension() {
            return this == EXTENSION1 || this == EXTENSION2 || this == EXTENSION3;
        }
    }

    private final Family family;
    private final String base;
    private final String extension;
    private final List<Degree> degrees = new ArrayList<>();
    private final Chord chord = new Chord();
    private final String degreeString;
    private List<DegreeIndex> mostImportantDegrees; // lazy

    // Built-in chord type registry (simple lookup by name)
    private static final Map<String, ChordType> REGISTRY = new HashMap<>();

    /**
     * Build a ChordType from degree accidental values.
     * Use NOT_PRESENT if a degree is not present.
     */
    public ChordType(String b, String e, Family f,
                     int i9, int i3, int i11, int i5, int i13, int i7) {
        Objects.requireNonNull(b);
        Objects.requireNonNull(e);
        checkArgument(checkDegree(i9) && checkDegree(i3) && checkDegree(i11)
                   && checkDegree(i5) && checkDegree(i13) && checkDegree(i7));

        this.base = b;
        this.extension = e;
        this.family = f;

        // Build chord and degrees
        chord.add(new Note(0));
        degrees.add(ROOT);

        if (i3 != NOT_PRESENT) {
            chord.add(new Note(4 + i3));
            degrees.add(Degree.getDegree(Degree.Natural.THIRD, i3));
        }
        if (i11 == 0 && i3 == NOT_PRESENT) {
            chord.add(new Note(5));
            degrees.add(FOURTH_OR_ELEVENTH);
        }
        if (i5 != NOT_PRESENT) {
            chord.add(new Note(7 + i5));
            degrees.add(Degree.getDegree(Degree.Natural.FIFTH, i5));
        }
        if (i13 == 0 && i7 == NOT_PRESENT) {
            chord.add(new Note(9));
            degrees.add(SIXTH_OR_THIRTEENTH);
        }
        if (i7 != NOT_PRESENT) {
            chord.add(new Note(11 + i7));
            degrees.add(Degree.getDegree(Degree.Natural.SEVENTH, i7));
        }
        if (i9 != NOT_PRESENT) {
            chord.add(new Note(2 + i9));
            degrees.add(Degree.getDegree(Degree.Natural.NINTH, i9));
        }
        if (i11 != NOT_PRESENT && !(i11 == 0 && i3 == NOT_PRESENT)) {
            chord.add(new Note(5 + i11));
            degrees.add(Degree.getDegree(Degree.Natural.ELEVENTH, i11));
        }
        if (i13 != NOT_PRESENT && !(i13 == 0 && i7 == NOT_PRESENT)) {
            chord.add(new Note(9 + i13));
            degrees.add(Degree.getDegree(Degree.Natural.SIXTH, i13));
        }

        this.degreeString = degrees.stream()
            .map(Degree::toStringShort)
            .collect(Collectors.joining(" ", "[", "]"));

        // Register in built-in table
        REGISTRY.put(getName(), this);
    }

    // --- Simple built-in chord type lookup (replaces ChordTypeDatabase) ---

    /** Get a chord type by its name (e.g. "m7", "7b9"). */
    public static ChordType getByName(String name) {
        return REGISTRY.get(name);
    }

    /**
     * Find a chord type matching the given degrees (exact match).
     * Returns null if not found in the built-in registry.
     */
    public static ChordType getByDegrees(List<Degree> searchDegrees) {
        for (ChordType ct : REGISTRY.values()) {
            if (ct.degrees.equals(searchDegrees)) return ct;
        }
        return null;
    }

    /** Get all registered chord types. */
    public static Collection<ChordType> getAll() {
        return Collections.unmodifiableCollection(REGISTRY.values());
    }

    // --- Accessors ---

    public Family getFamily() { return family; }
    public String getBase() { return base; }
    public String getExtension() { return extension; }
    public String getName() { return base + extension; }
    public List<Degree> getDegrees() { return Collections.unmodifiableList(degrees); }
    public int getNbDegrees() { return degrees.size(); }
    public Chord getChord() { return chord.clone(); }

    public List<DegreeIndex> getExtensionDegreeIndexes() {
        ArrayList<DegreeIndex> res = new ArrayList<>();
        int start = DegreeIndex.EXTENSION1.ordinal();
        if (isSpecial2Chord()) {
            res.add(DegreeIndex.EXTENSION1);
        } else {
            for (int i = start; i < degrees.size(); i++)
                res.add(DegreeIndex.values()[i]);
        }
        return res;
    }

    public DegreeIndex getDegreeIndex(Degree d) {
        Objects.requireNonNull(d);
        int index = degrees.indexOf(d);
        if (index != -1 && isSpecial2Chord()) {
            index = switch (d) {
                case ROOT -> index;
                case FIFTH -> index + 1;
                case NINTH -> index + 2;
                default -> throw new IllegalStateException("d=" + d);
            };
        }
        return index != -1 ? DegreeIndex.values()[index] : null;
    }

    public Degree getDegree(DegreeIndex di) {
        int ordinal = di.ordinal();
        if (isSpecial2Chord()) {
            ordinal = switch (di) {
                case ROOT -> ordinal;
                case FIFTH -> ordinal - 1;
                case EXTENSION1 -> ordinal - 2;
                default -> 1000;
            };
        }
        return ordinal < degrees.size() ? degrees.get(ordinal) : null;
    }

    public Degree getDegree(int relPitch) {
        checkArgument(relPitch >= 0 && relPitch <= 11);
        for (Degree d : degrees) {
            if (d.getPitch() == relPitch) return d;
        }
        return null;
    }

    public Degree getDegree(Degree.Natural nd) {
        for (Degree d : degrees) {
            if (d.getNatural() == nd) return d;
        }
        return null;
    }

    public int getNbCommonDegrees(ChordType ct, boolean sixthMajorSeventhEqual) {
        int res;
        for (res = 0; res < Math.min(degrees.size(), ct.degrees.size()); res++) {
            var d = degrees.get(res);
            var dCt = ct.degrees.get(res);
            if (sixthMajorSeventhEqual ? !d.equalsSixthMajorSeventh(dCt) : d != dCt) {
                break;
            }
        }
        return res;
    }

    public Degree getDegreeMostProbable(int relPitch) {
        checkArgument(relPitch >= 0 && relPitch <= 11);
        Degree d = getDegree(relPitch);
        if (d == null) {
            d = switch (relPitch) {
                case 0 -> ROOT;
                case 1 -> NINTH_FLAT;
                case 2 -> NINTH;
                case 3 -> isMajor() ? NINTH_SHARP : THIRD_FLAT;
                case 4 -> THIRD;
                case 5 -> FOURTH_OR_ELEVENTH;
                case 6 -> ELEVENTH_SHARP;
                case 7 -> FIFTH;
                case 8 -> THIRTEENTH_FLAT;
                case 9 -> SIXTH_OR_THIRTEENTH;
                case 10 -> SEVENTH_FLAT;
                case 11 -> SEVENTH;
                default -> throw new IllegalArgumentException("relPitch=" + relPitch);
            };
        }
        return d;
    }

    public List<DegreeIndex> getMostImportantDegreeIndexes() {
        if (mostImportantDegrees == null) {
            List<DegreeIndex> dis = new ArrayList<>();
            if (!isSpecial2Chord()) dis.add(DegreeIndex.THIRD_OR_FOURTH);
            if (!getDegree(DegreeIndex.FIFTH).equals(FIFTH)) dis.add(DegreeIndex.FIFTH);
            if (getDegree(DegreeIndex.SIXTH_OR_SEVENTH) != null) dis.add(DegreeIndex.SIXTH_OR_SEVENTH);
            if (getDegree(DegreeIndex.EXTENSION1) != null) dis.add(DegreeIndex.EXTENSION1);
            if (base.contains("6")) {
                dis.add(DegreeIndex.ROOT);
                if (getDegree(DegreeIndex.FIFTH).equals(FIFTH)) dis.add(DegreeIndex.FIFTH);
            } else {
                if (getDegree(DegreeIndex.FIFTH).equals(FIFTH)) dis.add(DegreeIndex.FIFTH);
                dis.add(DegreeIndex.ROOT);
            }
            if (getDegree(DegreeIndex.EXTENSION2) != null) dis.add(DegreeIndex.EXTENSION2);
            if (getDegree(DegreeIndex.EXTENSION3) != null) dis.add(DegreeIndex.EXTENSION3);
            mostImportantDegrees = Collections.unmodifiableList(dis);
        }
        return mostImportantDegrees;
    }

    public Degree fitDegree(Degree d) {
        Degree destDegree = getDegree(d.getNatural());
        if (destDegree == null) {
            destDegree = getDegree(d.getPitch());
        } else if (extension.contains("6") && d.getNatural().equals(Degree.Natural.SEVENTH)) {
            destDegree = SIXTH_OR_THIRTEENTH;
        } else if (getDegree(Natural.SEVENTH) != null && d.getNatural().equals(Degree.Natural.SIXTH)) {
            destDegree = getDegree(Natural.SEVENTH);
        }
        return destDegree;
    }

    /**
     * NOTE: StandardScaleInstance parameter is accepted but unused in this pure core version.
     * It's kept for API compatibility with the original. Only basic fitting is performed.
     */
    public Degree fitDegreeAdvanced(Degree d, Object optScale) {
        Degree destDegree = fitDegree(d);

        if (destDegree == null) {
            // Simplified fallback (original used scale matching - skipped in pure core)
            switch (d) {
                case NINTH_FLAT, NINTH, NINTH_SHARP -> {
                    destDegree = NINTH;
                    if (getName().equals("m7b5")) destDegree = NINTH_FLAT;
                }
                case THIRD_FLAT, THIRD -> destDegree = FOURTH_OR_ELEVENTH;
                case FOURTH_OR_ELEVENTH -> {
                    if (family == Family.MINOR || family == Family.DIMINISHED) destDegree = FOURTH_OR_ELEVENTH;
                    else if (getDegree(6) != null) destDegree = getDegree(6);
                    else if (getDegree(Degree.Natural.NINTH) != null && getDegree(Degree.Natural.NINTH).getAccidental() != 0)
                        destDegree = ELEVENTH_SHARP;
                    else destDegree = FOURTH_OR_ELEVENTH;
                }
                case ELEVENTH_SHARP -> {
                    if (getDegree(5) != null) destDegree = FOURTH_OR_ELEVENTH;
                    else destDegree = getDegree(Degree.Natural.FIFTH);
                }
                case THIRTEENTH_FLAT -> destDegree = getDegree(Degree.Natural.FIFTH);
                case SIXTH_OR_THIRTEENTH -> {
                    if (getName().equals("m7b5") || getName().equals("m9b5")) destDegree = THIRTEENTH_FLAT;
                    else if (getDegree(8) != null) destDegree = getDegree(8);
                    else destDegree = SIXTH_OR_THIRTEENTH;
                }
                case SEVENTH_FLAT -> {
                    destDegree = SEVENTH_FLAT;
                    if (family == Family.MAJOR && getDegree(9) != null) destDegree = SEVENTH;
                    else if (getName().equals("dim7")) destDegree = SIXTH_OR_THIRTEENTH;
                }
                case SEVENTH -> {
                    destDegree = SEVENTH;
                    if (family == Family.SUS) destDegree = SEVENTH_FLAT;
                    else if (family == Family.MINOR && getDegree(9) == null) destDegree = SEVENTH_FLAT;
                    else if (family == Family.DIMINISHED)
                        destDegree = getDegree(9) != null ? SIXTH_OR_THIRTEENTH : SEVENTH_FLAT;
                }
                default -> throw new IllegalStateException("d=" + d + " this=" + this);
            }
        }
        return destDegree;
    }

    public int getPitch(Degree.Natural nd, int rootPitch) {
        Degree d = getDegree(nd);
        return d != null ? rootPitch + d.getPitch() : -1;
    }

    /** Get simplified version keeping only nbMaxDegrees degrees. */
    public ChordType getSimplified(int nbMaxDegrees) {
        checkArgument(nbMaxDegrees >= 3);
        if (degrees.size() <= nbMaxDegrees) return this;
        var resDegrees = getDegrees().stream().limit(nbMaxDegrees).toList();
        ChordType res = getByDegrees(resDegrees);
        return res != null ? res : this;
    }

    // --- Boolean checks ---

    public boolean isSpecial2Chord() { return getName().equals("2"); }
    public boolean isMinor() { return THIRD_FLAT.equals(getDegree(Natural.THIRD)); }
    public boolean isMajor() { return THIRD.equals(getDegree(Natural.THIRD)); }
    public boolean isSeventhMinor() { return SEVENTH_FLAT.equals(getDegree(Natural.SEVENTH)); }
    public boolean isSeventhMajor() { return SEVENTH.equals(getDegree(Natural.SEVENTH)); }
    public boolean isSeventh() { return getDegree(Natural.SEVENTH) != null; }
    public boolean isFifthNatural() { return FIFTH.equals(getDegree(Natural.FIFTH)); }
    public boolean isFifthSharp() { return FIFTH_SHARP.equals(getDegree(Natural.FIFTH)); }
    public boolean isFifthFlat() { return FIFTH_FLAT.equals(getDegree(Natural.FIFTH)); }
    public boolean isEleventh() { return getDegree(Natural.ELEVENTH) != null; }
    public boolean isEleventhNatural() { return !isSus() && FOURTH_OR_ELEVENTH.equals(getDegree(Natural.ELEVENTH)); }
    public boolean isEleventhSharp() { return ELEVENTH_SHARP.equals(getDegree(Natural.ELEVENTH)); }
    public boolean isSus() { return getFamily() == Family.SUS; }
    public boolean isSixth() { return getDegree(Natural.SEVENTH) == null && SIXTH_OR_THIRTEENTH.equals(getDegree(Natural.SIXTH)); }
    public boolean isThirteenth() { return getDegree(Natural.SEVENTH) != null && SIXTH_OR_THIRTEENTH.equals(getDegree(Natural.SIXTH)); }
    public boolean isNinth() { return getDegree(Natural.NINTH) != null; }
    public boolean isNinthNatural() { return NINTH.equals(getDegree(Natural.NINTH)); }
    public boolean isNinthSharp() { return NINTH_SHARP.equals(getDegree(Natural.NINTH)); }
    public boolean isNinthFlat() { return NINTH_FLAT.equals(getDegree(Natural.NINTH)); }

    @Override
    public String toString() { return getName(); }

    public String toDegreeString() { return degreeString; }

    public boolean equalsSixthMajorSeventh(Object o) {
        if (!(o instanceof ChordType ct)) return false;
        var ctDegrees = ct.getDegrees();
        if (ctDegrees.size() != degrees.size()) return false;
        for (int i = 0; i < degrees.size(); i++) {
            if (!ctDegrees.get(i).equalsSixthMajorSeventh(degrees.get(i))) return false;
        }
        return true;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof ChordType ct)) return false;
        return degrees.equals(ct.degrees);
    }

    @Override
    public int hashCode() { return degrees.hashCode(); }

    private static boolean checkDegree(int d) {
        return d == -1 || d == 0 || d == 1 || d == NOT_PRESENT;
    }
}
