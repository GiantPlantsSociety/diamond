// whisper-fetch.py 
// Usage: whisper-fetch.py [options] path

// Options:
//   -h, --help            show this help message and exit
//   --from=_FROM          Unix epoch time of the beginning of your requested
//                         interval (default: 24 hours ago)
//   --until=UNTIL         Unix epoch time of the end of your requested interval
//                         (default: now)
//   --json                Output results in JSON form
//   --pretty              Show human-readable timestamps instead of unix times
//   -t TIME_FORMAT, --time-format=TIME_FORMAT
//                         Time format to use with --pretty; see time.strftime()
//   --drop=DROP           Specify 'nulls' to drop all null values. Specify
//                         'zeroes' to drop all zero values. Specify 'empty' to
//                         drop both null and zero values

fn main() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("whisper-fetch");
}
