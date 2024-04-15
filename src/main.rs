use std::io::{BufWriter, Write};

use catppuccin::{Color, FlavorName};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct App {
    #[clap(subcommand)]
    command: Option<Command>,
    #[clap(short, long)]
    flavor: Option<FlavorName>,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    Palette {
        #[clap(short = 'F', long)]
        format: Option<String>,
    },
}

impl Default for Command {
    fn default() -> Self {
        Command::Palette {
            format: Default::default(),
        }
    }
}

struct Format<'a> {
    template: &'a str,
}

impl<'a> Format<'a> {
    fn new(template: &'a str) -> Self {
        Self { template }
    }

    fn paint(&self, color: &Color, out: &mut impl std::io::Write) -> anyhow::Result<()> {
        let mut out = BufWriter::new(out);
        let mut chars = self.template.chars();

        while let Some(ch) = chars.next() {
            match ch {
                '%' => {
                    let next = chars.next().ok_or(anyhow::anyhow!(
                        "format string expected format specifier after %"
                    ))?;

                    match next {
                        '%' => write!(out, "%")?,
                        'n' => write!(out, "{:14}", color.name.to_string())?,
                        'b' => write!(out, "{}", color.ansi_paint("██████████████"))?,
                        'r' => write!(
                            out,
                            "{}",
                            ansi_term::Style::new().bold().paint(format!(
                                "rgb({:3}, {:3}, {:3})",
                                color.rgb.r, color.rgb.g, color.rgb.b
                            ))
                        )?,
                        'h' => write!(
                            out,
                            "{}",
                            ansi_term::Style::new().bold().paint(format!(
                                "hsl({:3}, {:3}%, {:3}%)",
                                color.hsl.h.round(),
                                (color.hsl.s * 100.0).round() / 100.0,
                                (color.hsl.l * 100.0).round() / 100.0
                            ))
                        )?,
                        'x' => write!(
                            out,
                            "{}",
                            ansi_term::Style::new().bold().paint(color.hex.to_string())
                        )?,
                        _ => {
                            anyhow::bail!(
                                "unknown format specifier {next}. Must be one of '%nbrhx'"
                            )
                        }
                    }
                }
                '\\' => {
                    let next = chars
                        .next()
                        .ok_or(anyhow::anyhow!("format string expected modifier after \\"))?;

                    match next {
                        'n' => writeln!(out)?,
                        't' => write!(out, "\t")?,
                        'r' => write!(out, "\r")?,
                        '\\' => write!(out, "\\")?,
                        _ => anyhow::bail!("unknown modifier {next}. Must be one of 'ntr\\'"),
                    }
                }
                _ => {
                    write!(out, "{ch}")?;
                }
            }
        }

        Ok(())
    }
}

struct PalettePrinter<'a> {
    format: Format<'a>,
    flavor: FlavorName,
}

impl<'a> PalettePrinter<'a> {
    fn new(flavor: FlavorName) -> Self {
        Self {
            format: Format::new("%n %b │ %x │ %r │ %h\n"),
            flavor,
        }
    }

    fn set_format(&mut self, format: Format<'a>) {
        self.format = format;
    }

    fn print(&self, mut out: impl std::io::Write) -> anyhow::Result<()> {
        let palette = catppuccin::PALETTE.get_flavor(self.flavor);
        for color in palette.iter() {
            self.format.paint(color, &mut out)?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let command = app.command.unwrap_or_default();

    match command {
        Command::Palette { format } => {
            let mut printer = PalettePrinter::new(app.flavor.unwrap_or(FlavorName::Macchiato));

            let fmt;
            if let Some(format) = format {
                fmt = format;
                printer.set_format(Format::new(&fmt));
            }

            printer.print(std::io::stdout().lock())?
        }
    }

    Ok(())
}
