//!
//! Manifest and resource compiler and code generator.
//!
//! # Build Setup
//!
//! Add *gamekit* to your build dependencies in the `Cargo.toml` file similar
//! to this:
//!
//! ```
//! [build-dependencies]
//! gamekit = { version = "x.y.z", path = "..." }
//! ```
//!
//! Then create a custom build file `build.rs` and call the `compile()`
//! function:
//!
//! ```
//! ///
//! /// Custom Build
//! ///
//!
//! extern crate gamekit;
//!
//! fn main() -> std::process::ExitCode {
//!     gamekit::compiler::compile()
//! }
//! ```
//!
//! # Code Usage
//!
//! In your application main module, include the generated application manifest:
//! ```
//! include!(concat!(env!("OUT_DIR"), "/manifest.rs"));
//! ```
//!

use std::{env, fs};
use std::path::{Path, PathBuf};
use std::process::Command;

use json5;

use crate::manifest::Manifest;
use crate::options::ScalingMode;

/// Static container descriptor table
pub type ApplicationDescriptorTable = crate::manifest::ApplicationDescriptorTable;

/// Static data descriptors
pub type StaticDataDescriptor = crate::manifest::StaticDataDescriptor;

/// Static bitmap descriptors
pub type StaticBitmapDescriptor = crate::manifest::StaticBitmapDescriptor;

/// Static texture descriptors
pub type StaticTextureDescriptor = crate::manifest::StaticTextureDescriptor;

/// Static font descriptors
pub type StaticFontDescriptor = crate::manifest::StaticFontDescriptor;

/// Static texture descriptors
pub type StaticShaderDescriptor = crate::manifest::StaticShaderDescriptor;

/// Static material descriptors
pub type StaticMaterialDescriptor = crate::manifest::StaticMaterialDescriptor;

/// Static task descriptors
pub type StaticTaskDescriptor = crate::manifest::StaticTaskDescriptor;

/// Static options descriptor
pub type StaticOptionsDescriptor = crate::manifest::StaticOptionsDescriptor;

/// Static sample descriptor
pub type StaticSampleDescriptor = crate::manifest::StaticSampleDescriptor;

const DISABLE_MTIME_CHECK: bool = true;
const MANIFEST_FILENAME: &str = "manifest.json";

struct FileSpec {
    pub name: String,
    pub base_name: String,
    pub extension: String,
    pub abs_path: PathBuf,
    pub dir_path: PathBuf
}

impl FileSpec {
    pub fn new(file_path: &Path, base_path: &Path) -> Self {

        let abs_path = base_path.join(file_path);
        let dir_path = abs_path.parent().unwrap();

        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let base_name = file_path.file_stem().unwrap().to_str().unwrap();
        let extension = file_path.extension().unwrap().to_str().unwrap();

        Self {
            name: file_name.to_owned(),
            base_name: base_name.to_owned(),
            abs_path: abs_path.to_owned(),
            dir_path: dir_path.to_owned(),
            extension: extension.to_owned()
        }
    }

    pub fn new_ext(file_path: &Path, base_path: &Path, extension: &str) -> Self {

        let org_abs_path = base_path.join(file_path);
        let dir_path = org_abs_path.parent().unwrap();

        let base_name = org_abs_path.file_stem().unwrap().to_str().unwrap();
        let file_name = format!("{}.{}", base_name, extension);
        let abs_path = base_path.join(&file_name);

        Self {
            name: file_name,
            base_name: base_name.to_owned(),
            abs_path: abs_path.to_owned(),
            dir_path: dir_path.to_owned(),
            extension: extension.to_owned()
        }
    }
}

struct CompileSpec {
    pub src: FileSpec,
    pub dest: FileSpec
}

struct CompileOptions {
    pub base_path: PathBuf,
    pub out_path: PathBuf,
    pub is_debug: bool,
    pub optimization_level: String,
    pub disable_checks: bool,
    pub use_stdout: bool
}

impl CompileSpec {
    pub fn new(src: FileSpec, base_path: &Path, out_path: &Path) -> Self {

        let src_rel_path = match src.abs_path.strip_prefix(base_path) {
            Ok(p) => p,
            Err(_) => &src.abs_path
        };

        let dest: FileSpec =
            if src.name == MANIFEST_FILENAME {
                FileSpec::new_ext(src_rel_path, out_path, "rs")
            } else {
                FileSpec::new(src_rel_path, out_path)
            };

        Self {
            src,
            dest
        }
    }

    pub fn from_path(src: &Path, base_path: &Path, out_path: &Path) -> Self {
        let spec = FileSpec::new(src, base_path);
        Self::new(spec, base_path, out_path)
    }

    pub fn src_file(&self) -> &str {
        let src = self.src.abs_path.to_str().unwrap();
        src
    }

    pub fn dest_file(&self) -> &str {
        let src = self.dest.abs_path.to_str().unwrap();
        src
    }

}

fn get_mtime(path: &str) -> u64 {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(_) => { return 0; }
    };

    let modified = match meta.modified() {
        Ok(modified) => modified,
        Err(_) => { return 0; }
    };

    let secs = match modified.duration_since(std::time::UNIX_EPOCH) {
        Ok(secs) => secs.as_secs(),
        Err(_) => 0
    };

    secs
}

pub fn show_help() {

}

///
/// Compiles manifest and referenced resources and generates a 'manifest.rs'
/// file in the target output directory.
///
/// This function shall be called from a custom 'build.rs' file.
///
/// The generated file shall be included like this:
///
/// `include!(concat!(env!("OUT_DIR"), "/manifest.rs"));`
///
pub fn compile() -> std::process::ExitCode {

    if env::args().position(|arg| arg == "--help" || arg == "-h").is_some() {
        println!("Usage: compiler [OPTION]...");
        println!("Gamekit compiler executable.\n");
        println!("  -n, --nochecks           do not check if files exist");
        println!("  -s, --stdout             write to stdout");
        println!("  -h, --help               display this help and exit");
        return std::process::ExitCode::SUCCESS;
    }

    let cwd = env::current_dir().unwrap();
    let base_path = cwd;
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    let is_debug = match env::var_os("DEBUG") {
        Some(debug) => { if debug == "true" || debug == "True" || debug == "TRUE" { true } else { false } },
        None => false
    };

    let opt_level = match env::var_os("OPT_LEVEL") {
        Some(level) => match level.into_string() {
            Ok(s) => s,
            Err(_) => String::from("")
        },
        None => String::from("0")
    };

    let disable_checks = env::args().position(|arg| arg == "--nochecks" || arg == "-n").is_some();
    let use_stdout = env::args().position(|arg| arg == "--stdout" || arg == "-s").is_some();

    let options = CompileOptions {
        base_path,
        out_path,
        is_debug,
        optimization_level: opt_level,
        disable_checks,
        use_stdout
    };

    //let src_dir = src_path.into_os_string();
    //println!("cargo:warning=$SRC_DIR='{}'", src_dir.to_str().unwrap());
    //println!("cargo:warning=$OUT_DIR='{}'", env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");

    let manifest_path = options.base_path.join(MANIFEST_FILENAME);
    if manifest_path.is_file() {

        let json = match fs::read_to_string(&manifest_path) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("{}", e.to_string());
                return std::process::ExitCode::FAILURE;
            }
        };

        let manifest = match json5::from_str(json.as_str()) {
            Ok(manifest) => manifest,
            Err(e) => {
                eprintln!("failed to load manifest: {}", e.to_string());
                return std::process::ExitCode::FAILURE;
            }
        };

        let compile_spec = CompileSpec::from_path(&manifest_path, &options.base_path, &options.out_path);
        let res = compile_file(&manifest, &compile_spec, &options);
        if res != 0u8 {
            eprintln!("failed to compile manifest");
            return std::process::ExitCode::FAILURE;
        }

        let res = process_manifest(&manifest, &options);
        if res != 0u8 {
            eprintln!("failed to process manifest");
            return std::process::ExitCode::FAILURE;
        }

    }

    std::process::ExitCode::SUCCESS
}

fn process_manifest(manifest: &Manifest, options: &CompileOptions) -> u8 {

    let shader_path = options.base_path.join("resources").join("shaders");
    for shader in &manifest.shaders {
        let file_path = PathBuf::from(shader.path());
        let file_spec = FileSpec::new(&file_path, &shader_path);
        let compile_spec = CompileSpec::new(file_spec, &options.base_path, &options.out_path);

        let res = compile_file(manifest, &compile_spec, options);
        if res != 0u8 {
            return res;
        }

    }

    0u8
}

fn compile_file(manifest: &Manifest, compile_spec: &CompileSpec, options: &CompileOptions) -> u8 {

    let input_file = compile_spec.src_file();
    let output_file = compile_spec.dest_file();

    println!("cargo:rerun-if-changed={}", input_file);

    if !DISABLE_MTIME_CHECK {
        if get_mtime(input_file) <= get_mtime(output_file) {
            // no need to do anything
            // println!("cargo:warning=already up-to-date: {}", output_file);
            return 0;
        }
    }

    check_output_dir(&compile_spec.dest.dir_path);

    let mut res = 0u8;

    if compile_spec.src.name == MANIFEST_FILENAME {
        res = compile_manifest(manifest, compile_spec, options);
    } else if compile_spec.src.extension == "frag" || compile_spec.src.extension == "vert" {
        res = compile_shader(input_file, output_file, options);
    }

    res
}

fn compile_manifest(manifest: &Manifest, compile_spec: &CompileSpec, options: &CompileOptions) -> u8 {
    //println!("cargo:warning=compiling manifest '{}' to '{}'", input_file, output_file);

    let mut manifest_str = String::from(r#"
///
/// Gamekit Application Manifest
/// THIS IS GENERATED - DO NOT EDIT!
///
/// USAGE: include!(concat!(env!("OUT_DIR"), "/manifest.rs"));
///

/// Default logger instance
static LOGGER: gamekit::api::DefaultLogger = gamekit::api::default_logger();

/// Use static descriptors for options, resources and materials
use gamekit::compiler::ApplicationDescriptorTable;
use gamekit::compiler::StaticOptionsDescriptor;
use gamekit::compiler::StaticDataDescriptor;
use gamekit::compiler::StaticBitmapDescriptor;
use gamekit::compiler::StaticTextureDescriptor;
use gamekit::compiler::StaticFontDescriptor;
use gamekit::compiler::StaticShaderDescriptor;
use gamekit::compiler::StaticMaterialDescriptor;
use gamekit::compiler::StaticTaskDescriptor;
use gamekit::compiler::StaticSampleDescriptor;

"#);

    if manifest.options.is_some() {

        manifest_str.push_str("/// General application options\n");

        let o = manifest.options.as_ref().unwrap();

        manifest_str.push_str("pub static OPTIONS_DESCRIPTOR: &'static StaticOptionsDescriptor = &StaticOptionsDescriptor {\n");

        manifest_str.push_str(format!("    title: \"{}\",\n", o.title).as_str());
        manifest_str.push_str(format!("    window_x: {},\n", o.window_x).as_str());
        manifest_str.push_str(format!("    window_y: {},\n", o.window_y).as_str());
        manifest_str.push_str(format!("    window_width: {},\n", o.window_width).as_str());
        manifest_str.push_str(format!("    window_height: {},\n", o.window_height).as_str());
        manifest_str.push_str(format!("    view_width: {},\n", o.view_width).as_str());
        manifest_str.push_str(format!("    view_height: {},\n", o.view_height).as_str());
        manifest_str.push_str(format!("    scaling_mode: {},\n", ScalingMode::from_string(&o.scaling_mode)).as_str());
        manifest_str.push_str(format!("    fps: {},\n", o.fps).as_str());
        manifest_str.push_str(format!("    show_statistics: {},\n", o.show_statistics).as_str());
        manifest_str.push_str(format!("    queue_size: {},\n", o.queue_size).as_str());
        manifest_str.push_str(format!("    headless: {},\n", o.headless).as_str());
        manifest_str.push_str(format!("    enable_validation_layer: {},\n", o.enable_validation_layer).as_str());
        manifest_str.push_str(format!("    enable_api_dump_layer: {}\n", o.enable_api_dump_layer).as_str());

        manifest_str.push_str("};\n");
    }

    manifest_str.push('\n');

    manifest_str.push_str("/// Bitmap descriptors\n");
    for (idx, bitmap) in manifest.bitmaps.iter().enumerate() {
        manifest_str.push_str(format!("static BMP_{}: &'static[u8] = gamekit::include_resource!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/resources/bitmaps/{}\"));\n", idx, bitmap.path()).as_str());
    }
    manifest_str.push_str("pub static BITMAP_DESCRIPTORS: &'static [StaticBitmapDescriptor] = &[\n");
    for (idx, bitmap) in manifest.bitmaps.iter().enumerate() {
        let abs_path = Path::new(&compile_spec.src.dir_path).join("resources/bitmaps").join(bitmap.path());
        if !options.disable_checks && !abs_path.is_file() {
            eprintln!("error: bitmap file does not exist: {}", abs_path.to_str().unwrap());
            return 1;
        }
        let ext = abs_path.extension().unwrap();
        let format = if ext == "bin" { "charmem" } else { "bitmap" };
        manifest_str.push_str(format!("    StaticBitmapDescriptor::new(\"{}\", BMP_{}, \"{}\"),\n", bitmap.name(), idx, format).as_str());
    }
    manifest_str.push_str("];\n\n");

    manifest_str.push_str("/// Texture descriptors\n");
    for (idx, texture) in manifest.textures.iter().enumerate() {
        manifest_str.push_str(format!("static TEX_{}: &'static[u8] = gamekit::include_resource!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/resources/textures/{}\"));\n", idx, texture.path()).as_str());
    }
    manifest_str.push_str("pub static TEXTURE_DESCRIPTORS: &'static [StaticTextureDescriptor] = &[\n");
    for (idx, texture) in manifest.textures.iter().enumerate() {
        let abs_path = Path::new(&compile_spec.src.dir_path).join("resources/textures").join(texture.path());
        if !options.disable_checks && !abs_path.is_file() {
            eprintln!("error: texture file does not exist: {}", abs_path.to_str().unwrap());
            return 1;
        }
        let ext = abs_path.extension().unwrap();
        let format = if ext == "bin" { "charmem" } else { "bitmap" };
        manifest_str.push_str(format!("    StaticTextureDescriptor::new(\"{}\", TEX_{}, \"{}\"),\n", texture.name(), idx, format).as_str());
    }
    manifest_str.push_str("];\n\n");

    manifest_str.push_str("/// Font descriptors\n");
    manifest_str.push_str("pub static FONT_DESCRIPTORS: &'static [StaticFontDescriptor] = &[\n");
    for (_idx, font) in manifest.fonts.iter().enumerate() {
        manifest_str.push_str(format!("    StaticFontDescriptor::new(\"{}\", r##\"{}\"##, {}, {}, \"{}\"),\n", font.name(), font.charset(), font.char_width(), font.char_height(), font.texture()).as_str());
    }
    manifest_str.push_str("];\n\n");

    manifest_str.push_str("/// Data descriptors\n");
    for (idx, data) in manifest.data.iter().enumerate() {
        manifest_str.push_str(format!("static DAT_{}: &'static[u8] = gamekit::include_resource!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/resources/data/{}\"));\n", idx, data.path()).as_str());
    }
    manifest_str.push_str("pub static DATA_DESCRIPTORS: &'static [StaticDataDescriptor] = &[\n");
    for (idx, data) in manifest.data.iter().enumerate() {
        let abs_path = Path::new(&compile_spec.src.dir_path).join("resources/data").join(data.path());
        if !options.disable_checks && !abs_path.is_file() {
            eprintln!("error: data file does not exist: {}", abs_path.to_str().unwrap());
            return 1;
        }
        manifest_str.push_str(format!("    StaticDataDescriptor::new(\"{}\", DAT_{}),\n", data.name(), idx).as_str());
    }
    manifest_str.push_str("];\n\n");

    manifest_str.push_str("/// Shader descriptors\n");
    for (idx, shader) in manifest.shaders.iter().enumerate() {
        manifest_str.push_str(format!("static SHD_{}: &'static[u8] = gamekit::include_resource!(concat!(env!(\"OUT_DIR\"), \"/resources/shaders/{}\"));\n", idx, shader.path()).as_str());
    }
    manifest_str.push_str("pub static SHADER_DESCRIPTORS: &'static [StaticShaderDescriptor] = &[\n");
    for (idx, shader) in manifest.shaders.iter().enumerate() {
        let abs_path = Path::new(&compile_spec.src.dir_path).join("resources/shaders").join(shader.path());
        if !options.disable_checks && !abs_path.is_file() {
            eprintln!("error: shader file does not exist: {}", abs_path.to_str().unwrap());
            return 1;
        }
        let ext = abs_path.extension().unwrap();
        let format = if ext == "vert" { "vertex" } else { "fragment" };
        manifest_str.push_str(format!("    StaticShaderDescriptor::new(\"{}\", SHD_{}, \"{}\"),\n", shader.name(), idx, format).as_str());
    }
    manifest_str.push_str("];\n\n");

    manifest_str.push_str("/// Material descriptors\n");

    manifest_str.push_str("pub static MATERIAL_DESCRIPTORS: &'static [StaticMaterialDescriptor] = &[\n");

    for m in &manifest.materials {
        manifest_str.push_str(format!("    StaticMaterialDescriptor::new(").as_str());
        manifest_str.push_str(format!(
            "\"{}\", \"{}\", \"{}\", {}, {}, \"{}\", \"{}\", {}, \"{}\", {}, {}, {}, {}",
            m.name, m.font, m.texture, m.texture_binding, m.texture_filtering, m.vertex_shader, m.fragment_shader, m.blending, m.blend_mode, m.backface_culling, m.frontface_clockwise, m.depth_testing, m.depth_writing
        ).as_str());
        manifest_str.push_str(format!("    ),\n").as_str());
    }

    manifest_str.push_str("];");

    manifest_str.push('\n');

    manifest_str.push_str("/// Task descriptors\n");

    manifest_str.push_str("pub static TASK_DESCRIPTORS: &'static [StaticTaskDescriptor] = &[\n");

    for t in &manifest.tasks {
        manifest_str.push_str(format!("   StaticTaskDescriptor::new(").as_str());

        manifest_str.push_str(format!(
            "\"{}\", {}, {}",
            t.name, t.id, t.interval
        ).as_str());

        manifest_str.push_str(format!("),\n").as_str());
    }

    manifest_str.push_str("];\n");


    manifest_str.push_str("/// Music descriptors\n");
    for (idx, music) in manifest.music.iter().enumerate() {
        manifest_str.push_str(format!("static MUS_{}: &'static[u8] = gamekit::include_resource!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/resources/music/{}\"));\n", idx, music.path()).as_str());
    }
    manifest_str.push_str("pub static MUSIC_DESCRIPTORS: &'static [StaticSampleDescriptor] = &[\n");
    for (idx, music) in manifest.music.iter().enumerate() {
        let abs_path = Path::new(&compile_spec.src.dir_path).join("resources/music").join(music.path());
        if !options.disable_checks && !abs_path.is_file() {
            eprintln!("error: music file does not exist: {}", abs_path.to_str().unwrap());
            return 1;
        }
        manifest_str.push_str(format!("    StaticSampleDescriptor::new(\"{}\", MUS_{}),\n", music.name(), idx).as_str());
    }
    manifest_str.push_str("];\n");


    manifest_str.push_str("/// Sample descriptors\n");
    for (idx, sample) in manifest.samples.iter().enumerate() {
        manifest_str.push_str(format!("static SAM_{}: &'static[u8] = gamekit::include_resource!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/resources/samples/{}\"));\n", idx, sample.path()).as_str());
    }
    manifest_str.push_str("pub static SAMPLE_DESCRIPTORS: &'static [StaticSampleDescriptor] = &[\n");
    for (idx, sample) in manifest.samples.iter().enumerate() {
        let abs_path = Path::new(&compile_spec.src.dir_path).join("resources/samples").join(sample.path());
        if !options.disable_checks && !abs_path.is_file() {
            eprintln!("error: sample file does not exist: {}", abs_path.to_str().unwrap());
            return 1;
        }
        manifest_str.push_str(format!("    StaticSampleDescriptor::new(\"{}\", SAM_{}),\n", sample.name(), idx).as_str());
    }
    manifest_str.push_str("];\n");


    manifest_str.push_str(r#"
///Descriptor table
pub static DESCRIPTOR_TABLE: &'static ApplicationDescriptorTable = &ApplicationDescriptorTable {
    options: OPTIONS_DESCRIPTOR,
    bitmaps: BITMAP_DESCRIPTORS,
    textures: TEXTURE_DESCRIPTORS,
    fonts: FONT_DESCRIPTORS,
    shaders: SHADER_DESCRIPTORS,
    data: DATA_DESCRIPTORS,
    materials: MATERIAL_DESCRIPTORS,
    tasks: TASK_DESCRIPTORS,
    music: MUSIC_DESCRIPTORS,
    samples: SAMPLE_DESCRIPTORS
};

/// Application main
fn gamekit_main() {
    gamekit::default_main::<App>(DESCRIPTOR_TABLE, &LOGGER);
}
"#);

    if !options.use_stdout {
        fs::write(compile_spec.dest_file(), manifest_str).unwrap();
    } else {
        print!("{}", manifest_str);
    }

    0u8
}

fn check_output_dir(out_dir: &Path) {
    //println!("create output directory: {}", out_dir.to_str().unwrap());
    let _ = fs::create_dir_all(out_dir);
}

fn compile_shader(input_file: &str, output_file: &str, options: &CompileOptions) -> u8 {
    //println!("cargo:warning=compiling shader '{}' to '{}'", input_file, output_file);

    let output_arg = format!("-o{}", output_file);

    let arg_executable = "glslc";

    let mut args = vec![
        "--target-env=vulkan1.3",
        "-mfmt=bin",
    ];

    if options.is_debug {
        args.push("-g") // add source level debug information
    }

    if options.optimization_level.len() > 0 && options.optimization_level != "0" {
        if options.optimization_level == "s" || options.optimization_level == "z" {
            args.push("-Os");
        } else {
            args.push("-O");
        }
    }

    args.push(output_arg.as_str());
    args.push(input_file);

    //println!("cargo:warning=args:{}", args.join(" "));

    let output = Command::new(arg_executable)
        .args(args)
        .output()
        .expect("failed to compile shader");

    let status = output.status;

    let exit_code: u8 = match status.code() {
        Some(code) => { code as u8 },
        None => 1u8
    };

    println!("{}", String::from_utf8(output.stdout).unwrap());
    eprintln!("{}", String::from_utf8(output.stderr).unwrap());

    exit_code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {

        let cwd = env::current_dir().unwrap();
        let out_path = cwd.join("build/debug");
        let out_dir = out_path.into_os_string();

        env::set_var("OUT_DIR", out_dir);
        env::set_var("DEBUG", "true");
        env::set_var("OPT_LEVEL", "1");

        compile();

    }
}

/*

pub fn fetch_files_impl(base_path: &Path, folder: &Path, compile_list: &mut Vec<CompileSpec>) {

    let paths = match fs::read_dir(&folder) {
        Ok(paths) => paths,
        Err(_) => {
            eprintln!("cannot access folder '{}'", folder.to_str().unwrap());
            return;
        }
    };

    for dir_entry in paths {

        let entry = match dir_entry {
            Ok(entry) => entry,
            Err(_) => { continue; }
        };

        let filepath = entry.path();
        let filename_os = entry.file_name();

        let filename = match filename_os.to_str() {
            Some(s) => s,
            None => { continue; }
        };

        if filename.starts_with(".") || filename == "target" {
            // ignore anything that starts with a period '.'
            continue;
        }

        if filepath.is_file() {

            match filepath.extension() {
                Some(ext) => {
                    let extension = match ext.to_str() {
                        Some(s) => s,
                        None => { continue; }
                    };

                    let base_name = match filepath.file_stem() {
                        Some(s) => {
                            match s.to_str() {
                                Some(basename) => basename,
                                None => { continue; }
                            }
                        },
                        None => { continue; }
                    };

                    if extension == "vert" || extension == "frag" || extension == "json" || extension == "json5" {

                        let file_spec = FileSpec {
                            name: filename.to_owned(),
                            base_name: base_name.to_owned(),
                            extension: extension.to_owned(),
                            abs_path: filepath.to_owned(),
                            dir_path: folder.to_owned()
                        };

                        let compile_spec = CompileSpec {
                            src: file_spec,
                            dest: None
                        };

                        compile_list.push(compile_spec);
                    }
                },
                None => {
                    continue;
                }
            };
        } else if filepath.is_dir() {
            fetch_files_impl(base_path, &filepath, compile_list)
        }
    }

}

pub fn fetch_files(src_dir: &Path, file_list: &mut Vec<CompileSpec>) {
    fetch_files_impl(src_dir, src_dir, file_list);
}

pub fn compile_list(base_dir: &Path, compile_list: &Vec<CompileSpec>, out_dir: &OsString, is_debug: bool, opt_level: &String) -> u8 {

    for compile_spec in compile_list {

        let src_file_spec = &compile_spec.src;

        let input_file = match src_file_spec.abs_path.to_str() {
            Some(s) => s,
            None => { continue; }
        };

        println!("cargo:rerun-if-changed={}", input_file);

        let input_rel_path = match src_file_spec.dir_path.strip_prefix(base_dir) {
            Ok(p) => p,
            Err(_) => { continue; }
        };

        let output_path = Path::new(&out_dir).join(input_rel_path);
        check_output_dir(&output_path);


        let output_ext = if src_file_spec.name == MANIFEST_FILENAME {
            "rs"
        } else {
            &src_file_spec.extension
        };

        let output_name = format!("{}.{}", src_file_spec.base_name, output_ext);

        let output_file_path = output_path.join(output_name);
        let output_file = match output_file_path.to_str() {
            Some(s) => s,
            None => { continue; }
        };

        let res = compile_file(&src_file_spec.name, &input_file, &output_file, &src_file_spec.extension, is_debug, opt_level);
        if res != 0u8 {
            return res;
        }
    }

    0u8
}
*/
