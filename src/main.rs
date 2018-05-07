extern crate debruijn;
extern crate bio;
extern crate clap;

// Import some modules
use std::fs::File;
use std::io;
use std::io::Write;

use debruijn::graph::{BaseGraph};
use debruijn::compression::{SimpleCompress, compress_graph};
use debruijn::dna_string::*;
use debruijn::{filter, kmer, Exts};
use debruijn::Mer;
use clap::{Arg, App};
use bio::io::fasta;
use std::str;

pub type KmerType = kmer::Kmer32;
const MIN_KMERS: usize = 1;
const STRANDED: bool = true;

fn read_fasta(reader: fasta::Reader<File>) -> () {

    let summarizer = filter::CountFilterSet::new(MIN_KMERS);
    let mut seqs = Vec::new();
    let mut trancript_counter = 0;

    println!("Starting Reading the Fasta file");
    for result in reader.records() {
        // obtain record or fail with error
        let record = result.unwrap();
        let dna_string = DnaString::from_dna_string( str::from_utf8(record.seq()).unwrap() );

        // obtain sequence and push into the relevant vector
        seqs.push((dna_string, Exts::empty(), ()));

        trancript_counter += 1;
        if trancript_counter % 1000 == 0 {
            print!("\r Done Reading {} transcripts", trancript_counter);
            io::stdout().flush().ok().expect("Could not flush stdout");
        }
        break;
    }

    println!("\nStarting kmer filtering");
    let (valid_kmers, obs_kmers): (Vec<(KmerType, (Exts, _))>, _) =
        filter::filter_kmers::<KmerType, _, _, _, _>(&seqs, summarizer, STRANDED);

    println!("Kmers observed: {}, kmers accepted: {}", obs_kmers.len(), valid_kmers.len());
    println!("Starting uncompressed de-bruijn graph construction");

    // Create a DBG with one node per input kmer
    let mut base_graph: BaseGraph<KmerType, u16> = BaseGraph::new(STRANDED);

    for (kmer, (exts, _)) in valid_kmers.clone() {
        base_graph.add(kmer.iter(), exts, 0 as u16);
    }
    let uncompressed_dbg = base_graph.finish();

    println!("Done uncompressed de-bruijn graph construction; Starting Compression");
    let spec = SimpleCompress::new(|d1: u16, d2: &u16| d1 + d2);
    let simp_dbg = compress_graph(STRANDED, spec, uncompressed_dbg, None);

    let is_cmp = simp_dbg.is_compressed();
    if is_cmp.is_some() {
        println!("not compressed: nodes: {:?}", is_cmp);
        simp_dbg.print();
    }

    println!("Finished Indexing !");
}

fn main() {
    let matches = App::new("De-bruijn-mapping")
        .version("1.0")
        .author("Avi S. <avi.srivastava@10xgenomics.com>")
        .about("De-bruijn graph based lightweight mapping for single-cell data")
        .arg(Arg::with_name("fasta")
             .short("f")
             .long("fasta")
             .value_name("FILE")
             .help("Genome Input file")
             .required(true))
        .get_matches();

    // Gets a value for config if supplied by user
    let fasta_file = matches.value_of("fasta").unwrap();
    println!("Path for FASTA: {}", fasta_file);

    // obtain reader or fail with error (via the unwrap method)
    let reader = fasta::Reader::from_file(fasta_file).unwrap();

    read_fasta(reader);
}