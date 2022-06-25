use anyhow::anyhow;
use countdown_bot3::countdown_bot::client::ResultType;
use num_bigint::BigInt;
use pyvm::builtins::PyInt;
use pyvm::function::{FuncArgs, KwArgs, PosArgs, IntoPyObject};
use pyvm::PyObjectRef;
use rustpython_vm as pyvm;
use rustpython_vm::builtins::PyBaseExceptionRef;
pub fn bigint_to_i64(bigint: &BigInt) -> i64 {
    let (sgn, digits) = bigint.to_u64_digits();
    return *digits.get(0).unwrap_or(&0) as i64
        * match sgn {
            num_bigint::Sign::Minus => -1,
            num_bigint::Sign::NoSign => 0,
            num_bigint::Sign::Plus => 1,
        };
}
pub fn transform_pyerr(pyerr: PyBaseExceptionRef) -> anyhow::Error {
    return anyhow!(
        "发生错误: \n{:?}\n{:?}",
        pyerr.args().as_slice(),
        pyerr.traceback()
    );
}

pub fn check_player_in_game(
    inpr: &pyvm::Interpreter,
    game_inst: PyObjectRef,
    user_id: i64,
) -> ResultType<bool> {
    let pyintval = inpr
        .enter(|vm| {
            // pyvm::eval::eval(vm, source, scope, source_path)
            return Ok(vm
                .call_method(
                    &*game_inst.clone().get_attr("players", vm)?,
                    "__contains__",
                    FuncArgs::new(
                        PosArgs::new(vec![PyInt::from(user_id).into_pyobject(vm)]),
                        KwArgs::default(),
                    ),
                )?
                .downcast::<PyInt>()
                .unwrap());
            // return Ok(PyMethod::get(
            //     game_inst.clone().get_attr("players", vm)?,
            //     PyStr::from("__contains__").into_ref(vm),
            //     vm,
            // )?
            // .invoke(
            //     FuncArgs::new(
            //         PosArgs::new(vec![PyInt::from(user_id).into_pyobject(vm)]),
            //         KwArgs::default(),
            //     ),
            //     vm,
            // )?
            // .downcast::<PyInt>()
            // .unwrap());
        })
        .map_err(transform_pyerr)?;
    return Ok(bigint_to_i64(pyintval.as_bigint()) == 1);
}
