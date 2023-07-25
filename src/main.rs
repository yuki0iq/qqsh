use rustyline::{
    config::{CompletionType, Config, EditMode},
    history::FileHistory,
    Editor, Result as RlResult,
};
use statusline::StatusLine;
use std::{env, path::PathBuf};

fn main() -> RlResult<()> {
    println!("Hello, world!");

    let mut rl: Editor<(), FileHistory> = Editor::with_config(
        Config::builder()
            .auto_add_history(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .tab_stop(4)
            .build(),
    )?;

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

        print!("{}\n{}\n", sl.to_title("qqsh"), sl.to_top());
        // TODO async prompt

        let readline = rl.readline(&sl.to_bottom());
        // TODO stop async prompt

        match &readline {
            Ok(s) if s == "exit" => break,
            Ok(s) => println!("entered command {s}"),
            Err(err) => eprintln!("Readline error: {err}"),
        }
    }

    if let Some(hist_path) = hist_path {
        rl.save_history(&hist_path)?;
    }

    Ok(())
}
