#![feature(let_chains)]
use rustyline::{
    completion::{FilenameCompleter},
    config::{CompletionType, Config, EditMode},
    error::ReadlineError,
    highlight::{MatchingBracketHighlighter},
    hint::{HistoryHinter},
    history::FileHistory,
    Editor, Result as RlResult,
};
use rustyline::{Completer, Highlighter, Hinter, Helper, Validator};
use statusline::StatusLine;
use std::{
    env,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Helper, Hinter, Highlighter, Completer, Validator)]
struct PromptHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,

    #[rustyline(Highlighter)]
    highlighter: MatchingBracketHighlighter,

    #[rustyline(Hinter)]
    hinter: HistoryHinter,

    #[rustyline(Validator)]
    validator: (),
}

fn main() -> RlResult<()> {
    println!("Hello, world!");

    let mut rl: Editor<PromptHelper, FileHistory> = Editor::with_config(
        Config::builder()
            .auto_add_history(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .tab_stop(4)
            .build(),
    )?;
    rl.set_helper(Some(PromptHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter{},
        validator: (),
    }));

    let hist_path = if let Ok(home) = &env::var("HOME") {
        let hist_path = PathBuf::from(String::from(home)).join(".qqsh-history");
        if rl.load_history(&hist_path).is_err() {
            eprintln!("Could not read history file");
        }
        Some(hist_path)
    } else {
        None
    };

    loop {
        let sl = StatusLine::from_env::<&str>(&[]);
        let prompt = sl.to_bottom();

        print!("{}\n{}\n", sl.to_title("qqsh"), sl.to_top());
        
        let stop_async = Arc::new(Mutex::new(false));
        let stop_async_cloned = Arc::clone(&stop_async);

        // TODO more _correct_ async prompt shutdown
        let _async_prompt_handle = thread::spawn(move || {
            let sl = sl.extended();
            if let Ok(guard) = stop_async_cloned.lock() && !*guard.deref() {
                eprint!("\x1b[s\x1b[G\x1b[A{}\x1b[u", sl.to_top());
            };
        });

        let readline = rl.readline(&prompt);
        {
            let mut sa = stop_async.lock().unwrap();
            *(sa.deref_mut()) = true;
        }

        match &readline {
            Ok(s) if s == "exit" => break,
            Ok(s) => println!("entered command {s}"),
            Err(ReadlineError::Interrupted) => eprintln!("--> Ctrl-C"),
            Err(ReadlineError::Eof) => eprintln!("--> <EOF>"),
            Err(err) => eprintln!("Readline error: {err}"),
        }
    }

    if let Some(hist_path) = hist_path {
        rl.save_history(&hist_path)?;
    }

    Ok(())
}
