use std::env;
use jjazz_engine::harmony::ChordSymbol;
use jjazz_engine::midi_tools::WalkingBassGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: walking-bass <和弦...>");
        eprintln!("示例: walking-bass C F G C");
        return;
    }
    let chords: Vec<ChordSymbol> = args[1..].iter()
        .map(|s| ChordSymbol::parse(s).expect("invalid chord"))
        .collect();

    let gen = WalkingBassGenerator::default();
    let phrase = gen.build(&chords, 4.0, 120);

    for ne in &phrase.notes {
        println!("beat {:5.2}  pitch {:3}  vel {:3}  dur {:.2}",
            ne.position, ne.pitch, ne.velocity, ne.duration);
    }
    println!("Total: {} notes", phrase.len());
}
