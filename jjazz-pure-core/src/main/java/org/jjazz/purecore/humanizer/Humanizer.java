package org.jjazz.purecore.humanizer;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.*;
import org.jjazz.purecore.phrase.NoteEvent;
import org.jjazz.purecore.phrase.Phrase;
import org.jjazz.purecore.phrase.Phrases;
import org.jjazz.purecore.harmony.TimeSignature;
import org.jjazz.purecore.util.FloatRange;

/**
 * Humanize notes by adding random deviations to timing and velocity.
 */
public class Humanizer {

    public record Config(float timingRandomness, float timingBias, float velocityRandomness) {
        public Config {
            checkArgument(timingRandomness >= 0 && timingRandomness <= 1);
            checkArgument(timingBias >= -0.5f && timingBias <= 0.5f);
            checkArgument(velocityRandomness >= 0 && velocityRandomness <= 1);
        }
        public Config() { this(0, 0, 0); }
    }

    public static final Config DEFAULT_CONFIG = new Config(0.2f, 0f, 0.2f);
    public static final Config ZERO_CONFIG = new Config(0f, 0f, 0f);

    private static final float MAX_TIMING_DEVIATION = 0.2f;
    private static final float MAX_TIMING_BIAS_DEVIATION = 0.2f;
    private static final int MAX_VELOCITY_DEVIATION = 30;

    private final Phrase sourcePhrase;
    private final FloatRange allowedBeatRange;
    private final double maxTimingDeviation;
    private final Set<NoteEvent> registeredNotes = new HashSet<>();
    private final Map<NoteEvent, NoteEvent> mapNoteOrig = new HashMap<>();
    private final Map<NoteEvent, float[]> mapNoteFactors = new HashMap<>();
    private Config config = DEFAULT_CONFIG;

    public Humanizer(Phrase phrase, TimeSignature ts, int tempo) {
        this(phrase, new FloatRange(0, Float.MAX_VALUE), tempo);
    }

    public Humanizer(Phrase phrase, FloatRange allowedBeatRange, int tempo) {
        checkArgument(tempo >= 10 && tempo <= 400);
        this.sourcePhrase = phrase;
        this.allowedBeatRange = allowedBeatRange;
        double tempoImpact = Math.max(-0.1, -0.1 + (tempo - 50) * 0.001);
        this.maxTimingDeviation = MAX_TIMING_DEVIATION + tempoImpact;
    }

    public void registerNotes(Collection<NoteEvent> nes) {
        for (NoteEvent ne : nes) {
            if (!registeredNotes.contains(ne)) {
                registeredNotes.add(ne);
                mapNoteOrig.put(ne, ne.clone());
                computeRandomFactors(ne);
            }
        }
    }

    public void setConfig(Config newConfig) { this.config = newConfig; }
    public Config getConfig() { return config; }

    /** Apply humanization to registered notes via manual replace loop. */
    public void humanize() {
        Map<NoteEvent, NoteEvent> replacements = new HashMap<>();

        for (NoteEvent ne : new ArrayList<>(sourcePhrase)) {
            if (!registeredNotes.contains(ne)) continue;

            NoteEvent neOrig = mapNoteOrig.get(ne);
            float[] factors = mapNoteFactors.get(ne);

            float posShift = (float)(factors[0] * maxTimingDeviation * config.timingRandomness()
                + MAX_TIMING_BIAS_DEVIATION * config.timingBias());
            float newPos = Math.max(allowedBeatRange.from,
                Math.min(allowedBeatRange.to - 0.1f, neOrig.getPositionInBeats() + posShift));

            float newDur = neOrig.getDurationInBeats();
            if (newPos + newDur > allowedBeatRange.to)
                newDur = allowedBeatRange.to - 0.05f - newPos;

            int velShift = Math.round(factors[1] * MAX_VELOCITY_DEVIATION * config.velocityRandomness());
            int newVel = clamp(neOrig.getVelocity() + velShift);

            if (Float.compare(newDur, ne.getDurationInBeats()) == 0
                && newVel == ne.getVelocity()
                && Float.compare(newPos, ne.getPositionInBeats()) == 0) continue;

            NoteEvent newNe = ne.setAll(-1, newDur, newVel, newPos, null);
            replacements.put(ne, newNe);
            registeredNotes.remove(ne);
            registeredNotes.add(newNe);
            mapNoteOrig.remove(ne);
            mapNoteOrig.put(newNe, neOrig);
            float[] f = mapNoteFactors.remove(ne);
            mapNoteFactors.put(newNe, f);
        }

        sourcePhrase.replaceAll(replacements);
        Phrases.fixOverlappedNotes(sourcePhrase);
    }

    private void computeRandomFactors(NoteEvent ne) {
        Random r = new Random();
        float tf = (float)(r.nextGaussian() * 0.33);
        float vf = (float)(r.nextGaussian() * 0.33);
        mapNoteFactors.put(ne, new float[]{
            Math.max(-1, Math.min(1, tf)),
            Math.max(-1, Math.min(1, vf))
        });
    }

    private static int clamp(int v) { return Math.max(0, Math.min(127, v)); }
}
