use super::constants::{LM_SIDLEN, MAX_LOG_MSG_LENGTH, MSTRACEID_SKIPLIST_HEIGHT, NSTUNSET, NULL};
use std::ffi::{c_char, c_double, c_float, c_int, c_void};
pub type Nstime = i64;
/** @addtogroup encoding-values
@brief Data encoding type defines

These are FDSN-defined miniSEED data encoding values.  The value
of ::MS3Record.encoding is set to one of these.  These values may
be used anywhere and encoding value is needed.

@{ */
#[repr(C)]
pub enum DataEncodingType {
    ///Text encoding (UTF-8)
    Text = 0,
    ///16-bit integer
    Int16 = 1,
    ///32-bit integer
    Int32 = 3,
    ///32-bit float (IEEE)
    Float32 = 4,
    /// 64-bit float (IEEE)
    Float64 = 5,
    /// Steim-1 compressed integers
    Steim1 = 10,
    ///Steim-2 compressed integers
    Steim2 = 11,
    ///[Legacy] GEOSCOPE 24-bit integer
    Geoscope24 = 12,
    /// [Legacy] GEOSCOPE 16-bit gain ranged, 3-bit exponent
    Geoscope163 = 13,
    /// [Legacy] GEOSCOPE 16-bit gain ranged, 4-bit exponent
    CDSN = 16,
    ///[Legacy] SRO 16-bit gain ranged
    SRO = 30,
    ///[Legacy] DWWSSN 16-bit gain ranged
    DWWSSN = 32,
}

/** @enum ms_timeformat_t
   @brief Time format identifiers

   Formats values:
   - \b ISOMONTHDAY - \c "YYYY-MM-DDThh:mm:ss.sssssssss", ISO 8601 in month-day format
   - \b ISOMONTHDAY_Z - \c "YYYY-MM-DDThh:mm:ss.sssssssssZ", ISO 8601 in month-day format with
  trailing Z
   - \b ISOMONTHDAY_DOY - \c "YYYY-MM-DD hh:mm:ss.sssssssss (doy)", ISOMONTHDAY with day-of-year
   - \b ISOMONTHDAY_DOY_Z - \c "YYYY-MM-DD hh:mm:ss.sssssssssZ (doy)", ISOMONTHDAY with day-of-year
  and trailing Z
   - \b ISOMONTHDAY_SPACE - \c "YYYY-MM-DD hh:mm:ss.sssssssss", same as ISOMONTHDAY with space
  separator
   - \b ISOMONTHDAY_SPACE_Z - \c "YYYY-MM-DD hh:mm:ss.sssssssssZ", same as ISOMONTHDAY with space
  separator and trailing Z
   - \b SEEDORDINAL - \c "YYYY,DDD,hh:mm:ss.sssssssss", SEED day-of-year format
   - \b UNIXEPOCH - \c "ssssssssss.sssssssss", Unix epoch value
   - \b NANOSECONDEPOCH - \c "sssssssssssssssssss", Nanosecond epoch value
*/
#[repr(C)]
pub enum ms_timeformat_t {
    IsoMonthDay = 0,
    IsoMonthDayZ = 1,
    IsoMonthDayDOY = 2,
    IsoMonthDayDOYZ = 3,
    IsoMonthDaySpace = 4,
    IsoMonthDaySpaceZ = 5,
    SEEDORDINAL = 6,
    UNIXEPOCH = 7,
    NANOSECONDEPOCH = 8,
}
/** @enum ms_subseconds_t
   @brief Subsecond format identifiers

   Formats values:
   - \b NONE - No subseconds
   - \b MICRO - Microsecond resolution
   - \b NANO - Nanosecond resolution
   - \b MICRO_NONE - Microsecond resolution if subseconds are non-zero, otherwise no subseconds
   - \b NANO_NONE - Nanosecond resolution if subseconds are non-zero, otherwise no subseconds
   - \b NANO_MICRO - Nanosecond resolution if there are sub-microseconds, otherwise microseconds
  resolution
   - \b NANO_MICRO_NONE - Nanosecond resolution if present, microsecond if present, otherwise no
  subseconds
*/
#[repr(C)]
pub enum ms_subseconds_t {
    NONE = 0,
    MICRO = 1,
    NANO = 2,
    MicroNone = 3,
    NanoNone = 4,
    NanoMicro = 5,
    NanoMicroNone = 6,
}
/** @addtogroup miniseed-record
@brief Definitions and functions related to individual miniSEED records
@{ */

/** @brief miniSEED record container */
#[repr(C)]
pub struct MS3Record {
    ///!< Raw miniSEED record, if available
    pub record: *const c_char,
    ///!< Length of miniSEED record in bytes
    pub reclen: i32,
    ///!< Byte swap indicator (bitmask), see @ref byte-swap-flags
    pub swapflag: u8,

    /* Common header fields in accessible form */
    ///!< Source identifier as URN, max length @ref LM_SIDLEN
    pub sid: [c_char; LM_SIDLEN],
    ///!< Format major version
    pub format_version: u8,
    ///!< Record-level bit flags
    pub flags: u8,
    ///!< Record start time (first sample)
    pub starttime: Nstime,
    ///!< Nominal sample rate as samples/second (Hz) or period (s)
    pub samprate: f64,
    ///!< Data encoding format, see @ref encoding-values
    pub encoding: i16,
    ///!< Publication version
    pub pubversion: u8,
    ///!< Number of samples in record
    pub samplecnt: i64,
    ///!< CRC of entire record
    pub crc: u32,
    ///!< Length of extra headers in bytes
    pub extralength: u16,
    ///!< Length of data payload in bytes
    pub datalength: u32,
    ///!< Pointer to extra headers
    pub extra: *mut c_char,

    /* Data sample fields */
    ///!< Data samples, \a numsamples of type \a sampletype
    pub datasamples: *mut c_void,
    ///!< Size of datasamples buffer in bytes
    pub datasize: u64,
    ///!< Number of data samples in datasamples
    pub numsamples: i64,
    ///!< Sample type code: t, i, f, d @ref sample-types
    pub sampletype: c_char,
}
impl MS3Record {
    pub const unsafe fn initializer() -> Self {
        Self {
            record: NULL as *const i8,
            reclen: -1,
            swapflag: 0,
            sid: [0; LM_SIDLEN],
            format_version: 0,
            flags: 0,
            starttime: NSTUNSET,
            samprate: 0.,
            encoding: -1,
            pubversion: 0,
            samplecnt: -1,
            crc: 0,
            extralength: 0,
            datalength: 0,
            extra: NULL as *mut c_char,
            datasamples: NULL as *mut c_void,
            datasize: 0,
            numsamples: 0,
            sampletype: 0,
        }
    }
}
#[repr(C)]
pub struct MS3RecordPacker {
    /// chatgpt claims that the size of the struct is roughly max 100 bytes. It is opaque so it should not matter to much
    buffer: [c_char; 100],
}
///Data selection structure time window definition containers
#[repr(C)]
pub struct MS3SelectTime {
    /// Earliest data for matching channels, use ::NSTUNSET for open
    pub starttime: Nstime,
    ///Latest data for matching channels, use ::NSTUNSET for open
    pub endtime: Nstime,
    /// Pointer to next selection time, NULL if the last
    pub next: *mut MS3SelectTime,
}
///Data selection structure definition containers
#[repr(C)]
pub struct MS3Selections {
    /// Matching (globbing) pattern for source ID
    pub sidpattern: [c_char; 100],
    ///Pointer to time window list for this source ID
    pub timewindows: *mut MS3SelectTime,
    /// Pointer to next selection, NULL if the last
    pub next: *mut MS3Selections,
    /// Selected publication version, use 0 for any
    pubversion: u8,
}
/** @brief A miniSEED record pointer and metadata
 *
 * Used to construct a list of data records that contributed to a
 * trace segment.
 *
 * The location of the record is identified at a memory address (\a
 * bufferptr), the location in an open file (\a fileptr and \a
 * fileoffset), or the location in a file (\a filename and \a
 * fileoffset).
 *
 * A ::MS3Record is stored with and contains the bit flags, extra
 * headers, etc. for the record.
 *
 * The \a dataoffset to the encoded data is stored to enable direct
 * decoding of data samples without re-parsing the header, used by
 * mstl3_unpack_recordlist().
 *
 * Note: the list is stored in the time order that the entries
 * contributed to the segment.
 *
 * @see mstl3_unpack_recordlist()
 */
#[repr(C)]
pub struct MS3RecordPtr {
    /// Pointer in buffer to record, NULL if not used
    pub buffptr: *const c_char,
    /// Pointer to open FILE containing record, NULL if not used
    pub fileptr: *const c_void,
    ///Pointer to file name containing record, NULL if not used
    pub filename: *const c_char,
    /// Offset into file to record for \a fileptr or \a filename
    pub fileoffset: i64,
    ///Pointer to ::MS3Record for this record
    pub msr: *mut MS3Record,
    ///End time of record, time of last sample
    pub endtime: Nstime,
    /// Offset from start of record to encoded data
    pub dataoffset: u32,
    ///Private pointer, will not be populated by library but will be free'd
    pub prvtptr: *mut c_void,
    /// Pointer to next entry, NULL if the last
    pub next: *mut MS3RecordPtr,
}
/// Record list, holds ::MS3RecordPtr entries that contribute to a given ::MS3TraceSeg
#[repr(C)]
pub struct MS3RecordList {
    /// Count of records in the list (for convenience)
    pub recordcnt: u64,
    /// Pointer to first entry, NULL if the none
    pub first: *mut MS3RecordPtr,
    /// Pointer to last entry, NULL if the none
    pub last: *mut MS3RecordPtr,
}
/* @addtogroup trace-list
@brief A container for continuous data

Trace lists are a container to organize continuous segments of
data.  By combining miniSEED data records into trace lists, the
time series is reconstructed and ready for processing, conversion,
summarization, etc.

A trace list container starts with an ::MS3TraceList, which
contains one or more ::MS3TraceID entries, which each contain one
or more ::MS3TraceSeg entries.  The ::MS3TraceID and ::MS3TraceSeg
entries are easily traversed as linked structures.

The overall structure is illustrated as:
  - MS3TraceList
    - MS3TraceID
      - MS3TraceSeg
      - MS3TraceSeg
      - ...
    - MS3TraceID
      - MS3TraceSeg
      - MS3TraceSeg
      - ...
    - ...

@note A trace list does not contain all of the details of a miniSEED
record.  In particular details that are not relevant to represent the series
such as header flags, extra headers like event detections, etc.

\sa ms3_readtracelist()
\sa ms3_readtracelist_timewin()
\sa ms3_readtracelist_selection()
\sa mstl3_writemseed()
@{ */

/// Container for a continuous trace segment, linkable
#[repr(C)]
pub struct MS3TraceSeg {
    ///Time of first sample
    pub starttime: Nstime,
    ///Time of last sample
    pub endtime: Nstime,
    ///Nominal sample rate (Hz)
    pub samprate: c_double,
    /// Number of samples in trace coverage
    pub samplecnt: i64,
    /// Data samples, \a numsamples of type \a sampletype
    pub datasamples: *mut c_void,
    ///< Size of datasamples buffer in bytes
    pub datasize: u64,
    /// Number of data samples in datasamples
    pub numsamples: i64,
    ///Sample type code, see @ref sample-types
    pub sampletype: c_char,
    /// Private pointer for general use, unused by library unless ::MSF_PPUPDATETIME is set
    pub prvptr: *mut c_void,
    /// List of pointers to records that contributed
    pub recordlist: *mut MS3RecordList,
    /// Pointer to previous segment
    pub prev: *mut MS3TraceSeg,
    /// Pointer to next segment, NULL if the last
    pub next: *mut MS3TraceSeg,
}
/// Container for a trace ID, linkable
#[repr(C)]
pub struct MS3TraceID {
    /// Source identifier as URN, max length @ref LM_SIDLEN
    pub sid: [c_char; LM_SIDLEN],
    ///  Largest contributing publication version
    pub pubversion: u8,
    /// Time of earliest sample
    pub earliest: Nstime,
    /// Time of latest sample
    pub latest: Nstime,
    /// Private pointer for general use, unused by library
    pub prvtptr: *mut c_void,
    /// Number of segments for this ID
    pub numsegments: u32,
    /// Pointer to first of list of segments
    pub first: *mut MS3TraceSeg,
    /// Pointer to last of list of segments
    pub last: *mut MS3TraceSeg,
    /// Next trace ID at first pointer, NULL if the last
    pub next: [*mut MS3TraceID; MSTRACEID_SKIPLIST_HEIGHT],
    /// Height of skip list at \a next
    height: u8,
}
/// Container for a collection of continuous trace segment, linkable
#[repr(C)]
pub struct MS3TraceList {
    /// Number of traces IDs in list
    pub numtraceids: u32,
    /// Head node of trace skip list, first entry at \a traces.next[0]
    pub traces: MS3TraceID,
    /// INTERNAL: State for Pseudo RNG
    pub prngstate: u64,
}
/** @brief Callback functions that return time and sample rate tolerances
 *
 * A container for function pointers that return time and sample rate
 * tolerances that are used for merging data into ::MS3TraceList
 * containers. The functions are provided with a ::MS3Record and must
 * return the acceptable tolerances to merge this with other data.
 *
 * The \c time(MS3Record) function must return a time tolerance in seconds.
 *
 * The \c samprate(MS3Record) function must return a sampling rate tolerance in Hertz.
 *
 * For any function pointer set to NULL a default tolerance will be used.
 *
 * Illustrated usage:
 * @code
 * MS3Tolerance tolerance;
 *
 * tolerance.time = my_time_tolerance_function;
 * tolerance.samprate = my_samprate_tolerance_function;
 *
 * mstl3_addmsr (mstl, msr, 0, 1, &tolerance);
 * @endcode
 *
 * \sa mstl3_addmsr()
 */
#[repr(C)]
pub struct MS3Tolerance {
    ///Pointer to function that returns time tolerance
    pub time: *mut ToleranceHandler,
    /// Pointer to function that returns sample rate tolerance
    pub samprate: *mut ToleranceHandler,
}
pub type ToleranceHandler = extern "C" fn(*const MS3Record) -> c_double;
impl MS3Tolerance {
    pub const unsafe fn initializer() -> Self {
        Self {
            time: NULL as *mut ToleranceHandler,
            samprate: NULL as *mut ToleranceHandler,
        }
    }
}
#[repr(C)]
pub enum LIMOType {
    ///< IO handle type is undefined
    LmioNull = 0,
    ///  IO handle is FILE-type
    LmioFile = 1,
    /// IO handle is URL-type
    LmioUrl = 2,
    /// IO handle is a provided file descriptor
    LmioFd = 3,
}
/// Type definition for data source I/O: file-system versus URL
#[repr(C)]
pub struct LMIO {
    /// IO handle type
    pub ty: LIMOType,
    ///  Primary IO handle, either file or URL
    pub handle: *mut c_void,
    /// Secondary IO handle for URL
    pub handle2: *mut c_void,
    /// Fetch status flag for URL transmission
    pub still_running: c_int,
}
impl LMIO {
    pub const unsafe fn initializer() -> Self {
        Self {
            ty: LIMOType::LmioNull,
            handle: NULL as *mut c_void,
            handle2: NULL as *mut c_void,
            still_running: 0,
        }
    }
}
/** State container for reading miniSEED records from files or URLs.

    In general these values should not be directly set or accessed.  It is
    possible to allocate a structure and set the \c path, \c startoffset,
    and \c endoffset values for advanced usage.  Note that file/URL start
    and end offsets can also be parsed from the path name as well.
*/
#[repr(C)]
pub struct MS3FileParam {
    ///INPUT: File name or URL
    pub path: [c_char; 512],
    /// INPUT: Start position in input stream
    pub startoffset: i64,
    /// INPUT: End position in input stream, 0 == unknown (e.g. pipe)
    pub endoffset: i64,
    ///  OUTPUT: Read position of input stream
    pub streampos: i64,
    ///  OUTPUT: Count of records read from this stream/file so far
    pub recordcount: i64,
    /// INTERNAL: Read buffer, allocated internally
    pub readbuffer: *mut c_char,
    ///  INTERNAL: Length of data in read buffer
    pub readlength: c_int,
    /// INTERNAL: Read offset in read buffer
    pub readoffset: c_int,
    /// INTERNAL: Stream reading state flags
    pub flags: u32,
    /// INTERNAL: IO handle, file or URL
    pub input: LMIO,
}
impl MS3FileParam {
    pub const unsafe fn initializer() -> Self {
        unsafe {
            Self {
                path: [0; 512],
                startoffset: 0,
                endoffset: 0,
                streampos: 0,
                recordcount: 0,
                readbuffer: NULL as *mut i8,
                readlength: 0,
                readoffset: 0,
                flags: 0,
                input: LMIO::initializer(),
            }
        }
    }
}
/**
 * @brief Container for event detection parameters for use in extra headers
 *
 * Actual values are optional, with special values indicating an unset
 * state.
 *
 * @see mseh_add_event_detection_r
 */
#[repr(C)]
pub struct MSEHEventDetection {
    /// Detector type (e.g. "MURDOCK"), zero length = not included
    pub ty: [c_char; 30],
    /// Detector name, zero length = not included
    pub detector: [c_char; 30],
    /// SignalAmplitude, 0.0 = not included
    pub signalapliitude: c_double,
    /// Signal period, 0.0 = not included
    pub signalperiod: c_double,
    /// Background estimate, 0.0 = not included
    pub bagroundestimate: c_double,
    /// Detection wave (e.g. "DILATATION"), zero length = not included
    pub wave: [c_char; 30],
    ///Units of amplitude and background estimate (e.g. "COUNTS"), zero length = not included
    pub units: [c_char; 30],
    ///  Onset time, NSTUNSET = not included
    pub onsettime: Nstime,
    /// Signal to noise ratio for Murdock event detection, all zeros = not included
    pub medsnr: [u8; 6],
    ///  Murdock event detection lookback value, -1 = not included
    pub medlookback: c_int,
    /// Murdock event detection pick algoritm, -1 = not included
    pub medpickalgorithm: c_int,
    /// Pointer to next, NULL if none
    pub next: *mut MSEHEventDetection,
}
/**
 * @brief Container for calibration parameters for use in extra headers
 *
 * Actual values are optional, with special values indicating an unset
 * state.
 *
 * @see mseh_add_calibration
 */
#[repr(C)]
pub struct MSEHCalibration {
    ///Calibration type  (e.g. "STEP", "SINE", "PSEUDORANDOM"), zero length = not included
    pub ty: [c_char; 30],
    /// Begin time, NSTUNSET = not included
    pub begintime: Nstime,
    /// End time, NSTUNSET = not included
    pub endtime: Nstime,
    /// Number of step calibrations, -1 = not included
    pub steps: c_int,
    /// Boolean, step cal. first pulse, -1 = not included
    pub firstpulsepositive: c_int,
    /// Boolean, step cal. alt. sign, -1 = not included
    pub alternatesign: c_int,
    /// Trigger, e.g. AUTOMATIC or MANUAL, zero length = not included
    pub trigger: [c_char; 30],
    ///Boolean, continued from prev. record, -1 = not included
    pub continued: c_int,
    /// Amp. of calibration signal, 0.0 = not included
    pub amplitude: c_double,
    /// Units of input (e.g. volts, amps), zero length = not included
    pub inputunits: [c_char; 30],
    /// E.g PEAKTOPTEAK, ZEROTOPEAK, RMS, RANDOM, zero length = not included
    pub amplituderange: [c_char; 30],
    /// Duration in seconds, 0.0 = not included
    pub duration: c_double,
    /// Period of sine, 0.0 = not included
    pub sineperiod: c_double,
    /// Interval bewteen steps, 0.0 = not included
    pub stepbetween: c_double,
    /// Channel of input, zero length = not included
    pub inputchannel: [c_char; 30],
    /// Reference amplitude, 0.0 = not included
    pub refamplitude: c_double,
    /// Coupling, e.g. Resistive, Capacitive, zero length = not included
    pub coupling: [c_char; 30],
    /// Rolloff of filters, zero length = not included
    pub rolloff: [c_char; 30],
    /// Noise for PR cals, e.g. White or Red, zero length = not included
    pub noise: [c_char; 30],
    /// Pointer to next, NULL if none
    pub next: *mut MSEHCalibration,
}
/**
 * @brief Container for timing exception parameters for use in extra headers
 *
 * Actual values are optional, with special values indicating an unset
 * state.
 *
 * @see mseh_add_timing_exception
 */
#[repr(C)]
pub struct MSEHTimingException {
    /// Time of exception, NSTUNSET = not included
    pub time: Nstime,
    /// VCO correction, from 0 to 100%, <0 = not included
    pub vcocorrection: c_float,
    ///[DEPRECATED] microsecond time offset, 0 = not included
    pub usec: c_int,
    /// Reception quality, 0 to 100% clock accurracy, <0 = not included
    pub receptionquality: c_int,
    /// The count thereof, 0 = not included
    pub count: u32,
    /// E.g. "MISSING" or "UNEXPECTED", zero length = not included
    pub ty: [c_char; 16],
    /// Description of clock-specific parameters, zero length = not included
    pub clockstatus: [c_char; 128],
}
/**
 * @brief Container for recenter parameters for use in extra headers
 *
 * Actual values are optional, with special values indicating an unset
 * state.
 *
 * @see mseh_add_recenter
 */
#[repr(C)]
pub struct MSEHRecenter {
    ///Recenter type  (e.g. "MASS", "GIMBAL"), zero length = not included
    pub ty: [c_char; 30],
    ///Begin time, NSTUNSET = not included
    pub negintime: Nstime,
    ///Estimated end time, NSTUNSET = not included
    pub endtime: Nstime,
    /// Trigger, e.g. AUTOMATIC or MANUAL, zero length = not included
    pub trigger: [c_char; 30],
}

/**
 * @brief Internal structure for holding parsed JSON extra headers.
 * @see mseh_get_ptr_r()
 * @see mseh_set_ptr_r()
 */

pub type LmParsedJson = c_void;
/** @brief Log registry entry.
\sa ms_rlog()
\sa ms_rlog_l() */
#[repr(C)]
pub struct MSLogEntry {
    /// Message level
    pub level: c_int,
    /// Function generating the message
    pub function: [c_char; 30],
    /// Log, warning or error message
    pub message: [c_char; MAX_LOG_MSG_LENGTH],
    pub next: *mut MSLogEntry,
}
/** @brief Log message registry.
\sa ms_rlog()
\sa ms_rlog_l() */
#[repr(C)]
pub struct MSLogRegistry {
    pub maxmessages: c_int,
    pub messagecnt: c_int,
    pub messages: *mut MSLogEntry,
}
impl MSLogRegistry {
    pub const unsafe fn initializer() -> Self {
        Self {
            maxmessages: 0,
            messagecnt: 0,
            messages: NULL as *mut MSLogEntry,
        }
    }
}
/** @brief Logging parameters.
__Callers should not modify these values directly and generally
should not need to access them.__

\sa ms_loginit() */
#[repr(C)]
pub struct MSLogParam {
    /// Function to call for regular messages
    pub log_print: *mut LogCallback,
    ///Message prefix for regular and diagnostic messages
    pub logprefix: *const c_char,
    /// Function to call for diagnostic and error messages
    pub diag_print: *mut LogCallback,
    ///Message prefix for error messages
    pub errprefix: *const c_char,
    ///Message registry
    pub registry: MSLogRegistry,
}
impl MSLogParam {
    /** @def MSLogParam_INITIALIZER
    @brief Initialializer for ::MSLogParam */
    pub const unsafe fn initializer() -> Self {
        unsafe {
            Self {
                log_print: NULL as *mut LogCallback,
                logprefix: NULL as *const c_char,
                diag_print: NULL as *mut LogCallback,
                errprefix: NULL as *const c_char,
                registry: MSLogRegistry::initializer(),
            }
        }
    }
}
pub type LogCallback = extern "C" fn(*const c_char);
/** @brief Leap second list container */
#[repr(C)]
pub struct LeapSecond {
    ///Time of leap second as epoch since 1 January 1900
    pub leapsecond: Nstime,
    ///TAI-UTC difference in seconds
    pub tai_delta: i32,
    ///Pointer to next entry, NULL if the last
    pub next: *mut LeapSecond,
}
/**
# Parsing, packing and trace construction control flags

These are bit flags that can be combined into a bitmask to control
aspects of the library's parsing, packing and trace managment routines.
*/
pub enum ControlFlags {
    ///[Parsing] Unpack data samples
    UnpackData = 0x1,
    ///[Parsing] Skip input that cannot be identified as miniSEED
    SkipNoData = 0x2,
    /// [Parsing] Validate CRC (if version 3)
    ValidateCrc = 0x4,
    ///[Parsing] Parse and utilize byte range from path name suffix
    PNameRange = 0x8,
    /// [Parsing] Reading routine is at the end of the file
    AtEndOfFile = 0x10,
    /// [Packing] UNSUPPORTED: Maintain a record-level sequence number
    Sequence = 0x20,
    ///[Packing] Pack all available data even if final record would not be filled
    FlushData = 0x40,
    /// [Packing] Pack as miniSEED version 2 instead of 3
    PackVersion2 = 0x80,
    /// [TraceList] Build a ::MS3RecordList for each ::MS3TraceSeg
    RecordList = 0x100,
    ///  [TraceList] Do not modify a trace list when packing
    MaintainMstl = 0x200,
    ///  [TraceList] Store update time (as nstime_t) at ::MS3TraceSeg.prvtptr
    PPUpdateTime = 0x400,
    /// [TraceList] Use the splitversion value as version instead of record version
    SplitSVersion = 0x800,
    /// [TraceList] Skip adjacent duplicate records
    SkipAdjacentDuplicates = 0x1000,
}
pub enum DataSampleType {
    /// Text data samples
    Text = 't' as isize,
    /// 32-bit integer data samples
    Int32 = 'i' as isize,
    /// 32-bit float (IEEE) data samples
    Float32 = 'f' as isize,
    /// 64-bit float (IEEE) data samples
    Float64 = 'd' as isize,
}
