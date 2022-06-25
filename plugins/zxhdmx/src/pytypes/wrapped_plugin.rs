use std::collections::HashMap;

use pyvm::{
    builtins::{PyDict, PyDictRef, PyFunction, PyInt, PyIntRef, PyStr},
    function::{FuncArgs, KwArgs, PosArgs, IntoPyObject},
    pyclass, pyimpl, PyObjectRef, PyValue, PyRef, PyResult, VirtualMachine,
};
use rustpython_vm as pyvm;
use serde::{de::DeserializeOwned, Deserialize};

use crate::{config::ZxhdmxConfig, DataType};

#[pyclass(module = false, name = "WrappedConfig")]
#[derive(PyValue, Debug)]
pub struct WrappedConfig {
    config: ZxhdmxConfig,
}

impl WrappedConfig {
    pub fn new(config: ZxhdmxConfig) -> Self {
        Self { config }
    }
}
#[pyimpl]
impl WrappedConfig {
    #[allow(non_snake_case)]
    #[pyproperty]
    pub fn MIN_REQUIRED_PLAYERS(&self, vm: &VirtualMachine) -> PyResult<PyIntRef> {
        return Ok(PyInt::from(self.config.min_required_players).into_ref(vm));
    }
}
#[pyclass(module = false, name = "WrappedPlugin")]
#[derive(PyValue, Debug)]
pub struct WrappedPlugin {
    data: DataType,
    config_obj: PyRef<WrappedConfig>,
}

#[pyimpl]
impl WrappedPlugin {
    pub fn new(data: DataType, config: PyRef<WrappedConfig>) -> Self {
        Self {
            data,
            config_obj: config,
        }
    }
    // #[allow(non_snake_case)]
    #[pyproperty]
    pub fn config(&self) -> PyResult<PyRef<WrappedConfig>> {
        return Ok(self.config_obj.clone());
    }
    #[pymethod]
    pub fn load_data(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        let json_module = vm.import("json", None, 0)?;
        let _guard = tokio::runtime::Handle::current().enter();
        let value = futures::executor::block_on(self.data.read());

        let json_str = serde_json::to_string(&value.clone()).unwrap();
        drop(value);
        let pyobjref = PyStr::from(json_str).into_object(vm);
        // let mut pos_args = ;
        let py_dict_obj = json_module
            .get_attr("loads", vm)
            .expect("Bad python stdlib! no json.loads!")
            .downcast::<PyFunction>()
            .unwrap()
            .invoke(
                FuncArgs::new(PosArgs::new(vec![pyobjref]), KwArgs::default()),
                vm,
            )?;
        return Ok(py_dict_obj);
    }
    #[pymethod]
    pub fn get_items(&self, vm: &VirtualMachine) -> PyResult<PyDictRef> {
        return self.make_pydict::<HolderForItems>(vm);
    }
    #[pymethod]
    pub fn get_problem_set_list(&self, vm: &VirtualMachine) -> PyResult<PyDictRef> {
        return self.make_pydict::<HolderForProblemset>(vm);
    }
    fn make_pydict<T: DeserializeOwned + GetMyHashMap>(
        &self,
        vm: &VirtualMachine,
    ) -> PyResult<PyDictRef> {
        let items = {
            let _guard = tokio::runtime::Handle::current().enter();
            let value = futures::executor::block_on(self.data.read());
            serde_json::from_value::<T>(value.clone())
                .map_err(|e| vm.new_value_error(format!("Expected correct data format!\n{}", e)))?
        };
        let result = PyDict::default();
        for (k, v) in items.get_my_hashmap().iter() {
            result.get_or_insert(vm, PyStr::from(k.as_str()).into_pyobject(vm), || {
                PyStr::from(v.name.as_str()).into_pyobject(vm)
            })?;
        }
        return Ok(result.into_ref(vm));
    }
}
#[derive(Deserialize, Clone)]
struct GeneralItemWithNameAttribute {
    pub name: String,
}
#[derive(Deserialize)]
struct HolderForItems {
    pub items: HashMap<String, GeneralItemWithNameAttribute>,
}
#[derive(Deserialize)]
struct HolderForProblemset {
    pub problem_set: HashMap<String, GeneralItemWithNameAttribute>,
}
trait GetMyHashMap {
    fn get_my_hashmap(&self) -> &HashMap<String, GeneralItemWithNameAttribute>;
}
impl GetMyHashMap for HolderForItems {
    fn get_my_hashmap(&self) -> &HashMap<String, GeneralItemWithNameAttribute> {
        return &self.items;
    }
}
impl GetMyHashMap for HolderForProblemset {
    fn get_my_hashmap(&self) -> &HashMap<String, GeneralItemWithNameAttribute> {
        return &self.problem_set;
    }
}
