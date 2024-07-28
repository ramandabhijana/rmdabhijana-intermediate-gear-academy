use gstd::ActorId;
use gtest::{Program, ProgramBuilder, System};

pub const PROXY_PROGRAM: u64 = 1;
pub const TARGET_PROGRAM: u64 = 2;

pub const USER: u64 = 3;

pub struct ProgramPair<'a> {
    pub target_program: Program<'a>,
    pub proxy_program: Program<'a>,
}

pub fn init_system() -> System {
    let system = System::new();
    system.init_logger();
    system
}

pub fn init_programs<'a>(sys: &'a System) -> ProgramPair<'a> {
    let proxy_program = Program::current(sys);
    let target_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(TARGET_PROGRAM)
            .build(sys);

    let result = target_program.send_bytes(USER, []);
    assert!(!result.main_failed());

    let result = proxy_program.send::<_, ActorId>(USER, TARGET_PROGRAM.into());
    assert!(!result.main_failed());

    ProgramPair {
        target_program,
        proxy_program,
    }
}

// https://docs.rs/gstd/latest/gstd/#panic-handler-profiles
pub fn final_panic_message(message: &str) -> String {
    format!("Panic occurred: panicked with '{message}'")
}
