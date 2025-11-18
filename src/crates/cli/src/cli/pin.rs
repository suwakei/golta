use gomod_rs::GoMod;
use semver;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs::{read_to_string, write};

#[derive(Serialize, Deserialize)]
struct PinFile {
    go: String,
}

pub fn run(tool: String, update_go_mod: bool) {
    if let Err(e) = pin_go_version(&tool, update_go_mod) {
        eprintln!("Error: {}", e);
    }
}

fn pin_go_version(tool: &str, update_go_mod: bool) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err("Invalid format. Use `golta pin go@<version>`.".into());
    }

    let version = tool.trim_start_matches("go@");
    let project_dir = env::current_dir()?;
    let pin_file = project_dir.join(".golta.json");

    let pin = PinFile {
        go: version.to_string(),
    };
    write(&pin_file, serde_json::to_string_pretty(&pin)?)?;

    println!("Pinned Go version {} to {}", version, pin_file.display());

    if update_go_mod {
        if let Err(e) = update_go_mod_version(version) {
            eprintln!("Warning: Could not update go.mod: {}", e);
        }
    }

    Ok(())
}

/// go.mod の go version を書き換える
fn update_go_mod_version(version_str: &str) -> Result<(), Box<dyn Error>> {
    let path = env::current_dir()?.join("go.mod");
    if !path.exists() {
        return Ok(()); // go.mod がない場合はスキップ
    }

    let content = read_to_string(&path)?;
    let mut mod_file = GoMod::parse(&content)?;

    // Go の version => "1.23.4" → "1.23"
    let parsed = semver::Version::parse(version_str)?;
    let major_minor = format!("{}.{}", parsed.major, parsed.minor);

    // go directive が既にある場合、同じなら何もしない
    if let Some(go) = &mod_file.go {
        if go.version == major_minor {
            println!("go.mod is already up to date.");
            return Ok(());
        }
    }

    // go バージョンを書き換え / 追加
    mod_file.edit_go_version(&major_minor)?;

    // 上書き保存
    write(&path, mod_file.to_string())?;
    println!("Updated go.mod to use Go {}", major_minor);

    Ok(())
}
