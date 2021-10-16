use embuild::{self, build::LinkArgs};

// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> anyhow::Result<()> {
    LinkArgs::output_propagated("ESP_IDF")?;

    Ok(())
}
