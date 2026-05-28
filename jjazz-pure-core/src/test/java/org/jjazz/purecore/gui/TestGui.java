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
 * 测试 GUI — 纯 Swing，零额外依赖。
 * 双击 jjazz-pure-core/run-gui.bat 启动，
 * 或 java -cp ... org.jjazz.purecore.gui.TestGui
 */
public class TestGui {

    private JTextArea logArea;
    private JTextField inputField;
    private JLabel statusLabel;
    private java.util.List<ChordSymbol> chords = new ArrayList<>();
    private Phrase phrase;

    public static void main(String[] args) {
        SwingUtilities.invokeLater(() -> new TestGui().build());
    }

    void build() {
        JFrame f = new JFrame("JJazz Pure Core Test");
        f.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        f.setSize(900, 600);
        f.setLocationRelativeTo(null);

        JPanel main = new JPanel(new BorderLayout(8, 8));
        main.setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        JPanel top = new JPanel(new BorderLayout(5, 0));
        top.add(new JLabel("和弦:"), BorderLayout.WEST);
        inputField = new JTextField("Dm7");
        inputField.setFont(new Font("Monospaced", Font.PLAIN, 16));
        inputField.addActionListener(e -> addChord());
        top.add(inputField, BorderLayout.CENTER);

        JButton addBtn = new JButton("添加和弦");
        addBtn.addActionListener(e -> addChord());
        top.add(addBtn, BorderLayout.EAST);
        main.add(top, BorderLayout.NORTH);

        JPanel btnBar = new JPanel(new FlowLayout(FlowLayout.LEFT, 5, 5));
        JButton genBtn = new JButton("生成乐句");
        genBtn.addActionListener(e -> generate());
        JButton humBtn = new JButton("人性化");
        humBtn.addActionListener(e -> humanize());
        JButton quantBtn = new JButton("量化");
        quantBtn.addActionListener(e -> quantize());
        JButton dumpBtn = new JButton("显示音符");
        dumpBtn.addActionListener(e -> dump());
        JButton infoBtn = new JButton("和弦详情");
        infoBtn.addActionListener(e -> showInfo());
        JButton clearBtn = new JButton("清空");
        clearBtn.addActionListener(e -> { chords.clear(); phrase = null; log("已清空"); updateStatus(); });

        btnBar.add(genBtn); btnBar.add(humBtn); btnBar.add(quantBtn);
        btnBar.add(dumpBtn); btnBar.add(infoBtn); btnBar.add(clearBtn);

        JPanel center = new JPanel(new BorderLayout(0, 5));
        center.add(btnBar, BorderLayout.NORTH);
        logArea = new JTextArea();
        logArea.setEditable(false);
        logArea.setFont(new Font("Monospaced", Font.PLAIN, 13));
        JScrollPane sp = new JScrollPane(logArea);
        sp.setPreferredSize(new Dimension(860, 400));
        center.add(sp, BorderLayout.CENTER);
        main.add(center, BorderLayout.CENTER);

        statusLabel = new JLabel("就绪。输入和弦如 Dm7, G7, Cmaj7 然后点 [生成乐句]");
        main.add(statusLabel, BorderLayout.SOUTH);

        f.add(main);
        f.setVisible(true);
        inputField.requestFocus();

        log("=== JJazz Pure Core 测试 GUI ===");
        log("步骤: 1) 输入和弦 → [添加和弦] 2) 重复 3) [生成] 4) [人性化] 5) [量化]");
        log("");
    }

    void addChord() {
        String s = inputField.getText().trim();
        if (s.isEmpty()) return;
        try {
            ChordSymbol cs = new ChordSymbol(s);
            chords.add(cs);
            log("OK: " + s + " -> " + cs.getName()
                + "  root=" + cs.getRootNote().toRelativeNoteString()
                + "  type=" + cs.getChordType().toDegreeString());
            inputField.setText("");
            inputField.requestFocus();
            updateStatus();
        } catch (ParseException ex) {
            log("FAIL: " + s + " — " + ex.getMessage());
            JOptionPane.showMessageDialog(null,
                "无法解析: " + s + "\n" + ex.getMessage(), "错误", JOptionPane.ERROR_MESSAGE);
        }
    }

    void generate() {
        if (chords.isEmpty()) { log("请先添加和弦!"); return; }
        phrase = new Phrase(0);
        int baseOctave = 36;
        for (int i = 0; i < chords.size(); i++) {
            int rp = chords.get(i).getRootNote().getRelativePitch();
            int root = baseOctave + rp;
            float start = i * 4f;
            phrase.add(new NoteEvent(root, 1.5f, 100, start));
            phrase.add(new NoteEvent(root + 7, 0.5f, 85, start + 2f));
        }
        log("生成: " + phrase.size() + " 音符, " + chords.size() + " 小节");
        dump();
    }

    void humanize() {
        if (phrase == null || phrase.isEmpty()) { log("请先生成!"); return; }
        Humanizer h = new Humanizer(phrase, TimeSignature.FOUR_FOUR, 120);
        h.registerNotes(new ArrayList<>(phrase));
        h.setConfig(Humanizer.DEFAULT_CONFIG);
        h.humanize();
        log("人性化: 完成");
        dump();
    }

    void quantize() {
        if (phrase == null || phrase.isEmpty()) { log("请先生成!"); return; }
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
        log("量化: " + reps.size() + " 音符调整");
        dump();
    }

    void dump() {
        if (phrase == null || phrase.isEmpty()) { log("  (无音符)"); return; }
        int i = 0;
        for (NoteEvent ne : phrase) {
            log(String.format("  #%02d %-6s pos=%-6.3f dur=%-5.2f vel=%d",
                i++, ne.toPianoOctaveString(), ne.getPositionInBeats(),
                ne.getDurationInBeats(), ne.getVelocity()));
        }
    }

    void showInfo() {
        if (chords.isEmpty()) { log("请先添加和弦!"); return; }
        for (ChordSymbol cs : chords) {
            ChordType ct = cs.getChordType();
            log(cs.getName() + " | root=" + cs.getRootNote().toPianoOctaveString()
                + " | " + ct.getFamily() + " | " + ct.toDegreeString()
                + " | notes=" + cs.getChord(48).toAbsoluteNoteString());
        }
    }

    void log(String s) {
        logArea.append(s + "\n");
        logArea.setCaretPosition(logArea.getDocument().getLength());
    }

    void updateStatus() {
        StringBuilder sb = new StringBuilder("和弦: ");
        if (chords.isEmpty()) sb.append("(空)");
        else for (ChordSymbol cs : chords) sb.append(cs.getName()).append(" ");
        statusLabel.setText(sb.toString());
    }
}
