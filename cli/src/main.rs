use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = git_seek::Cli::parse();
    git_seek::run(cli)
}
