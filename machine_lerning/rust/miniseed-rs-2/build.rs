fn main() {
    if cfg!(target_os = "windows") {};
    cc::Build::new()
        .file("vendor/libmseed/crc32c.c")
        .file("vendor/libmseed/extraheaders.c")
        .file("vendor/libmseed/genutils.c")
        .file("vendor/libmseed/gmtime64.c")
        .file("vendor/libmseed/logging.c")
        .file("vendor/libmseed/lookup.c")
        .file("vendor/libmseed/msrutils.c")
        .file("vendor/libmseed/pack.c")
        .file("vendor/libmseed/packdata.c")
        .file("vendor/libmseed/parseutils.c")
        .file("vendor/libmseed/selection.c")
        .file("vendor/libmseed/tracelist.c")
        .file("vendor/libmseed/unpack.c")
        .file("vendor/libmseed/unpackdata.c")
        .file("vendor/libmseed/yyjson.c")
        .compile("libmseed");
}
