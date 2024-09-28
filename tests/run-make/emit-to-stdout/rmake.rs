// If `-o -` or `--emit KIND=-` is provided, output should be written
// to stdout instead. Binary output (`obj`, `llvm-bc`, `link` and
// `metadata`) being written this way will result in an error unless
// stdout is not a tty. Multiple output types going to stdout will
// trigger an error too, as they will all be mixed together.
// See https://github.com/rust-lang/rust/pull/111626

use std::io::IsTerminal;

use run_make_support::{diff, rfs, rustc};

fn main() {
    rfs::create_dir("out");

    #[cfg(not(windows))]
    let stdout = std::fs::File::options().write(true).open("/dev/ptmx").unwrap();
    // FIXME: If this test fails and the compiler does print to the console, then this will produce a lot of output.
    // We should spawn a new console instead to print stdout.
    #[cfg(windows)]
    let stdout = std::fs::File::options().read(true).write(true).open(r"\\.\CONOUT$").unwrap();

    assert!(stdout.is_terminal());
    test_asm();
    test_llvm_ir();
    test_dep_info();
    test_mir();
    test_llvm_bc(&stdout);
    test_obj(&stdout);
    test_metadata(&stdout);
    test_link(&stdout);
    test_multiple_types();
    test_multiple_types_option_o();
}

fn test_asm() {
    rustc().emit("asm=out/asm").input("test.rs").run();
    let emit = rustc().emit("asm=-").input("test.rs").run().stdout_utf8();
    diff().expected_file("out/asm").actual_text("actual", &emit).run();
}

fn test_llvm_ir() {
    rustc().emit("llvm-ir=out/llvm-ir").input("test.rs").run();
    let emit = rustc().emit("llvm-ir=-").input("test.rs").run().stdout_utf8();
    diff().expected_file("out/llvm-ir").actual_text("actual", &emit).run();
}

fn test_dep_info() {
    rustc()
        .emit("dep-info=out/dep-info")
        .input("test.rs")
        .arg("-Zdep-info-omit-d-target=yes")
        .run();
    let emit = rustc().emit("dep-info=-").input("test.rs").run().stdout_utf8();
    diff().expected_file("out/dep-info").actual_text("actual", &emit).run();
}

fn test_mir() {
    rustc().emit("mir=out/mir").input("test.rs").run();
    let emit = rustc().emit("mir=-").input("test.rs").run().stdout_utf8();
    diff().expected_file("out/mir").actual_text("actual", &emit).run();
}

// FIXME: ptmx
fn test_llvm_bc(stdout: &std::fs::File) {
    let emit = rustc()
        .emit("llvm-bc=-")
        .stdout(stdout.try_clone().unwrap())
        .input("test.rs")
        .run_fail()
        .stderr_utf8();
    diff().expected_file("emit-llvm-bc.stderr").actual_text("actual", &emit).run();
}

// FIXME: ptmx
fn test_obj(stdout: &std::fs::File) {
    let emit = rustc()
        .emit("obj=-")
        .stdout(stdout.try_clone().unwrap())
        .input("test.rs")
        .run_fail()
        .stderr_utf8();
    diff().expected_file("emit-obj.stderr").actual_text("actual", &emit).run();
}

// FIXME: ptmx
fn test_metadata(stdout: &std::fs::File) {
    let emit = rustc()
        .emit("metadata=-")
        .input("test.rs")
        .stdout(stdout.try_clone().unwrap())
        .run_fail()
        .stderr_utf8();
    diff().expected_file("emit-metadata.stderr").actual_text("actual", &emit).run();
}

// FIXME: ptmx
fn test_link(stdout: &std::fs::File) {
    let emit = rustc()
        .emit("link=-")
        .input("test.rs")
        .stdout(stdout.try_clone().unwrap())
        .run_fail()
        .stderr_utf8();
    diff().expected_file("emit-link.stderr").actual_text("actual", &emit).run();
}

fn test_multiple_types() {
    diff()
        .expected_file("emit-multiple-types.stderr")
        .actual_text(
            "actual",
            rustc()
                .output("-")
                .emit("asm=-")
                .emit("llvm-ir=-")
                .emit("dep-info=-")
                .emit("mir=-")
                .input("test.rs")
                .run_fail()
                .stderr_utf8(),
        )
        .run();
}

fn test_multiple_types_option_o() {
    diff()
        .expected_file("emit-multiple-types.stderr")
        .actual_text(
            "actual",
            rustc()
                .output("-")
                .emit("asm,llvm-ir,dep-info,mir")
                .input("test.rs")
                .run_fail()
                .stderr_utf8(),
        )
        .run();
}
