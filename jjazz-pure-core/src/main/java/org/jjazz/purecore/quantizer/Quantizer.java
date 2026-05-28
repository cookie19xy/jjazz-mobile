package org.jjazz.purecore.quantizer;

import static com.google.common.base.Preconditions.checkArgument;
import org.jjazz.purecore.harmony.Position;
import org.jjazz.purecore.harmony.TimeSignature;

/**
 * Quantize note positions to the nearest grid point.
 * Extracted from JJazzLab Quantizer.
 */
public class Quantizer {

    public enum Quantization {
        OFF,
        BEAT(new float[]{0f}),
        HALF_BEAT(new float[]{0f, 0.5f}),
        ONE_THIRD_BEAT(new float[]{0f, 0.33333f, 0.66667f}),
        ONE_QUARTER_BEAT(new float[]{0f, 0.25f, 0.5f, 0.75f}),
        ONE_SIXTH_BEAT(new float[]{0f, 0.16667f, 0.33333f, 0.5f, 0.66667f, 0.83333f}),
        HALF_BAR;

        private final float[] beats;

        Quantization() { this.beats = null; }
        Quantization(float[] beats) { this.beats = beats; }

        public float[] getBeats() { return beats; }
    }

    private static final float ROUND_WINDOW = 0.01f;

    public static Position quantize(Quantization q, Position pos, TimeSignature ts, float strength, int maxBarIndex) {
        if (q == Quantization.OFF) return new Position(pos);
        if (q == Quantization.HALF_BAR) return quantizeHalfBar(pos, ts, maxBarIndex);
        return quantizeImpl(pos, ts, maxBarIndex, strength, q.getBeats());
    }

    public static float quantizeBeat(Quantization q, float beatPos) {
        if (q == Quantization.OFF || q == Quantization.HALF_BAR || q.getBeats() == null) return beatPos;
        return quantizeBeatImpl(beatPos, q.getBeats());
    }

    private static Position quantizeHalfBar(Position pos, TimeSignature ts, int maxBarIndex) {
        int bar = pos.getBar();
        float beat = pos.getBeat();
        float halfBeat = ts.getHalfBarBeat(false);
        if (beat < halfBeat / 2) beat = 0;
        else if (beat < 3 * halfBeat / 2) beat = halfBeat;
        else if (bar < maxBarIndex) { bar++; beat = 0; }
        else beat = halfBeat;
        return new Position(bar, beat);
    }

    private static Position quantizeImpl(Position pos, TimeSignature ts, int maxBarIndex, float strength, float[] qPoints) {
        float beatInt = (float)Math.floor(pos.getBeat());
        float frac = pos.getBeatFractionalPart();
        int bar = pos.getBar();

        if (qPoints.length == 1) {
            // Only one quantization point (e.g. BEAT = {0})
            float target = qPoints[0];
            float diff = Math.abs(frac - target);
            if (diff <= 0.5f) {
                frac = target;
            } else if (bar + 1 <= maxBarIndex) {
                bar++;
                frac = 0;
            }
        } else {
            for (int i = 0; i < qPoints.length - 1; i++) {
                if (frac == qPoints[i] || frac == qPoints[i + 1]) break;
                if (frac < qPoints[i + 1]) {
                    float lower = qPoints[i], upper = qPoints[i + 1];
                    float step = (upper - lower) / 2 * strength;
                    if (frac - lower < upper - frac) {
                        frac -= step;
                        if (frac - lower <= ROUND_WINDOW) frac = lower;
                    } else {
                        frac += step;
                        if (upper - frac <= ROUND_WINDOW) frac = upper;
                    }
                    break;
                }
            }
        }

        float resultBeat = beatInt + frac;
        if (ts.checkBeat(resultBeat)) return new Position(bar, resultBeat);
        else if (bar + 1 <= maxBarIndex) return new Position(bar + 1);
        else return new Position(bar, beatInt + qPoints[qPoints.length - 1]);
    }

    private static float quantizeBeatImpl(float beatPos, float[] qPoints) {
        float beatInt = (float)Math.floor(beatPos);
        float frac = beatPos - beatInt;
        for (int i = 0; i < qPoints.length - 1; i++) {
            if (frac == qPoints[i] || frac == qPoints[i + 1]) return beatPos;
            if (frac < qPoints[i + 1]) {
                float lower = qPoints[i], upper = qPoints[i + 1];
                return beatInt + (frac < (lower + upper) / 2 ? lower : upper);
            }
        }
        return beatPos;
    }
}
