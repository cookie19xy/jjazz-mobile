package org.jjazz.purecore.phrase;

import static com.google.common.base.Preconditions.checkArgument;
import java.util.*;
import java.util.function.Consumer;
import java.util.function.Predicate;
import java.util.logging.Logger;
import org.jjazz.purecore.util.FloatRange;

/**
 * A sorted collection of NoteEvents kept ordered by start position.
 * Extracted from JJazzLab Phrase, stripped of Swing/NetBeans/UndoManager.
 */
public class Phrase implements SortedSet<NoteEvent> {

    private final TreeSet<NoteEvent> notes;
    private final int channel;
    private final boolean drums;

    public Phrase(int channel) {
        this(channel, false);
    }

    public Phrase(int channel, boolean drums) {
        this.channel = channel;
        this.drums = drums;
        this.notes = new TreeSet<>(NoteEvent::compareTo);
    }

    public int getChannel() { return channel; }
    public boolean isDrums() { return drums; }

    // === Collection / SortedSet delegation ===

    @Override public int size() { return notes.size(); }
    @Override public boolean isEmpty() { return notes.isEmpty(); }
    @Override public boolean contains(Object o) { return notes.contains(o); }
    @Override public Iterator<NoteEvent> iterator() { return notes.iterator(); }
    @Override public Object[] toArray() { return notes.toArray(); }
    @Override public <T> T[] toArray(T[] a) { return notes.toArray(a); }
    @Override public boolean add(NoteEvent ne) { return notes.add(ne); }
    @Override public boolean remove(Object o) { return notes.remove(o); }
    @Override public boolean containsAll(Collection<?> c) { return notes.containsAll(c); }
    @Override public boolean addAll(Collection<? extends NoteEvent> c) { return notes.addAll(c); }
    @Override public boolean retainAll(Collection<?> c) { return notes.retainAll(c); }
    @Override public boolean removeAll(Collection<?> c) { return notes.removeAll(c); }
    @Override public void clear() { notes.clear(); }
    @Override public Comparator<? super NoteEvent> comparator() { return notes.comparator(); }
    @Override public NoteEvent first() { return notes.first(); }
    @Override public NoteEvent last() { return notes.last(); }
    @Override public SortedSet<NoteEvent> headSet(NoteEvent to) { return notes.headSet(to); }
    @Override public SortedSet<NoteEvent> tailSet(NoteEvent from) { return notes.tailSet(from); }
    @Override public SortedSet<NoteEvent> subSet(NoteEvent from, NoteEvent to) { return notes.subSet(from, to); }

    // === Phrase-specific ===

    /** Replace old note with new note. */
    public void replace(NoteEvent oldNote, NoteEvent newNote) {
        if (notes.remove(oldNote)) notes.add(newNote);
        else throw new NoSuchElementException("oldNote=" + oldNote);
    }

    /** Replace multiple notes. */
    public void replaceAll(Map<NoteEvent, NoteEvent> mapOldNew) {
        for (var entry : mapOldNew.entrySet()) {
            notes.remove(entry.getKey());
            notes.add(entry.getValue());
        }
    }

    /** Process notes matching predicate. */
    public void processNotes(Predicate<NoteEvent> predicate, Consumer<NoteEvent> processor) {
        // Work on a copy since processing may modify
        List<NoteEvent> matching = new ArrayList<>();
        for (NoteEvent ne : notes) if (predicate.test(ne)) matching.add(ne);
        for (NoteEvent ne : matching) processor.accept(ne);
    }

    /** Get the beat range of all notes in this phrase. */
    public FloatRange getNotesBeatRange() {
        if (isEmpty()) return new FloatRange(0, 0);
        float min = Float.MAX_VALUE, max = 0;
        for (NoteEvent ne : notes) {
            min = Math.min(min, ne.getPositionInBeats());
            max = Math.max(max, ne.getPositionInBeats() + ne.getDurationInBeats());
        }
        return new FloatRange(min, max);
    }

    @Override
    public String toString() {
        return "Phrase[ch=" + channel + " notes=" + size() + "]";
    }
}
