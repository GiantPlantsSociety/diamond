#[macro_use]
extern crate structopt;
extern crate failure;
extern crate walkdir;
extern crate whisper;

use failure::Error;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use whisper::fill::fill_archives;

/// Copies data from src to dst, if missing.
#[derive(Debug, StructOpt)]
#[structopt(name = "whisper-fill")]
struct Args {
    /// Lock whisper files (is not implemented)
    #[structopt(long = "lock")]
    lock: bool,

    /// Source whisper file.
    #[structopt(name = "SRC", parse(from_os_str))]
    src: PathBuf,

    /// Destination whisper file.
    #[structopt(name = "DST", parse(from_os_str))]
    dst: PathBuf,
}

// # whisper-fill: unlike whisper-merge, don't overwrite data that's
// # already present in the target file, but instead, only add the missing
// # data (e.g. where the gaps in the target file are).  Because no values
// # are overwritten, no data or precision gets lost.  Also, unlike
// # whisper-merge, try to take the highest-precision archive to provide
// # the data, instead of the one with the largest retention.
// # Using this script, reconciliation between two replica instances can be
// # performed by whisper-fill-ing the data of the other replica with the
// # data that exists locally, without introducing the quite remarkable
// # gaps that whisper-merge leaves behind (filling a higher precision
// # archive with data from a lower precision one)

// # Work performed by Fabian Groffen @grobian while working at Booking.com.
// # additional patches are from https://github.com/jssjr/carbonate/

// try:
//     from whisper import operator
//     HAS_OPERATOR = True
// except ImportError:
//     HAS_OPERATOR = False

// def itemgetter(*items):
//     if HAS_OPERATOR:
//         return operator.itemgetter(*items)
//     else:
//         if len(items) == 1:
//             item = items[0]

//             def g(obj):
//                 return obj[item]
//         else:

//             def g(obj):
//                 return tuple(obj[item] for item in items)
//         return g

// def fill(src, dst, tstart, tstop):
//     # fetch range start-stop from src, taking values from the highest
//     # precision archive, thus optionally requiring multiple fetch + merges
//     srcHeader = whisper.info(src)

//     srcArchives = srcHeader['archives']
//     srcArchives.sort(key=itemgetter('retention'))

//     # find oldest point in time, stored by both files
//     srcTime = int(time.time()) - srcHeader['maxRetention']

//     if tstart < srcTime and tstop < srcTime:
//         return

//     # we want to retain as much precision as we can, hence we do backwards
//     # walk in time

//     # skip forward at max 'step' points at a time
//     for archive in srcArchives:
//         # skip over archives that don't have any data points
//         rtime = time.time() - archive['retention']
//         if tstop <= rtime:
//             continue

//         untilTime = tstop
//         fromTime = rtime if rtime > tstart else tstart

//         (timeInfo, values) = whisper.fetch(src, fromTime, untilTime)
//         (start, end, archive_step) = timeInfo
//         pointsToWrite = list(filter(
//             lambda points: points[1] is not None,
//             zip(xrange(start, end, archive_step), values)))
//         # order points by timestamp, newest first
//         pointsToWrite.sort(key=lambda p: p[0], reverse=True)
//         whisper.update_many(dst, pointsToWrite)

//         tstop = fromTime

//         # can stop when there's nothing to fetch any more
//         if tstart == tstop:
//             return

// def fill_archives(src, dst, startFrom):
//     header = whisper.info(dst)
//     archives = header['archives']
//     archives = sorted(archives, key=lambda t: t['retention'])

//     for archive in archives:
//         fromTime = time.time() - archive['retention']
//         if fromTime >= startFrom:
//             continue

//         (timeInfo, values) = whisper.fetch(dst, fromTime, startFrom)
//         (start, end, step) = timeInfo
//         gapstart = None
//         for v in values:
//             if not v and not gapstart:
//                 gapstart = start
//             elif v and gapstart:
//                 # ignore single units lost
//                 if (start - gapstart) > archive['secondsPerPoint']:
//                     fill(src, dst, gapstart - step, start)
//                 gapstart = None
//             elif gapstart and start == end - step:
//                 fill(src, dst, gapstart - step, start)

//             start += step

//         startFrom = fromTime

// def main():

//         if len(args) != 2:
//                 option_parser.print_help()
//                 sys.exit(1)

//         if options.lock is True and whisper.CAN_LOCK:
//             whisper.LOCK = True

//         src = args[0]
//         dst = args[1]
//         startFrom = time.time()

//         fill_archives(src, dst, startFrom)

fn run(args: &Args) -> Result<(), Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

    fill_archives(&args.src, &args.dst, now, now)?;
    Ok(())
}

fn main() {
    let args = Args::from_args();
    println!("{:?}", args);
    if let Err(err) = run(&args) {
        eprintln!("{}", err);
        exit(1);
    }
}
