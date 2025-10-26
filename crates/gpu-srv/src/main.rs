use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gpu-srv", version, author, about = "Headless GPU service (Phase-1)")]
struct Cli {
    /// Render a simple triangle (stub). For now, clears to solid color.
    #[arg(long)]
    triangle: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _img = gpu_srv::render_solid_rgba8(64, 64, if cli.triangle { [0.0, 1.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0, 1.0] })?;
    if cli.triangle {
        println!("triangle ok");
    } else {
        println!("solid ok");
    }
    Ok(())
}

