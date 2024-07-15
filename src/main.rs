use anyhow::{ Context, Result };
use std::env::args;
use std::fs::File;
use std::io::{ self, BufReader, Read };
use std::thread;

fn read_input<R: Read>(mut reader: R, pager: minus::Pager) -> Result<()> {
    let mut changes = || -> Result<()> {
        let mut buff = String::new();
        reader.read_to_string(&mut buff).context("Failed to read input")?;
        pager.push_str(&buff).context("Failed to push content to pager")?;
        Ok(())
    };

    let pager_clone = pager.clone();
    let res1 = thread::spawn(move || minus::dynamic_paging(pager_clone));
    let res2 = changes();

    res1.join().expect("Paging thread panicked").context("Failed to run dynamic paging")?;
    res2?;

    Ok(())
}

fn main() -> Result<()> {
    let arguments: Vec<String> = args().collect();

    let input: Box<dyn Read> = if atty::is(atty::Stream::Stdin) {
        // No piped input, check for file argument
        if arguments.len() < 2 {
            anyhow::bail!(
                "No input provided. Usage: {} <filename> or pipe content to stdin",
                arguments[0]
            );
        }
        let filename = &arguments[1];
        let file = File::open(filename).with_context(||
            format!("Failed to open file '{}'", filename)
        )?;
        Box::new(BufReader::new(file))
    } else {
        // Piped input
        Box::new(BufReader::new(io::stdin()))
    };

    let output = minus::Pager::new();

    if atty::is(atty::Stream::Stdin) {
        output.set_prompt(&arguments[1]).context("Failed to set pager prompt")?;
    } else {
        output.set_prompt("stdin").context("Failed to set pager prompt")?;
    }

    read_input(input, output)
}
