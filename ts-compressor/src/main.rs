use anyhow::{Context, Result};
use clap::Parser;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use swc_core::{
    common::{
        errors::Handler,
        source_map::SourceMap,
        Globals, GLOBALS, Mark,
    },
    ecma::{
        codegen::{text_writer::JsWriter, Emitter},
        minifier::{
            optimize,
            option::{ExtraOptions, MinifyOptions},
        },
        parser::{lexer::Lexer, Parser as SwcParser, StringInput, Syntax, TsSyntax},
        transforms::typescript::strip,
        visit::FoldWith,
    },
};
use walkdir::WalkDir;

#[derive(Parser)]
struct Args {
    input_dir: PathBuf,
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    fs::create_dir_all(&args.output_dir)?;

    for entry in WalkDir::new(&args.input_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(false, |e| e == "ts" || e == "tsx") {
            let minified = minify_file(entry.path())?;
            let out_path = args.output_dir.join(entry.path().file_name().unwrap()).with_extension("js");
            let mut out_file = File::create(&out_path)?;
            out_file.write_all(minified.as_bytes())?;
        }
    }

    Ok(())
}

fn minify_file(path: &Path) -> Result<String> {
    let cm = std::rc::Rc::new(SourceMap::default());
    let _handler = Handler::with_emitter_writer(Box::new(std::io::stderr()), Some(cm.clone()));

    let fm = cm.load_file(path).context("Failed to load file")?;

    GLOBALS.set(&Globals::new(), || {
        // Parse TS
        let ts_config = TsSyntax { 
            tsx: path.extension().map_or(false, |e| e == "tsx"), 
            ..Default::default() 
        };
        let lexer = Lexer::new(
            Syntax::Typescript(ts_config),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = SwcParser::new_from(lexer);
        let mut program = parser.parse_program().map_err(|e| anyhow::anyhow!("Parse failed: {:?}", e))?;

        // Strip TS types
        program = program.fold_with(&mut strip(Mark::new(), Mark::new()));

        // Minify with compression and mangling
        let minify_opts = MinifyOptions {
            compress: Some(Default::default()),
            mangle: Some(Default::default()),
            ..Default::default()
        };
        program = optimize(
            program.into(),
            cm.clone(),
            None,
            None,
            &minify_opts,
            &ExtraOptions { 
                unresolved_mark: Mark::new(),
                top_level_mark: Mark::new(),
                mangle_name_cache: None,
            },
        );

        // Serialize to code
        let mut buf = Vec::new();
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let mut emitter = Emitter {
            cfg: Default::default(),
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };
        emitter.emit_program(&program).context("Failed to emit code")?;
        
        Ok(String::from_utf8(buf).context("Invalid UTF-8")?)
    })
}