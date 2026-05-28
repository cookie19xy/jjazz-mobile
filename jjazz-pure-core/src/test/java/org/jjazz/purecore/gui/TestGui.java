package org.jjazz.purecore.gui;

import org.jjazz.purecore.harmony.*;
import org.jjazz.purecore.phrase.*;
import org.jjazz.purecore.humanizer.*;
import org.jjazz.purecore.quantizer.*;
import org.jjazz.purecore.quantizer.Quantizer.Quantization;

import javax.swing.*;
import javax.swing.border.*;
import java.awt.*;
import java.awt.event.*;
import java.text.ParseException;
import java.util.*;

/**
 * Minimal test GUI for Phase 1 Pure Core verification.
 * Zero external dependencies beyond JDK Swing.
 * 
 * Usage: Run this class's main() method.
 *   Left panel  → input chords, settings, action buttons
 *   Right panel → output log
 */
public class TestGui extends JFrame {

    // ---- Left panel widgets ----
    private JTextField chordInput;
    private JComboBox<String> timeSigCombo;
    private JComboBox<String> tempoCombo;
    private JComboBox<String> quantizeCombo;
    private JCheckBox humanizeCheck;
    private JTextArea outputArea;
    private JLabel statusLabel;

    // ---- State ----
    private Phrase currentPhrase;
    private final java.util.List<ChordSymbol> chordList = new ArrayList<>();

    // ---- Colors ----
    private static final Color BG_DARK  = new Color(30, 30, 30);
    private static final Color BG_PANEL = new Color(45, 45, 45);
    private static final Color FG_TEXT  = new Color(220, 220, 220);
    private static final Color ACCENT   = new Color(70, 130, 200);

    public TestGui() {
        super("JJazz Pure Core — Test GUI");
        setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        setSize(1000, 650);
        setLocationRelativeTo(null);
        setLayout(new BorderLayout());

        // Apply dark theme
        UIManager.put("Panel.background", BG_DARK);
        UIManager.put("OptionPane.background", BG_DARK);
        UIManager.put("OptionPane.messageForeground", FG_TEXT);

        add(buildLeftPanel(), BorderLayout.CENTER);
        add(buildRightPanel(), BorderLayout.EAST);
        add(buildStatusBar(), BorderLayout.SOUTH);

        log("JJazz Pure Core 测试 GUI 就绪");
        log("用法: 输入和弦 → 点 [解析和弦] → 重复添加多个 → 点 [生成乐句] → [人性化] → [量化]");
        log("示例和弦: Dm7, G7, Cmaj7, F7b9, Bbm7b5, Am7, D7#11\n");
    }

    // ============== LEFT PANEL ==============
    private JPanel buildLeftPanel() {
        JPanel panel = new JPanel();
        panel.setLayout(new BoxLayout(panel, BoxLayout.Y_AXIS));
        panel.setBorder(new EmptyBorder(12, 12, 12, 12));
        panel.setBackground(BG_PANEL);
        panel.setPreferredSize(new Dimension(480, 0));

        // --- Chord input ---
        addSection(panel, "和弦输入");
        JPanel chordRow = new JPanel(new BorderLayout(5, 0));
        chordRow.setBackground(BG_PANEL);
        chordInput = new JTextField("Dm7");
        chordInput.setFont(new Font("Monospaced", Font.PLAIN, 18));
        chordInput.setBackground(new Color(60, 60, 60));
        chordInput.setForeground(FG_TEXT);
        chordInput.setCaretColor(FG_TEXT);
        chordInput.addActionListener(e -> parseAndAddChord());
        chordRow.add(chordInput, BorderLayout.CENTER);

        JButton parseBtn = mkButton("解析和弦 →", ACCENT);
        parseBtn.addActionListener(e -> parseAndAddChord());
        chordRow.add(parseBtn, BorderLayout.EAST);
        panel.add(chordRow);
        panel.add(Box.createVerticalStrut(4));
        JButton clearChordsBtn = mkButton("清空和弦列表", new Color(180, 70, 70));
        clearChordsBtn.addActionListener(e -> { chordList.clear(); log("--- 和弦列表已清空 ---"); });
        panel.add(clearChordsBtn);
        panel.add(Box.createVerticalStrut(16));

        // --- Settings ---
        addSection(panel, "设置");
        JPanel settingsGrid = new JPanel(new GridLayout(2, 4, 10, 5));
        settingsGrid.setBackground(BG_PANEL);

        settingsGrid.add(mkLabel("拍号"));
        timeSigCombo = new JComboBox<>(new String[]{"4/4", "3/4", "6/8", "2/4", "5/4"});
        styleCombo(timeSigCombo);
        settingsGrid.add(timeSigCombo);

        settingsGrid.add(mkLabel("速度 (BPM)"));
        tempoCombo = new JComboBox<>(new String[]{"60", "90", "120", "140", "180", "220"});
        tempoCombo.setSelectedItem("120");
        styleCombo(tempoCombo);
        settingsGrid.add(tempoCombo);

        settingsGrid.add(mkLabel("量化"));
        quantizeCombo = new JComboBox<>(new String[]{"OFF", "BEAT", "HALF_BEAT", "1/4_BEAT", "1/3_BEAT"});
        quantizeCombo.setSelectedItem("BEAT");
        styleCombo(quantizeCombo);
        settingsGrid.add(quantizeCombo);

        settingsGrid.add(mkLabel("人性化"));
        humanizeCheck = new JCheckBox("开启 (20%随机)");
        humanizeCheck.setSelected(true);
        humanizeCheck.setBackground(BG_PANEL);
        humanizeCheck.setForeground(FG_TEXT);
        settingsGrid.add(humanizeCheck);

        panel.add(settingsGrid);
        panel.add(Box.createVerticalStrut(16));

        // --- Action buttons ---
        addSection(panel, "操作");
        JPanel btnGrid = new JPanel(new GridLayout(3, 2, 8, 8));
        btnGrid.setBackground(BG_PANEL);

        JButton genBtn = mkButton("🎵 生成乐句 (Phrase)", new Color(50, 160, 80));
        genBtn.addActionListener(e -> generatePhrase());
        btnGrid.add(genBtn);

        JButton humBtn = mkButton("🎲 应用人性化", new Color(180, 140, 40));
        humBtn.addActionListener(e -> applyHumanizer());
        btnGrid.add(humBtn);

        JButton quantBtn = mkButton("📏 应用量化", new Color(60, 120, 190));
        quantBtn.addActionListener(e -> applyQuantizer());
        btnGrid.add(quantBtn);

        JButton dumpBtn = mkButton("📋 输出当前乐句", new Color(130, 130, 130));
        dumpBtn.addActionListener(e -> dumpPhrase());
        btnGrid.add(dumpBtn);

        JButton chordInfoBtn = mkButton("🔍 显示和弦详情", ACCENT);
        chordInfoBtn.addActionListener(e -> showChordDetails());
        btnGrid.add(chordInfoBtn);

        JButton resetBtn = mkButton("🔄 全部重置", new Color(150, 60, 60));
        resetBtn.addActionListener(e -> resetAll());
        btnGrid.add(resetBtn);

        panel.add(btnGrid);

        return panel;
    }

    // ============== RIGHT PANEL (log) ==============
    private JPanel buildRightPanel() {
        JPanel panel = new JPanel(new BorderLayout());
        panel.setPreferredSize(new Dimension(480, 0));
        panel.setBorder(new EmptyBorder(12, 0, 12, 12));
        panel.setBackground(BG_DARK);

        JLabel title = new JLabel("  输出日志");
        title.setForeground(ACCENT);
        title.setFont(new Font("SansSerif", Font.BOLD, 14));
        panel.add(title, BorderLayout.NORTH);

        outputArea = new JTextArea();
        outputArea.setEditable(false);
        outputArea.setFont(new Font("Monospaced", Font.PLAIN, 13));
        outputArea.setBackground(new Color(25, 25, 25));
        outputArea.setForeground(new Color(200, 220, 200));
        outputArea.setCaretColor(FG_TEXT);
        outputArea.setMargin(new Insets(8, 8, 8, 8));

        JScrollPane scroll = new JScrollPane(outputArea);
        scroll.setBorder(BorderFactory.createLineBorder(new Color(60, 60, 60)));
        panel.add(scroll, BorderLayout.CENTER);

        return panel;
    }

    // ============== STATUS BAR ==============
    private JPanel buildStatusBar() {
        JPanel panel = new JPanel(new FlowLayout(FlowLayout.LEFT));
        panel.setBackground(new Color(40, 40, 40));
        panel.setBorder(new EmptyBorder(4, 12, 4, 12));
        statusLabel = new JLabel("和弦列表: 空");
        statusLabel.setForeground(new Color(160, 160, 160));
        panel.add(statusLabel);
        return panel;
    }

    // ============== ACTIONS ==============

    private void parseAndAddChord() {
        String input = chordInput.getText().trim();
        if (input.isEmpty()) return;
        try {
            ChordSymbol cs = new ChordSymbol(input);
            chordList.add(cs);
            log("✅ 解析: \"" + input + "\" → " + cs.getName()
                + " | 根音=" + cs.getRootNote().toRelativeNoteString()
                + " | 类型=" + cs.getChordType().getName()
                + " | 度数=" + cs.getChordType().toDegreeString());
            chordInput.setText("");
            chordInput.requestFocus();
            updateStatus();
        } catch (ParseException ex) {
            log("❌ 无法解析: \"" + input + "\" — " + ex.getMessage());
        }
    }

    private void generatePhrase() {
        if (chordList.isEmpty()) {
            log("❌ 请先添加至少一个和弦!");
            return;
        }
        TimeSignature ts = parseTimeSig();
        int rootOctave = 36; // C2

        currentPhrase = new Phrase(0);
        for (int bar = 0; bar < chordList.size(); bar++) {
            ChordSymbol cs = chordList.get(bar);
            int rootPitch = rootOctave + cs.getRootNote().getRelativePitch();
            float barStart = bar * ts.getNbNaturalBeats();

            // Root on beat 1
            currentPhrase.add(new NoteEvent(rootPitch, 1.5f, 100, barStart));
            // Fifth on beat 3 (if room)
            float beat3 = barStart + 2f;
            if (beat3 + 0.5f <= (bar + 1) * ts.getNbNaturalBeats()) {
                currentPhrase.add(new NoteEvent(rootPitch + 7, 0.5f, 85, beat3));
            }
        }
        log("\n🎵 已生成乐句: " + currentPhrase.size() + " 个音符, "
            + chordList.size() + " 个小节, 拍号=" + ts);
        dumpPhrase();
    }

    private void applyHumanizer() {
        if (currentPhrase == null || currentPhrase.isEmpty()) {
            log("❌ 请先生成乐句!");
            return;
        }
        int tempo = Integer.parseInt((String) tempoCombo.getSelectedItem());
        TimeSignature ts = parseTimeSig();

        Humanizer h = new Humanizer(currentPhrase, ts, tempo);
        h.registerNotes(new ArrayList<>(currentPhrase));
        h.setConfig(humanizeCheck.isSelected() ? Humanizer.DEFAULT_CONFIG : Humanizer.ZERO_CONFIG);
        h.humanize();

        log("\n🎲 人性化已应用 (tempo=" + tempo
            + ", config=" + (humanizeCheck.isSelected() ? "DEFAULT" : "ZERO") + ")");
        dumpPhrase();
    }

    private void applyQuantizer() {
        if (currentPhrase == null || currentPhrase.isEmpty()) {
            log("❌ 请先生成乐句!");
            return;
        }
        Quantization q = parseQuantization();
        if (q == Quantization.OFF) {
            log("📏 量化=OFF, 跳过");
            return;
        }
        TimeSignature ts = parseTimeSig();
        int maxBar = Math.max(1, chordList.size() + 1);

        Map<NoteEvent, NoteEvent> replacements = new HashMap<>();
        for (NoteEvent ne : currentPhrase) {
            Position pos = Position.fromAbsoluteBeat(ne.getPositionInBeats(), ts);
            Position qPos = Quantizer.quantize(q, pos, ts, 1.0f, maxBar);
            float newBeat = qPos.toAbsoluteBeat(ts);
            if (Math.abs(newBeat - ne.getPositionInBeats()) > 0.01f) {
                NoteEvent qNe = ne.setAll(-1, -1, -1, newBeat, null);
                replacements.put(ne, qNe);
            }
        }
        currentPhrase.replaceAll(replacements);

        log("\n📏 量化已完成 (" + q + ", " + replacements.size() + " 个音符被调整)");
        dumpPhrase();
    }

    private void dumpPhrase() {
        if (currentPhrase == null || currentPhrase.isEmpty()) {
            log("  (空乐句)");
            return;
        }
        log("  ┌─ 当前乐句 (" + currentPhrase.size() + " notes) ─────────────────");
        int i = 0;
        for (NoteEvent ne : currentPhrase) {
            log(String.format("  │ #%02d  %-5s  pos=%-6.3f  dur=%-5.2f  vel=%-3d",
                i++, ne.toPianoOctaveString(), ne.getPositionInBeats(),
                ne.getDurationInBeats(), ne.getVelocity()));
        }
        log("  └──────────────────────────────────────────────");
    }

    private void showChordDetails() {
        if (chordList.isEmpty()) {
            log("❌ 请先添加至少一个和弦!");
            return;
        }
        log("\n🔍 === 和弦详情 ===");
        for (int i = 0; i < chordList.size(); i++) {
            ChordSymbol cs = chordList.get(i);
            ChordType ct = cs.getChordType();
            Chord chord = cs.getChord(48); // C3 area
            log(String.format("  [%d] %-10s 根音=%-3s 低音=%-3s 类别=%-10s 度数=%s",
                i, cs.getName(),
                cs.getRootNote().toPianoOctaveString(),
                cs.getBassNote().toPianoOctaveString(),
                ct.getFamily(),
                ct.toDegreeString()));
            log("       音符=" + chord.toAbsoluteNoteString());
            log("       属性: major=" + ct.isMajor() + " minor=" + ct.isMinor()
                + " seventh=" + ct.isSeventh() + " dim=" + (ct.getFamily() == ChordType.Family.DIMINISHED));
        }
    }

    private void resetAll() {
        chordList.clear();
        currentPhrase = null;
        chordInput.setText("Dm7");
        chordInput.requestFocus();
        outputArea.setText("");
        log("JJazz Pure Core 测试 GUI — 已重置\n");
        updateStatus();
    }

    // ============== HELPERS ==============

    private TimeSignature parseTimeSig() {
        return switch ((String) timeSigCombo.getSelectedItem()) {
            case "3/4" -> TimeSignature.THREE_FOUR;
            case "6/8" -> TimeSignature.SIX_EIGHT;
            case "2/4" -> TimeSignature.TWO_FOUR;
            case "5/4" -> TimeSignature.FIVE_FOUR;
            default -> TimeSignature.FOUR_FOUR;
        };
    }

    private Quantization parseQuantization() {
        return switch ((String) quantizeCombo.getSelectedItem()) {
            case "BEAT" -> Quantization.BEAT;
            case "HALF_BEAT" -> Quantization.HALF_BEAT;
            case "1/4_BEAT" -> Quantization.ONE_QUARTER_BEAT;
            case "1/3_BEAT" -> Quantization.ONE_THIRD_BEAT;
            default -> Quantization.OFF;
        };
    }

    private void log(String msg) {
        outputArea.append(msg + "\n");
        outputArea.setCaretPosition(outputArea.getDocument().getLength());
    }

    private void updateStatus() {
        statusLabel.setText("和弦列表: " + chordList.size() + " 个 | "
            + (chordList.isEmpty() ? "空" : chordList.stream().map(ChordSymbol::getName)
                .reduce((a, b) -> a + ", " + b).orElse("")));
    }

    private void addSection(JPanel parent, String title) {
        JLabel label = new JLabel(title);
        label.setForeground(ACCENT);
        label.setFont(new Font("SansSerif", Font.BOLD, 13));
        label.setAlignmentX(Component.LEFT_ALIGNMENT);
        parent.add(label);
        parent.add(Box.createVerticalStrut(4));
    }

    private JLabel mkLabel(String text) {
        JLabel l = new JLabel(text, SwingConstants.RIGHT);
        l.setForeground(new Color(180, 180, 180));
        l.setFont(new Font("SansSerif", Font.PLAIN, 12));
        return l;
    }

    private JButton mkButton(String text, Color bg) {
        JButton b = new JButton(text);
        b.setBackground(bg);
        b.setForeground(Color.WHITE);
        b.setFont(new Font("SansSerif", Font.BOLD, 12));
        b.setFocusPainted(false);
        b.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(bg.darker(), 1),
            BorderFactory.createEmptyBorder(6, 14, 6, 14)));
        return b;
    }

    private void styleCombo(JComboBox<?> cb) {
        cb.setBackground(new Color(60, 60, 60));
        cb.setForeground(FG_TEXT);
        cb.setFont(new Font("SansSerif", Font.PLAIN, 12));
    }

    // ============== MAIN ==============
    public static void main(String[] args) {
        SwingUtilities.invokeLater(() -> new TestGui().setVisible(true));
    }
}
