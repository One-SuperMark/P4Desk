use std::{
    collections::{BTreeSet, HashMap},
    env, fs,
    path::PathBuf,
};
use tiny_flutter::graphics::fontpack::{encode_fontpack, FontPack, PackGlyph};

fn snapshot_text(path: &str) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(path).map_err(|_| "snapshot read failed")?)
            .map_err(|_| "invalid snapshot")?;
    let value = value.get("state").unwrap_or(&value);
    let mut text = String::new();
    for note in value
        .get("notes")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        for key in ["title", "body"] {
            if let Some(s) = note.get(key).and_then(|v| v.as_str()) {
                text.push_str(s);
                text.push('\n');
            }
        }
    }
    for button in value
        .get("buttons")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        if let Some(s) = button.get("label").and_then(|v| v.as_str()) {
            text.push_str(s);
            text.push('\n');
        }
    }
    Ok(text)
}
fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args
        .first()
        .filter(|v| !v.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("bake");
    let mut options = HashMap::new();
    let mut i = usize::from(args.first().is_some_and(|v| !v.starts_with('-')));
    while i < args.len() {
        if args[i] == "--help" {
            println!("p4desk-fontpack [bake] --font TTF (--text-file UTF8 | --snapshot JSON) --output P4F --sizes 18,22,28,36 [--missing-glyphs skip] [--report JSON]\np4desk-fontpack validate --input P4F [--snapshot JSON]");
            return Ok(());
        }
        if i + 1 >= args.len() || !args[i].starts_with("--") {
            return Err("invalid CLI arguments".into());
        }
        options.insert(args[i].clone(), args[i + 1].clone());
        i += 2;
    }
    if command == "validate" {
        let path = options.get("--input").ok_or("--input is required")?;
        let pack = FontPack::from_file(path).map_err(str::to_string)?;
        if let Some(path) = options.get("--snapshot") {
            if !pack.covers(&snapshot_text(path)?, &[18, 22, 28, 36]) {
                return Err("font coverage incomplete".into());
            }
        }
        println!(
            "{}",
            serde_json::json!({"valid":true,"bytes":pack.byte_length(),"glyph_count":pack.glyph_count()})
        );
        return Ok(());
    }
    if command != "bake" {
        return Err("unknown command".into());
    }
    let font_path = options
        .get("--font")
        .map(PathBuf::from)
        .or_else(|| env::var_os("P4DESK_FONT_PATH").map(PathBuf::from))
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets/fonts/HarmonyOS_Sans_SC_Regular.ttf")
        });
    let text = if let Some(p) = options.get("--snapshot") {
        snapshot_text(p)?
    } else {
        fs::read_to_string(
            options
                .get("--text-file")
                .ok_or("--text-file or --snapshot is required")?,
        )
        .map_err(|_| "text input read failed")?
    };
    let sizes: Vec<u16> = options
        .get("--sizes")
        .map(String::as_str)
        .unwrap_or("18,22,28,36")
        .split(',')
        .map(|s| s.parse().map_err(|_| "invalid font size"))
        .collect::<Result<_, _>>()?;
    if sizes.is_empty() || sizes.iter().any(|s| !(8..=128).contains(s)) {
        return Err("invalid font sizes".into());
    }
    let mut characters: BTreeSet<char> = text.chars().filter(|c| !c.is_control()).collect();
    characters.extend(' '..='~');
    characters.extend(['□', '×', '÷', '±', '√']);
    let bytes = fs::read(font_path).map_err(|_| "TTF read failed")?;
    let font = fontdue::Font::from_bytes(
        bytes,
        fontdue::FontSettings {
            load_substitutions: false,
            ..Default::default()
        },
    )
    .map_err(|_| "TTF parse failed")?;
    let skip_missing = match options.get("--missing-glyphs").map(String::as_str) {
        None | Some("error") => false,
        Some("skip") => true,
        _ => return Err("invalid missing glyph policy".into()),
    };
    let unsupported_count = characters.iter().filter(|c| !font.has_glyph(**c)).count();
    if unsupported_count > 0 && !skip_missing {
        return Err("source font lacks required glyphs".into());
    }
    if skip_missing {
        characters.retain(|c| font.has_glyph(*c));
    }
    let mut glyphs = Vec::new();
    for size in &sizes {
        for character in &characters {
            let (m, bitmap) = font.rasterize(*character, *size as f32);
            glyphs.push(PackGlyph {
                character: *character,
                size: *size,
                width: m.width.try_into().map_err(|_| "glyph too wide")?,
                height: m.height.try_into().map_err(|_| "glyph too high")?,
                xmin: m.xmin.try_into().map_err(|_| "glyph bearing overflow")?,
                ymin: m.ymin.try_into().map_err(|_| "glyph bearing overflow")?,
                advance: m.advance_width,
                bitmap,
            });
        }
    }
    let bytes = encode_fontpack(glyphs).map_err(str::to_string)?;
    let output = PathBuf::from(options.get("--output").ok_or("--output is required")?);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|_| "output directory failed")?;
    }
    fs::write(&output, &bytes).map_err(|_| "font package write failed")?;
    let report = serde_json::json!({"valid":true,"bytes":bytes.len(),"glyph_count":characters.len()*sizes.len(),"sizes":sizes,"unsupported_count":unsupported_count});
    if let Some(path) = options.get("--report") {
        fs::write(path, serde_json::to_vec(&report).map_err(|_| "font report failed")?)
            .map_err(|_| "font report write failed")?;
    }
    println!("{}", report);
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{}", serde_json::json!({"valid":false,"error":error}));
        std::process::exit(1);
    }
}
