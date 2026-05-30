use crate::style::RetriggerRule;
use std::collections::HashMap;

/// Per-channel settings parsed from CASM data.
#[derive(Debug, Clone)]
pub struct CasmChannelSetting {
    pub channel: u8,
    pub ntr: u8,          // Note Transposition Rule: 0=ROOT_FIXED, 1=ROOT_TRANS, 2=GUITAR
    pub ntt: u8,          // Note Transposition Table index
    pub retrigger: u8,    // Retrigger rule (SFF2 only)
    pub note_low: u8,     // Lower pitch limit
    pub note_high: u8,    // Upper pitch limit
    pub chord_root_upper: u8, // Chord root limit
    pub bass_on: bool,    // Bass mode
}

impl CasmChannelSetting {
    pub fn to_retrigger_rule(&self) -> RetriggerRule {
        match self.retrigger {
            0 => RetriggerRule::Stop,
            1 => RetriggerRule::PitchShift,
            2 => RetriggerRule::PitchShiftToRoot,
            3 => RetriggerRule::Retrigger,
            4 => RetriggerRule::RetriggerToRoot,
            _ => RetriggerRule::Retrigger,
        }
    }
}

/// Parsed CASM data: style parts with channel settings.
#[derive(Debug, Clone)]
pub struct CasmData {
    pub sff_version: u8, // 1=SFF1, 2=SFF2
    /// section_name → (channel → setting)
    pub sections: HashMap<String, HashMap<u8, CasmChannelSetting>>,
}

/// Parse CASM binary data from a style file.
/// The CASM section follows the "MThd" + "MTrk" chunks in the file.
pub fn parse_casm(raw: &[u8]) -> Result<CasmData, String> {
    let mut pos = 0;

    // Skip MThd header
    if !expect(raw, &mut pos, b"MThd") {
        return Err("Not a valid MIDI file (no MThd)".into());
    }
    pos += 4; // header size (skip: 6 bytes)
    pos += 2; // format
    pos += 2; // nTracks
    let _ticks = read_u16(raw, &mut pos);

    // Skip MTrk
    if !expect(raw, &mut pos, b"MTrk") {
        return Err("No MTrk found".into());
    }
    let track_size = read_u32(raw, &mut pos) as usize;
    pos += track_size;

    // Find "CASM" marker (may be offset by ±1 byte)
    if pos + 4 > raw.len() { return Ok(empty_casm()); }
    
    let mut found = false;
    for offset in 0..3 {
        if pos + offset + 4 <= raw.len() 
            && &raw[pos + offset..pos + offset + 4] == b"CASM" {
            pos += offset + 4;
            found = true;
            break;
        }
    }
    if !found { return Ok(empty_casm()); }

    let _casm_size = read_u32(raw, &mut pos) as usize;
    let casm_end = pos + _casm_size.min(raw.len() - pos);

    let mut sections: HashMap<String, HashMap<u8, CasmChannelSetting>> = HashMap::new();
    let mut sff_version: u8 = 1;

    // Parse CSEG sections
    while pos + 8 <= casm_end {
        if pos + 4 > casm_end || &raw[pos..pos + 4] != b"CSEG" { break; }
        pos += 4;
        let cseg_size = read_u32(raw, &mut pos) as usize;
        let cseg_end = pos + cseg_size;

        // Parse Sdec
        if pos + 4 > cseg_end || &raw[pos..pos + 4] != b"Sdec" { break; }
        pos += 4;
        let sdec_size = read_u32(raw, &mut pos) as usize;
        let sdec_str = String::from_utf8_lossy(&raw[pos..pos + sdec_size]).to_string();
        pos += sdec_size;

        // Parse channel sections (Ctab/Ctb2/Cntt)
        while pos + 8 <= cseg_end {
            let tag = if pos + 4 <= cseg_end { &raw[pos..pos + 4] } else { break };
            
            match tag {
                b"Ctab" | b"Ctb2" | b"Cntt" => {
                    let is_sff2 = tag == b"Ctb2" || tag == b"Cntt";
                    if is_sff2 { sff_version = 2; }
                    pos += 4;
                    let sect_size = read_u32(raw, &mut pos) as usize;
                    let data = &raw[pos..(pos + sect_size).min(cseg_end)];
                    pos += sect_size;

                    let ch_settings = if tag == b"Cntt" {
                        parse_cntt(data)?
                    } else if is_sff2 {
                        parse_ctb2(data)?
                    } else {
                        parse_ctab(data)?
                    };

                    // Apply to all style parts in this Sdec section
                    for part_name in sdec_str.split(',') {
                        let name = part_name.trim().to_string();
                        if !name.is_empty() {
                            sections.entry(name)
                                .or_default()
                                .insert(ch_settings.channel, ch_settings.clone());
                        }
                    }
                }
                _ => break, // Unknown section, stop parsing
            }
        }
        pos = cseg_end;
    }

    Ok(CasmData { sff_version, sections })
}

fn parse_cntt(data: &[u8]) -> Result<CasmChannelSetting, String> {
    if data.is_empty() { return Err("Cntt data empty".into()); }
    Ok(CasmChannelSetting {
        channel: data[0] & 0x0F,
        ntr: 0,
        ntt: if data.len() > 1 { data[1] } else { 0 },
        retrigger: 0,
        note_low: 0,
        note_high: 127,
        chord_root_upper: if data.len() > 4 { data[4] } else { 0x7F },
        bass_on: data.len() > 2 && data[2] != 0,
    })
}

fn parse_ctab(data: &[u8]) -> Result<CasmChannelSetting, String> {
    if data.is_empty() { return Err("Ctab data empty".into()); }
    Ok(CasmChannelSetting {
        channel: data[0] & 0x0F,
        ntr: if data.len() > 1 { data[1] } else { 0 },
        ntt: if data.len() > 2 { data[2] } else { 0 },
        retrigger: 0,
        note_low: 0,
        note_high: 127,
        chord_root_upper: if data.len() > 4 { data[4] } else { 0x7F },
        bass_on: false,
    })
}

fn parse_ctb2(data: &[u8]) -> Result<CasmChannelSetting, String> {
    if data.is_empty() { return Err("Ctb2 data empty".into()); }
    Ok(CasmChannelSetting {
        channel: data[0] & 0x0F,
        ntr: if data.len() > 1 { data[1] } else { 0 },
        ntt: if data.len() > 2 { data[2] } else { 0 },
        retrigger: if data.len() > 7 { data[7] } else { 0 },
        note_low: if data.len() > 5 { data[5] } else { 0 },
        note_high: if data.len() > 6 { data[6] } else { 127 },
        chord_root_upper: if data.len() > 4 { data[4] } else { 0x7F },
        bass_on: false,
    })
}

fn empty_casm() -> CasmData {
    CasmData { sff_version: 0, sections: HashMap::new() }
}

fn expect(raw: &[u8], pos: &mut usize, expected: &[u8]) -> bool {
    if *pos + expected.len() > raw.len() { return false; }
    let matches = &raw[*pos..*pos + expected.len()] == expected;
    if matches { *pos += expected.len(); }
    matches
}

fn read_u16(raw: &[u8], pos: &mut usize) -> u16 {
    if *pos + 2 > raw.len() { *pos += 2; return 0; }
    let val = u16::from_be_bytes([raw[*pos], raw[*pos + 1]]);
    *pos += 2;
    val
}

fn read_u32(raw: &[u8], pos: &mut usize) -> u32 {
    if *pos + 4 > raw.len() { *pos += 4; return 0; }
    let val = u32::from_be_bytes([raw[*pos], raw[*pos + 1], raw[*pos + 2], raw[*pos + 3]]);
    *pos += 4;
    val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = parse_casm(b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x03\xc0MTrk\x00\x00\x00\x00");
        assert!(result.is_ok());
        let casm = result.unwrap();
        assert_eq!(casm.sff_version, 0);
        assert!(casm.sections.is_empty());
    }

    #[test]
    fn test_parse_ctab_raw() {
        // Channel 0, NTR=0, NTT=1, ct=0, fm_highkey=60
        let data = vec![0x00, 0x00, 0x01, 0x00, 0x3C];
        let ctab = parse_ctab(&data).unwrap();
        assert_eq!(ctab.channel, 0);
        assert_eq!(ctab.ntr, 0);
        assert_eq!(ctab.ntt, 1);
        assert_eq!(ctab.chord_root_upper, 0x3C);
    }

    #[test]
    fn test_parse_ctb2_raw() {
        // Channel 1, NTR=1, NTT=3, ct=0, fm=80, lo=36, hi=96, rtr=4
        let data = vec![0x01, 0x01, 0x03, 0x00, 0x50, 0x24, 0x60, 0x04];
        let ctb2 = parse_ctb2(&data).unwrap();
        assert_eq!(ctb2.channel, 1);
        assert_eq!(ctb2.ntr, 1);
        assert_eq!(ctb2.ntt, 3);
        assert_eq!(ctb2.retrigger, 4);
        assert_eq!(ctb2.note_low, 0x24);
        assert_eq!(ctb2.note_high, 0x60);
        assert_eq!(ctb2.to_retrigger_rule(), RetriggerRule::RetriggerToRoot);
    }

    #[test]
    fn test_parse_real_style_casm() {
        use std::path::Path;
        let path = "styles/Swing&Jazz/JazzClub.S120.prs";
        if !Path::new(path).exists() { return; }
        let raw = std::fs::read(path).unwrap();
        let result = parse_casm(&raw);
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        let casm = result.unwrap();
        assert!(!casm.sections.is_empty(), "Expected some CASM sections");
        println!("SFF version: {}, sections: {:?}", casm.sff_version, casm.sections.keys().collect::<Vec<_>>());
    }
}
