use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{BytecodeObject, BytecodeParts, Environment, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[path = "bytecode/compiler.rs"]
mod compiler;
#[path = "bytecode/control.rs"]
mod control;
#[path = "bytecode/guards.rs"]
mod guards;
#[path = "bytecode/inspection.rs"]
mod inspection;
#[path = "bytecode/machine.rs"]
mod machine;
#[path = "bytecode/promises.rs"]
mod promises;

fn owner(runtime: &RuntimeState) -> Result<Environment> {
    let Some(Sexp::Environment(owner)) = runtime.get(".GlobalEnv")? else {
        panic!("global")
    };
    Ok(owner)
}

fn bind(frontend: &Frontend, name: &str, value: &Sexp) -> Result {
    owner(frontend.runtime())?.insert(name.into(), value)?;
    Ok(())
}

fn compile(frontend: &mut Frontend, source: &str) -> Result<Sexp> {
    let function = frontend
        .eval_source(source)?
        .pop()
        .ok_or("closure result")?;
    frontend
        .runtime_mut()
        .compile_closure(&function)?
        .ok_or("unsupported closure".into())
}

fn body_code(function: &Sexp) -> &BytecodeObject {
    let Sexp::Closure { body, .. } = function else {
        panic!("closure")
    };
    let Sexp::Bytecode(code) = body.as_ref() else {
        panic!("genuine compiled body")
    };
    code
}

fn raw(runtime: &RuntimeState, code: Vec<i32>, constants: Vec<Sexp>) -> Result<Sexp> {
    Ok(Sexp::Bytecode(BytecodeObject::new(
        &owner(runtime)?,
        BytecodeParts {
            code,
            constants,
            expression: Sexp::Nil,
        },
    )?))
}
