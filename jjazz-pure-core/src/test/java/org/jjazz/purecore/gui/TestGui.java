package org.jjazz.purecore.gui;

import org.jjazz.purecore.harmony.*;
import org.jjazz.purecore.phrase.*;
import org.jjazz.purecore.humanizer.*;
import org.jjazz.purecore.quantizer.*;
import org.jjazz.purecore.quantizer.Quantizer.Quantization;

import javax.swing.*;
import java.awt.*;
import java.text.ParseException;
import java.util.*;

/**
 * 最简测试 GUI: 输入和弦 → 点生成 → 看到结果。
 */
public class TestGui {

    private JTextArea logArea;
    private JTextField inputField;

    public static void main(String[] args) {
        SwingUtilities.invokeLater(() -> new TestGui().build());
    }

    void build() {
        JFrame f = new JFrame("JJazz Pure Core Test");
        f.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        f.setSize(800, 600);
        f.setLocationRelativeTo(null);

        JPanel main = new JPanel(new BorderLayout(8, 8));
        main.setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        // === TOP: input ===
        JPanel top = new JPanel(new BorderLayout(5, 0));
        top.add(new JLabel("和弦 (逗号/空格分隔):"), BorderLayout.WEST);
        inputField = new JTextField("Dm7 G7 Cmaj7");
        inputField.setFont(new Font("Monospaced", Font.PLAIN, 16));
        inputField.addActionListener(e -> run());
        top.add(inputField, BorderLayout.CENTER);

        JPanel topBtns = new JPanel(new GridLayout(1, 2, 5, 0));
        JButton goBtn = new JButton("生成");
        goBtn.addActionListener(e -> run());
        topBtns.add(goBtn);
        JButton clearBtn = new JButton("清空");
        clearBtn.addActionListener(e -> logArea.setText(""));
        topBtns.add(clearBtn);
        top.add(topBtns, BorderLayout.EAST);

        main.add(top, BorderLayout.NORTH);

        // === CENTER: log ===
        logArea = new JTextArea();
        logArea.setEditable(false);
        logArea.setFont(new Font("Monospaced", Font.PLAIN, 13));
        JScrollPane sp = new JScrollPane(logArea);
        main.add(sp, BorderLayout.CENTER);

        f.add(main);
        f.setVisible(true);
        inputField.requestFocus();

        log("=== JJazz Pure Core ===");
        log("输入和弦 (如 Dm7 G7 Cmaj7) 然后回车或点 [生成]");
        log("");
    }

    void run() {
        String raw = inputField.getText().trim();
        if (raw.isEmpty()) return;

        // 1. Parse chords
        java.util.List<ChordSymbol> chords = new ArrayList<>();
        for (String s : raw.split("[,，\\s]+")) {
            if (s.isEmpty()) continue;
            try {
                chords.add(new ChordSymbol(s));
            } catch (ParseException ex) {
                log("✗ 无法解析: " + s + " — " + ex.getMessage());
                return;
            }
        }
        if (chords.isEmpty()) return;

        log("=== 输入: " + raw + " (" + chords.size() + " 个和弦) ===");
        for (ChordSymbol cs : chords) {
            log("  " + cs.getName() + "  " + cs.getChordType().toDegreeString()
                + "  notes=" + cs.getChord(48).toAbsoluteNoteString());
        }

        // 2. Generate phrase
        Phrase phrase = new Phrase(0);
        int baseOctave = 36;
        for (int i = 0; i < chords.size(); i++) {
            int rp = chords.get(i).getRootNote().getRelativePitch();
            int root = baseOctave + rp;
            float start = i * 4f;
            phrase.add(new NoteEvent(root, 1.5f, 100, start));
            phrase.add(new NoteEvent(root + 7, 0.5f, 85, start + 2f));
        }
        log("\n--- 生成乐句 (" + phrase.size() + " 音符) ---");
        dump(phrase);

        // 3. Humanize
        Humanizer h = new Humanizer(phrase, TimeSignature.FOUR_FOUR, 120);
        h.registerNotes(new ArrayList<>(phrase));
        h.setConfig(Humanizer.DEFAULT_CONFIG);
        h.humanize();
        log("\n--- 人性化后 ---");
        dump(phrase);

        // 4. Quantize
        Map<NoteEvent, NoteEvent> reps = new HashMap<>();
        for (NoteEvent ne : phrase) {
            Position p = Position.fromAbsoluteBeat(ne.getPositionInBeats(), TimeSignature.FOUR_FOUR);
            Position q = Quantizer.quantize(Quantization.BEAT, p, TimeSignature.FOUR_FOUR, 1f, 99);
            float nb = q.toAbsoluteBeat(TimeSignature.FOUR_FOUR);
            if (Math.abs(nb - ne.getPositionInBeats()) > 0.005f) {
                reps.put(ne, ne.setAll(-1, -1, -1, nb, null));
            }
        }
        phrase.replaceAll(reps);
        log("\n--- 量化后 (拍点对齐) ---");
        dump(phrase);
        log("\n========== 完成 ==========\n");
    }

    void dump(Phrase p) {
        int i = 0;
        for (NoteEvent ne : p) {
            log(String.format("  #%02d %-6s pos=%-6.3f dur=%-5.2f vel=%d",
                i++, ne.toPianoOctaveString(), ne.getPositionInBeats(),
                ne.getDurationInBeats(), ne.getVelocity()));
        }
    }

    void log(String s) {
        logArea.append(s + "\n");
        logArea.setCaretPosition(logArea.getDocument().getLength());
    }
}
