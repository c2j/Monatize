use ai_runtime::{ask, summarize_text};
use clap::Parser;
use message_defs::{AiRequest};

#[derive(Parser, Debug)]
#[command(name="ai-runtime")]
struct Args {
    /// Run a quick benchmark (mock)
    #[arg(long)]
    bench: bool,

    /// Optional prompt to summarize
    #[arg(long)]
    ask: Option<String>,

    /// Max tokens for summary (default 16)
    #[arg(long, default_value_t = 16)]
    max_tokens: u32,
}

fn main() {
    let args = Args::parse();
    if args.bench {
        // quick micro benchmark
        let mut acc = 0usize;
        for _ in 0..10_000 {
            let s = summarize_text("The quick brown fox jumps over the lazy dog", 8);
            acc += s.len();
        }
        println!("ai-runtime bench ok: acc={acc}");
        return;
    }

    if let Some(p) = args.ask {
        let resp = ask(AiRequest { prompt: p, max_tokens: args.max_tokens }).unwrap();
        println!("{}", resp.text);
        return;
    }

    eprintln!("ai-runtime: nothing to do. Use --bench or --ask");
}

