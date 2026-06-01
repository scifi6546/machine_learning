use super::{
    constants::NULL,
    structs::{
        LmParsedJson, LogCallback, MS3FileParam, MS3Record, MS3RecordPacker, MS3RecordPtr,
        MS3SelectTime, MS3Selections, MS3Tolerance, MS3TraceID, MS3TraceList, MS3TraceSeg,
        MSEHCalibration, MSEHEventDetection, MSEHRecenter, MSEHTimingException, MSLogParam, Nstime,
        ms_subseconds_t, ms_timeformat_t,
    },
};

use libc::FILE;
use std::ffi::{c_char, c_double, c_int, c_void};
pub type RecordHandler = extern "C" fn(*mut c_char, i32, *mut c_void);
unsafe extern "C" {
    pub unsafe fn ms_nstime2time(
        nstime: Nstime,
        year: *mut u16,
        yday: *mut u16,
        hour: *mut u8,
        min: *mut u8,
        sec: *mut u8,
        nsec: *mut u32,
    ) -> c_int;
    pub unsafe fn ms_nstime2timestr_n(
        nstime: Nstime,
        timestr: *mut c_char,
        timestrsize: usize,
        timeformat: ms_timeformat_t,
        subsecond: ms_subseconds_t,
    ) -> *mut c_char;

    pub unsafe fn ms_time2nstime(
        year: c_int,
        yday: c_int,
        hour: c_int,
        min: c_int,
        sec: c_int,
        nsec: u32,
    ) -> Nstime;
    pub unsafe fn ms_timestr2nstime(timestr: *const c_char) -> Nstime;
    pub unsafe fn ms_mdtimestr2nstime(timestr: *const c_char) -> Nstime;
    pub unsafe fn ms_seedtimestr2nstime(sweedtimestr: *const c_char) -> Nstime;
    pub unsafe fn ms_doy2md(year: c_int, yday: c_int, month: *mut c_int, mday: *mut c_int)
    -> c_int;
    pub unsafe fn ms_md2doy(year: c_int, month: c_int, mday: c_int, yday: *mut c_int) -> c_int;
    /** Parse miniSEED from a buffer.

    This routine will attempt to parse (detect and unpack) a miniSEED record from a specified memory buffer and populate a supplied MS3Record structure. Both miniSEED 2.x and 3.x records are supported.

    The record length is automatically detected. For miniSEED 2.x this means the record must contain a 1000 blockette.
    ### parameters:

     */
    pub unsafe fn msr3_parse(
        record: *const c_char,
        recbuflen: u64,
        ppmsr: *mut *mut MS3Record,
        flags: u32,
        verbose: u8,
    ) -> c_int;
    pub unsafe fn msr3_pack(
        record: *const MS3Record,
        record_handler: *mut RecordHandler,
        handler_data: *mut c_void,
        packed_samples: *mut i64,
        flags: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn msr3_pack_init(
        msr: *const MS3Record,
        flags: u32,
        verbose: i8,
    ) -> *mut MS3RecordPacker;
    pub unsafe fn ms3_pack_next(
        packer: *mut MS3RecordPacker,
        record: *mut *mut c_char,
        reclen: *mut i32,
    ) -> c_int;
    pub unsafe fn msr3_pack_free(packer: *mut *mut MS3RecordPacker, packed_samples: *mut i64);
    pub unsafe fn msr3_repack_mseed3(
        msr: *const MS3Record,
        record: *mut c_char,
        recbuflen: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn msr3_repack_mseed2(
        msr: *const MS3Record,
        record: *mut c_char,
        recbuflen: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn ms3_pack_header3(
        msr: *const MS3Record,
        record: *mut c_char,
        recbuflen: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn ms2_pack_header3(
        msr: *const MS3Record,
        record: *mut c_char,
        recbuflen: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn msr3_unpack_data(msr: *mut MS3Record, verbose: i8) -> i64;
    pub unsafe fn msr3_data_bounds(
        msr: *const MS3Record,
        dataoffset: *mut u32,
        datasize: *mut u32,
    ) -> c_int;
    pub unsafe fn ms_decode_data(
        input: *const c_void,
        inputsize: u64,
        encoding: u8,
        samplecount: u64,
        output: *mut c_void,
        outputsize: u64,
        sampletype: *mut c_char,
        swapflag: i8,
        sid: *const c_char,
        verbose: i8,
    ) -> i64;
    pub unsafe fn msr3_init(msr: *mut MS3Record) -> *mut MS3Record;
    pub unsafe fn msr3_free(ppmsr: *mut *mut MS3Record);
    pub unsafe fn msr3_duplicate(msr: *const MS3Record, datadup: i8) -> *mut MS3Record;
    pub unsafe fn ms3r_endtime(msr: *const MS3Record) -> Nstime;
    pub unsafe fn ms3r_print(msr: *const MS3Record, details: i8);
    pub unsafe fn ms3r_resize_buffer(msr: *mut MS3Record) -> c_void;
    pub unsafe fn msr3_sampratehz(msr: *const MS3Record) -> c_double;
    pub unsafe fn msr3_nsperiod(msr: *const MS3Record) -> Nstime;
    pub unsafe fn msr3_host_latency(msr: *const MS3Record) -> c_double;
    pub unsafe fn ms3_detect(record: *const c_char, recbuflen: u64, formatversion: *mut u8) -> i64;
    pub unsafe fn ms_parse_raw3(record: *const c_char, maxreclen: c_int, details: i8) -> c_int;
    pub unsafe fn ms_parse_raw2(
        record: *const c_char,
        maxreclen: c_int,
        details: i8,
        swapflag: i8,
    ) -> c_int;

    /* @addtogroup data-selections
    @brief Data selections to be used as filters

    Selections are the identification of data, by source identifier
    and time ranges, that are desired.  Capability is included to read
    selections from files and to match data against a selection list.

    For data to be selected it must only match one of the selection
    entries.  In other words, multiple selection entries are treated
    with OR logic.

    The ms3_readmsr_selection() and ms3_readtracelist_selection()
    routines accept ::MS3Selections and allow selective (and
    efficient) reading of data from files.
    @{ */
    pub unsafe fn ms3_matchselect(
        selections: *const MS3Selections,
        sid: *const c_char,
        starttime: Nstime,
        endtime: Nstime,
        pubversion: c_int,
        ppselecttime: *const *const MS3SelectTime,
    ) -> *const MS3Selections;
    pub unsafe fn ms3r_match_select(
        selections: *const MS3Selections,
        msr: *const MS3Record,
        ppselecttime: *const *const MS3SelectTime,
    ) -> *const MS3Selections;
    pub unsafe fn ms3_addselect(
        ppselections: *mut *mut MS3Selections,
        sidpattern: *const c_char,
        starttime: Nstime,
        endtime: Nstime,
        pubversion: u8,
    ) -> c_int;
    pub unsafe fn ms3_addselect_comp(
        ppselections: *mut *mut MS3Selections,
        network: *mut c_char,
        station: *mut c_char,
        location: *mut c_char,
        channel: *mut c_char,
        starttime: Nstime,
        endtime: Nstime,
        pubversion: u8,
    ) -> c_int;
    pub unsafe fn ms3_readselectionsfile(
        ppselections: *mut *mut MS3Selections,
        filename: *const c_char,
    ) -> c_int;
    pub unsafe fn ms3_free_selections(selections: *mut MS3Selections);
    pub unsafe fn ms3_printselections(selections: *const MS3Selections);
    /* @brief Callback functions that return time and sample rate tolerances
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
    pub unsafe fn mstl3_init(mstl: *mut MS3TraceList) -> *mut MS3TraceList;
    pub unsafe fn mstl3_free(ppmstl: *mut *mut MS3TraceList, freeprvtptr: i8);
    pub unsafe fn mstl3_findID(
        mstl: *mut MS3TraceList,
        sid: *const c_char,
        pubversion: u8,
        prev: *mut *mut MS3TraceID,
    ) -> MS3TraceID;
    pub unsafe fn mstl3_addmsr(
        mstl: *mut MS3TraceList,
        msr: *const MS3Record,
        splitversion: i8,
        autoheal: i8,
        flags: u32,
        tolerance: *const MS3Tolerance,
    ) -> *mut MS3TraceSeg;
    pub unsafe fn mstl3_addmsr_recordptr(
        mstl: *mut MS3TraceList,
        msr: *const MS3Record,
        pprecptr: *mut *mut MS3RecordPtr,
        splitversion: i8,
        autoheal: i8,
        flags: u32,
        tolerance: *const MS3Tolerance,
    ) -> *mut MS3TraceSeg;
    pub unsafe fn mstl3_readbuffer(
        ppmstl: *mut *mut MS3TraceList,
        buffer: *const c_char,
        bufferlength: u64,
        splitversion: i8,
        flags: u32,
        tolerance: *const MS3Tolerance,
        verbose: i8,
    ) -> i64;
    pub unsafe fn mstl3_readbuffer_selection(
        ppmstl: *mut *mut MS3TraceList,
        buffer: *const c_char,
        bufferlength: u64,
        splitversion: i8,
        flags: u32,
        tolerance: *const MS3Tolerance,
        selections: *const MS3Selections,
        verbose: i8,
    ) -> i64;
    pub unsafe fn mstl3_unpack_recordlist(
        id: *mut MS3TraceID,
        seg: *mut MS3TraceSeg,
        output: *mut c_void,
        outputsize: u64,
        verbose: i8,
    ) -> i64;
    pub unsafe fn mstl3_convert_samples(seg: *mut MS3TraceSeg, ty: c_char, truncate: i8) -> c_int;
    pub unsafe fn mstl3_resize_buffers(mstl: *mut MS3TraceList) -> c_int;

    pub unsafe fn mstl3_pack(
        mstl: *mut MS3TraceList,
        record_handler: *mut RecordHandler,
        handlerdata: *mut c_void,
        reclen: c_int,
        encoding: i8,
        packedsamples: *mut i64,
        flags: u32,
        verbose: i8,
        extra: *mut c_char,
    ) -> i64;
    pub unsafe fn mstl3_printtracelist(
        mstl: *const MS3TraceList,
        timeformat: ms_timeformat_t,
        details: i8,
        gaps: i8,
        versions: i8,
    );
    pub unsafe fn mstl3_printsynclist(
        mstl: *const MS3TraceList,
        dccid: *const c_char,
        subseconds: ms_subseconds_t,
    );
    pub unsafe fn mstl3_printgaplist(
        mstl: *const MS3TraceList,
        timeformat: ms_timeformat_t,
        mingap: *mut c_double,
        maxgap: *mut c_double,
    );
    /* @addtogroup io-functions
     @brief Reading and writing interfaces for miniSEED to/from files or URLs

     The miniSEED reading interfaces read from either regular files or
     URLs (if optional support is included).  The miniSEED writing
     interfaces write to regular files.

     URL support for reading is included by building the library with the
     \b LIBMSEED_URL variable defined. URL path-specified resources can only be
     read, e.g. HTTP GET requests.  More advanced POST or form-based requests are
     not supported.

     The function @ref libmseed_url_support() can be used as a run-time test
     to determine if URL support is included in the library.

     Some parameters can be set that affect the reading of data from URLs, including:
     - set the User-Agent header with @ref ms3_url_useragent()
     - set username and password for authentication with @ref ms3_url_userpassword()
     - set arbitrary headers with @ref ms3_url_addheader()
     - disable TLS/SSL peer and host verficiation by setting \b LIBMSEED_SSL_NOVERIFY environment
    variable

     Diagnostics: Setting environment variable \b LIBMSEED_URL_DEBUG enables detailed verbosity of
    URL protocol exchanges.

     \sa ms3_readmsr()
     \sa ms3_readmsr_selection()
     \sa ms3_readtracelist()
     \sa ms3_readtracelist_selection()
     \sa msr3_writemseed()
     \sa mstl3_writemseed()
     @{ */
    pub unsafe fn ms3_readmsr(
        ppmsr: *mut *mut MS3Record,
        mspath: *const c_char,
        flags: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn ms3_readmsr_r(
        ppmsfp: *mut *mut MS3FileParam,
        ppmsr: *mut *mut MS3Record,
        mspath: *const c_char,
        flags: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn ms3_readmsr_selection(
        ppmsfp: *mut *mut MS3FileParam,
        ppmsr: *mut *mut MS3Record,
        mspath: *const c_char,
        flags: u32,
        selections: *const MS3Selections,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn ms3_readtracelist(
        ppmstl: *mut *mut MS3TraceList,
        mspath: *const c_char,
        tolerance: *const MS3Tolerance,
        splitversion: i8,
        flags: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn ms3_read_tracelist_timewin(
        ppmstl: *mut *mut MS3TraceList,
        mspath: *const c_char,
        tolerance: *const MS3Tolerance,
        starttime: Nstime,
        endtime: Nstime,
        splitversion: i8,
        flags: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn ms3_readtracelist_selection(
        ppmstl: *mut *mut MS3TraceList,
        mspath: *const c_char,
        tolerance: *const MS3Tolerance,
        selections: *const MS3Selections,
        split_version: i8,
        flags: u32,
        verbose: i8,
    ) -> c_int;
    pub unsafe fn ms3_url_useragent(program: *const c_char, version: *const c_char) -> c_int;
    pub unsafe fn ms3_url_userpassword(userpassword: *const c_char) -> c_int;
    pub unsafe fn ms3_url_addheader(header: *const c_char) -> c_int;
    pub unsafe fn ms3_url_freeheaders();
    pub unsafe fn msr3_writemseed(
        msr: *mut MS3Record,
        mspath: *const c_char,
        overwrite: i8,
        flags: u32,
        verbose: i8,
    ) -> i64;
    pub unsafe fn mstl3_writemseed(
        mstl: *mut MS3TraceList,
        mspath: *const c_char,
        overwrite: i8,
        maxreclen: c_int,
        encoding: i8,
        flags: u32,
        verbose: i8,
    ) -> i64;
    pub unsafe fn libmseed_url_support() -> c_int;
    pub unsafe fn ms3_msfp_int_fd(fd: c_int) -> *mut MS3FileParam;
    /* @addtogroup string-functions
    @brief Source identifier (SID) and string manipulation functions

    A source identifier uniquely identifies the generator of data in a
    record.  This is a small string, usually in the form of a URI.
    For data identified with FDSN codes, the SID is usally a simple
    combination of the codes.

    @{ */
    pub unsafe fn ms_sid2nslc_n(
        sid: *const c_char,
        net: *mut c_char,
        netsize: isize,
        sta: *mut c_char,
        stasize: isize,
        loc: *mut c_char,
        locsize: isize,
        chan: *mut c_char,
        chansize: isize,
    ) -> c_int;
    pub unsafe fn ms_nslc2sid(
        sid: *mut c_char,
        sidlen: c_int,
        flags: u16,
        net: *const c_char,
        sta: *const c_char,
        loc: *const c_char,
        chan: *const c_char,
    ) -> c_int;
    pub unsafe fn ms_seedchan2xchan(xchan: *mut c_char, seedchan: *const c_char) -> c_int;
    pub unsafe fn ms_xchan2seedchan(seedchan: *mut c_char, xchan: *const c_char) -> c_int;
    pub unsafe fn ms_strncplean(dest: *mut c_char, source: *mut c_char, length: c_int) -> c_int;
    pub unsafe fn ms_strncpleantail(
        dest: *mut c_char,
        source: *const c_char,
        length: c_int,
    ) -> c_int;
    pub unsafe fn ms_strncpopen(dest: *mut c_char, source: *const c_char, length: c_int) -> c_int;
    /* @addtogroup extra-headers
    @brief Structures and funtions to support extra headers

    Extra headers are stored as JSON within a data record header using
    an anonymous, root object as a container for all extra headers.
    For a full description consult the format specification.

    The library functions supporting extra headers allow specific
    header identification using JSON Pointer identification.  In this
    notation each path element is an object until the final element
    which is a key to specified header value.

    For example, a \a path specified as:
    \code
    "/objectA/objectB/header"
    \endcode

    would correspond to the single JSON value in:
    \code
    {
       "objectA": {
         "objectB": {
           "header":VALUE
          }
       }
    }
    \endcode
    @{ */

    pub unsafe fn mseh_get_ptr_type(
        msr: *const MS3Record,
        ptr: *const c_char,
        parsestate: *mut *mut LmParsedJson,
    ) -> c_int;
    pub unsafe fn mseh_get_ptr_r(
        msr: *const MS3Record,
        ptr: *const c_char,
        value: *mut c_void,
        ty: c_char,
        max_length: u32,
        parsestate: *mut *mut LmParsedJson,
    ) -> c_int;
    pub unsafe fn mseh_set_ptr_r(
        msr: *mut MS3Record,
        ptr: *const c_char,
        value: *mut c_void,
        ty: c_char,
        parsestate: *mut *mut LmParsedJson,
    ) -> c_int;
    pub unsafe fn mseh_add_event_detection_r(
        msr: *mut MS3Record,
        ptr: *const c_char,
        eventdetection: *mut MSEHEventDetection,
        parsestate: *mut *mut LmParsedJson,
    ) -> c_int;
    pub unsafe fn mseh_add_calibration_r(
        msr: *mut MS3Record,
        ptr: *const c_char,
        calibration: *mut MSEHCalibration,
        parsestate: *mut *mut LmParsedJson,
    ) -> c_int;
    pub unsafe fn mseh_add_timing_exception_r(
        msr: *mut MS3Record,
        ptr: *const c_char,
        exception: *mut MSEHTimingException,
        parsestate: *mut *mut LmParsedJson,
    ) -> c_int;
    pub unsafe fn mseh_add_recenter_r(
        msr: *mut MS3Record,
        ptr: *const c_char,
        recenter: *mut MSEHRecenter,
        parsestate: *mut *mut LmParsedJson,
    ) -> c_int;
    pub unsafe fn mseh_serialize(msr: *mut MS3Record, parsestate: *mut *mut LmParsedJson) -> c_int;
    pub unsafe fn mseh_free_parsestate(parsestate: *mut *mut LmParsedJson);
    pub unsafe fn mseh_print(msr: *mut MS3Record, indent: c_int);
    /* @addtogroup record-list
        @brief Functionality to build a list of records that contribute to a ::MS3TraceSeg

        As a @ref trace-list is constructed from data records, a list of
        the records that contribute to each segment can be built by using
        the ::MSF_RECORDLIST flag to @ref mstl3_readbuffer() and @ref
        ms3_readtracelist().  Alternatively, a record list can be built by
        adding records to a @ref trace-list using mstl3_addmsr_recordptr().

        The main purpose of this functionality is to support an efficient,
        2-pass pattern of first reading a summary of data followed by
        unpacking the samples.  The unpacking can be performed selectively
        on desired segments and optionally placed in a caller-supplied
        buffer.

        The @ref mstl3_unpack_recordlist() function allows for the
        unpacking of data samples for a given ::MS3TraceSeg into a
        caller-specified buffer, or allocating the buffer if needed.

        \sa mstl3_readbuffer()
        \sa mstl3_readbuffer_selection()
        \sa ms3_readtracelist()
        \sa ms3_readtracelist_selection()
        \sa mstl3_unpack_recordlist()
        \sa mstl3_addmsr_recordptr()
    */

    /* @addtogroup logging
    @brief Central logging functions for the library and calling programs

    This central logging facility is used for all logging performed by
    the library.  Calling programs may also wish to log messages via
    the same facility for consistency.

    The logging can be configured to send messages to arbitrary
    functions, referred to as \c log_print() and \c diag_print().
    This allows output to be re-directed to other logging systems if
    needed.

    It is also possible to assign prefixes to log messages for
    identification, referred to as \c logprefix and \c errprefix.

    @anchor logging-levels
    Logging levels
    --------------

    Three message levels are recognized:
    - 0 : Normal log messages, printed using \c log_print() with \c logprefix
    - 1  : Diagnostic messages, printed using \c diag_print() with \c logprefix
    - 2+ : Error messages, printed using \c diag_print() with \c errprefix

    It is the task of the ms_rlog() and ms_rlog_l() functions to
    format a message using printf conventions and pass the formatted
    string to the appropriate printing function.  The convenience
    macros ms_log() and ms_log_l() can be used to automatically set
    the calling function name.

    @anchor log-registry
    Log Registry
    ------------

    The log registry facility allows a calling program to disable
    error (and warning) output from the library and either inspect it
    or emit (print) as desired.

    By default log messages are sent directly to the printing
    functions.  Optionally, **error and warning messages** (levels 1
    and 2) can be accumulated in a log-registry.  Verbose output
    messages (level 0) are not accumulated in the registry.  The
    registry is enabled by setting the \c maxmessages argument of
    either ms_rloginit() or ms_rloginit_l().  Messages can be emitted,
    aka printed, using ms_rlog_emit() and cleared using
    ms_rlog_free().  Alternatively, the ::MSLogRegistry associated
    with a ::MSLogParam (or the global parameters at \c gMSLogParam).

    See \ref example-mseedview for a simple example of error and
    warning message registry usage.

    @anchor log-threading
    Logging in Threads
    ------------------

    By default the library is compiled in a mode where each thread of
    a multi-threaded program will have it's own, default logging
    parameters.  __If you wish to change the default printing
    functions, message prefixes, or enable the log registry, this must
    be done per-thread.__

    The library can be built with the \b LIBMSEED_NO_THREADING
    variable defined, resulting in a mode where there are global
    parameters for all threads.  In general this should not be used
    unless the system does not support the necessary thread-local
    storage directives.

    @anchor MessageOnError
    Message on Error
    ----------------

    Functions marked as \ref MessageOnError log a message when
    returning an error status or logging a warning (log levels 1 and
    2).  This indication can be useful when error and warning messages
    are retained in \ref log-registry.

    @{ */
    pub unsafe fn ms_rloginit(
        log_print: *mut LogCallback,
        logprefix: *const c_char,
        diag_print: *mut LogCallback,
        errorprefix: *const c_char,
        max_messages: c_int,
    );
    pub unsafe fn ms_rloginit_l(
        logp: *mut MSLogParam,
        log_print: *mut LogCallback,
        logprefix: *const c_char,
        diag_print: *mut LogCallback,
        errorprefix: *const c_char,
        max_messages: c_int,
    ) -> *mut MSLogParam;
    pub unsafe fn ms_rlog_enit(logp: *mut MSLogParam, count: c_int, context: c_int) -> c_int;
    pub unsafe fn ms_rlog_free(logp: *mut MSLogParam) -> c_int;

    // todo: handle global leap second list
    pub unsafe fn ms_readleapseconds(envvarname: *const c_char) -> c_int;
    pub unsafe fn ms_readleapsecondffile(filename: *const c_char) -> c_int;

    /* @addtogroup utility-functions
    @brief General utilities
    @{ */
    pub unsafe fn ms_samplesize(sampletype: c_char) -> u8;
    pub unsafe fn ms_encoding_sizetype(
        encoding: u8,
        samplesize: *const u8,
        sampletype: *mut c_char,
    ) -> c_int;
    pub unsafe fn ms_encodingstr(encoding: u8) -> *const c_char;
    pub unsafe fn ms_errorstr(errorcode: c_int) -> *const c_char;
    pub unsafe fn ms_sampletime(time: Nstime, offset: i64, samprate: c_double) -> Nstime;
    pub unsafe fn ms_bigendianhost() -> c_int;
    ///Portable version of POSIX ftello() to get file position in large files
    pub unsafe fn lmb_ftell64(stream: *mut FILE) -> i64;
    ///Portable version of POSIX fseeko() to set position in large files
    pub unsafe fn lmp_fsee64(stream: *mut FILE, offset: i64, whence: c_int) -> c_int;
    /// Portable version of POSIX nanosleep() to sleep for nanoseconds
    pub unsafe fn lmp_nanosleep(nanoseconds: u64) -> u64;
    /// Portable function to return the current system time
    pub unsafe fn lmp_systemtime() -> Nstime;
    /// Portable function for case-insensitive, ASCII-only string comparison
    pub unsafe fn lmp_strncasecmp(s1: *const c_char, s2: *const c_char, n: isize) -> c_int;
    ///Return CRC32C value of supplied buffer, with optional starting CRC32C value
    pub unsafe fn ms_crc32c(input: *const u8, length: c_int, previousCRC32C: u32) -> u32;
}
pub unsafe fn ms3_mstl_init_fd(fd: c_int) -> *mut MS3FileParam {
    unsafe { ms3_msfp_int_fd(fd) }
}
/** @def mseh_get
@brief A simple wrapper to access any type of extra header */
pub unsafe fn mseh_get(
    msr: *const MS3Record,
    ptr: *const c_char,
    value: *mut c_void,
    ty: c_char,
    max_length: u32,
) -> c_int {
    unsafe {
        mseh_get_ptr_r(
            msr,
            ptr,
            value,
            ty,
            max_length,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_get_uint64
@brief A simple wrapper to access an unsigned integer type extra header */
pub unsafe fn mseh_get_uint64(
    msr: *const MS3Record,
    ptr: *const c_char,
    value: *mut c_void,
) -> c_int {
    unsafe {
        mseh_get_ptr_r(
            msr,
            ptr,
            value,
            'u' as c_char,
            0,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_get_int64
@brief A simple wrapper to access an integer type extra header */
pub unsafe fn mseh_get_int64(
    msr: *const MS3Record,
    ptr: *const c_char,
    value: *mut c_void,
) -> c_int {
    unsafe {
        mseh_get_ptr_r(
            msr,
            ptr,
            value,
            'i' as c_char,
            0,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_get_number
@brief A simple wrapper to access a number type extra header */
pub unsafe fn mseh_get_number(
    msr: *const MS3Record,
    ptr: *const c_char,
    value: *mut c_void,
) -> c_int {
    unsafe {
        mseh_get_ptr_r(
            msr,
            ptr,
            value,
            'n' as c_char,
            0,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_get_string
@brief A simple wrapper to access a string type extra header */
pub unsafe fn mseh_get_string(
    msr: *const MS3Record,
    ptr: *const c_char,
    value: *mut c_void,
    max_length: u32,
) -> c_int {
    unsafe {
        mseh_get_ptr_r(
            msr,
            ptr,
            value,
            's' as c_char,
            max_length,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_get_boolean
@brief A simple wrapper to access a boolean type extra header */
pub unsafe fn mseh_get_boolean(
    msr: *const MS3Record,
    ptr: *const c_char,
    value: *mut c_void,
) -> c_int {
    unsafe {
        mseh_get_ptr_r(
            msr,
            ptr,
            value,
            'b' as c_char,
            0,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_get_boolean
@brief A simple wrapper to access a boolean type extra header */
pub unsafe fn mseh_exists(msr: *const MS3Record, ptr: *const c_char) -> c_int {
    unsafe {
        mseh_get_ptr_r(
            msr,
            ptr,
            NULL as *mut c_void,
            'b' as c_char,
            0,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_set
@brief A simple wrapper to set any type of extra header */
pub unsafe fn mseh_set(
    msr: *mut MS3Record,
    ptr: *const c_char,
    valueptr: *mut c_void,
    ty: c_char,
) -> c_int {
    unsafe { mseh_set_ptr_r(msr, ptr, valueptr, ty, NULL as *mut *mut LmParsedJson) }
}
/** @def mseh_set_uint64
@brief A simple wrapper to set an unsigned integer type extra header */
pub unsafe fn mseh_set_uint64(
    msr: *mut MS3Record,
    ptr: *const c_char,
    valueptr: *mut c_void,
) -> c_int {
    unsafe {
        mseh_set_ptr_r(
            msr,
            ptr,
            valueptr,
            'u' as c_char,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_set_int64
@brief A simple wrapper to set a number type extra header */
pub unsafe fn mseh_set_int64(
    msr: *mut MS3Record,
    ptr: *const c_char,
    valueptr: *mut c_void,
) -> c_int {
    unsafe {
        mseh_set_ptr_r(
            msr,
            ptr,
            valueptr,
            'i' as c_char,
            NULL as *mut *mut LmParsedJson,
        )
    }
}

/** @def mseh_set_number
@brief A simple wrapper to set a number type extra header */
pub unsafe fn mseh_set_number(
    msr: *mut MS3Record,
    ptr: *const c_char,
    valueptr: *mut c_void,
) -> c_int {
    unsafe {
        mseh_set_ptr_r(
            msr,
            ptr,
            valueptr,
            'n' as c_char,
            NULL as *mut *mut LmParsedJson,
        )
    }
}

/** @def mseh_set_string
@brief A simple wrapper to set a string type extra header */
pub unsafe fn mseh_set_string(
    msr: *mut MS3Record,
    ptr: *const c_char,
    valueptr: *mut c_void,
) -> c_int {
    unsafe {
        mseh_set_ptr_r(
            msr,
            ptr,
            valueptr,
            's' as c_char,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
/** @def mseh_set_boolean
@brief A simple wrapper to set a boolean type extra header */
pub unsafe fn mseh_set_boolean(
    msr: *mut MS3Record,
    ptr: *const c_char,
    valueptr: *mut c_void,
) -> c_int {
    unsafe {
        mseh_set_ptr_r(
            msr,
            ptr,
            valueptr,
            'b' as c_char,
            NULL as *mut *mut LmParsedJson,
        )
    }
}
pub unsafe fn ms_loginit(
    log_print: *mut LogCallback,
    logprefix: *const c_char,
    diag_print: *mut LogCallback,
    errorprefix: *const c_char,
) {
    unsafe { ms_rloginit(log_print, logprefix, diag_print, errorprefix, 0) }
}
pub unsafe fn ms_loginit_l(
    logp: *mut MSLogParam,
    log_print: *mut LogCallback,
    logprefix: *const c_char,
    diag_print: *mut LogCallback,
    errorprefix: *const c_char,
) -> *mut MSLogParam {
    unsafe { ms_rloginit_l(logp, log_print, logprefix, diag_print, errorprefix, 0) }
}
