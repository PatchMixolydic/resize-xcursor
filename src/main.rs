#![feature(array_chunks)]

mod write_xcursor;

use anyhow::bail;
use clap::Parser;
use std::{
    fs::{self, File},
    iter,
    path::PathBuf,
};
use write_xcursor::{Image, Xcursor};
use xcursor::parser::parse_xcursor;

/// Resizes Xcursor files
#[derive(Parser)]
struct Args {
    /// The scale factor to apply to each cursor.
    ///
    /// For example, a scale of 2 applied to a 32x32 pixel cursor will
    /// result in a 64x64 pixel cursor.
    #[clap(short, long, verbatim_doc_comment)]
    scale: u32,

    /// If given, ignores any unrecognized filetypes.
    ///
    /// This is useful if the current directory contains files that
    /// aren't Xcursors, such as PNGs or metadata files.
    #[clap(short, long, verbatim_doc_comment)]
    ignore_unrecognized: bool,

    /// A list of output filenames.
    ///
    /// There must be exactly as many input filenames as output filenames.
    #[clap(short, long = "output", value_parser, verbatim_doc_comment)]
    // `value_parser` on `PathBuf` ^^^^^^^^^^^^ allows for non-UTF-8 paths
    output_filenames: Option<Vec<PathBuf>>,

    /// One or more Xcursor files to process.
    ///
    /// Note that directories are currently unsupported; a glob can
    /// be used instead.
    #[clap(value_parser, verbatim_doc_comment)]
    input_filenames: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let output_filenames = match args.output_filenames {
        Some(output_filenames) => {
            if output_filenames.len() != args.input_filenames.len() {
                bail!(
                    "if output filenames are provided, there must be as many output file names as input filenames\n\
                    (got {} input filenames and {} output filenames)",
                    args.input_filenames.len(),
                    output_filenames.len(),
                );
            }
            output_filenames
        }

        None => args.input_filenames.clone(),
    };

    for (input_filename, output_filename) in args.input_filenames.into_iter().zip(output_filenames)
    {
        let cursor_bytes = fs::read(&input_filename)?;

        let cursor_images = match parse_xcursor(&cursor_bytes) {
            Some(res) => res,
            None => {
                if args.ignore_unrecognized {
                    continue;
                }

                bail!("{} doesn't seem to be a valid Xcursor file", input_filename.display());
            }
        };

        let mut cursor = Xcursor::new();

        for image in cursor_images {
            let unscaled_pixels = image
                .pixels_rgba
                .array_chunks::<4>()
                .copied()
                .map(u32::from_le_bytes)
                .collect::<Vec<_>>();

            let scaled_pixels = unscaled_pixels
                // Get each row
                .chunks_exact(image.width as usize)
                .flat_map(|row| {
                    let scaled_rows = row
                        .iter()
                        // Duplicate each pixel `args.scale` times
                        .flat_map(|pixel| iter::repeat(*pixel).take(args.scale as usize));

                    // Duplicate each row `args.scale` times
                    iter::repeat(scaled_rows)
                        .take(args.scale as usize)
                        .flatten()
                })
                .collect();

            let output_image = Image::new(
                image.size * args.scale,
                image.width * args.scale,
                image.height * args.scale,
                image.xhot * args.scale,
                image.yhot * args.scale,
                image.delay,
                scaled_pixels,
            )?;

            cursor.add_chunk(output_image)?;
        }

        let output_file = File::create(output_filename)?;
        cursor.write_to(output_file)?;
    }

    Ok(())
}
