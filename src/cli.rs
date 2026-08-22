use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(long)]
    pub system: Option<String>,

    #[arg(long)]
    pub model: Option<String>,

    #[arg(long)]
    pub temperature: Option<f32>,

    #[arg(long = "max-tokens")]
    pub max_tokens: Option<usize>,

    #[arg(long)]
    pub budget: Option<i32>,
}
