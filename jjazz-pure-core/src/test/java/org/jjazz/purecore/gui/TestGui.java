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

public class TestGui {

    private JTextArea logArea;
    private JTextField inputField;
    private JButton goBtn;

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

        JPanel top = new JPanel(new BorderLayout(5, 0));
        top.add(new JLabel("和弦:"), BorderLayout.WEST);
        inputField = new JTextField("Dm7 G7 Cmaj7");
        inputField.setFont(new Font("Monospaced", Font.PLAIN, 16));
        inputField.addActionListener(e -> doRun());
        top.add(inputField, BorderLayout.CENTER);

        JPanel topBtns = new JPanel(new GridLayout(1, 2, 5, 0));
        goBtn = new JButton("生成");
        goBtn.addActionListener(e -> doRun());
        topBtns.add(goBtn);
        JButton clearBtn = new JButton("清空");
        clearBtn.addActionListener(e -> logArea.setText(""));
        topBtns.add(clearBtn);
        top.add(topBtns, BorderLayout.EAST);
        main.add(top, BorderLayout.NORTH);

        logArea = new JTextArea();
        logArea.setEditable(false);
        logArea.setFont(new Font("Monospaced", Font.PLAIN, 13));
        main.add(new JScrollPane(logArea), BorderLayout.CENTER);

        f.add(main);
        f.setVisible(true);
        inputField.requestFocus();

        log("=== JJazz Pure Core ===");
        log("输入和弦 → 回车 或 点 [生成]");
        log("");

        // Sanity check: does the button work at all?
        JOptionPane.showMessageDialog(f, "GUI 启动成功。\n输入和弦后点 [生成] 或按回车。", "就绪", JOptionPane.INFORMATION_MESSAGE);
    }

    void doRun() {
        goBtn.setText("处理中...");
        goBtn.setEnabled(false);
        // Run on background thread so GUI doesn't freeze
        new Thread(() -> {
            try {
                run();
            } catch (Throwable t) {
                String msg = t.toString();
                log("!!! 崩溃: " + msg);
                for (StackTraceElement ste : t.getStackTrace()) {
                    if (ste.getClassName().contains("purecore")) log("    at " + ste);
                }
                SwingUtilities.invokeLater(() ->
                    JOptionPane.showMessageDialog(null, "错误:\n" + msg, "崩溃", JOptionPane.ERROR_MESSAGE));
            } finally {
                SwingUtilities.invokeLater(() -> { goBtn.setText("生成"); goBtn.setEnabled(true); });
            }
        }).start();
    }

    void run() {
        String raw = inputField.getText().trim();
        log(">>> 开始处理: " + raw);

        // 1. Parse
        log("[1/4] 解析和弦...");
        java.util.List<ChordSymbol> chords = new ArrayList<>();
        for (String s : raw.split("[,，\\s]+")) {
            if (s.isEmpty()) continue;
            try {
                ChordSymbol cs = new ChordSymbol(s);
                chords.add(cs);
                log("  ✓ " + s + " → " + cs.getName() + " " + cs.getChordType().toDegreeString());
            } catch (ParseException ex) {
                log("  ✗ " + s + " — " + ex.getMessage());
                return;
            }
        }
        if (chords.isEmpty()) { log("  无有效和弦!"); return; }
        log("  解析完成: " + chords.size() + " 个和弦");

        // 2. Generate
        log("[2/4] 生成乐句...");
        Phrase phrase = new Phrase(0);
        int baseOctave = 36;
        for (int i = 0; i < chords.size(); i++) {
            int rp = chords.get(i).getRootNote().getRelativePitch();
            int root = baseOctave + rp;
            float start = i * 4f;
            phrase.add(new NoteEvent(root, 1.5f, 100, start));
            phrase.add(new NoteEvent(root + 7, 0.5f, 85, start + 2f));
        }
        log("  生成 " + phrase.size() + " 个音符");
        dump(phrase);

        // 3. Humanize
        log("[3/4] 人性化...");
        Humanizer h = new Humanizer(phrase, TimeSignature.FOUR_FOUR, 120);
        h.registerNotes(new ArrayList<>(phrase));
        h.setConfig(Humanizer.DEFAULT_CONFIG);
        h.humanize();
        log("  人性化完成");
        dump(phrase);

        // 4. Quantize
        log("[4/4] 量化...");
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
        log("  量化完成, " + reps.size() + " 音符调整");
        dump(phrase);

        log("\n========== 完成 ==========\n");
    }

    void dump(Phrase p) {
        int i = 0;
        for (NoteEvent ne : p) {
            log(String.format("    #%02d %-6s pos=%-6.3f dur=%-5.2f vel=%d",
                i++, ne.toPianoOctaveString(), ne.getPositionInBeats(),
                ne.getDurationInBeats(), ne.getVelocity()));
        }
    }

    void log(String s) {
        SwingUtilities.invokeLater(() -> {
            logArea.append(s + "\n");
            logArea.setCaretPosition(logArea.getDocument().getLength());
        });
    }
}
