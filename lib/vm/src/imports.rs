// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

use crate::instance::{ImportInitializerFuncPtr, ImportInitializerThunks};
use crate::vmcontext::{VMFunctionImport, VMGlobalImport, VMMemoryImport, VMTableImport};
use wasmer_types::entity::{BoxedSlice, PrimaryMap};
use wasmer_types::{FunctionIndex, GlobalIndex, MemoryIndex, TableIndex};

/// Resolved import pointers.
#[derive(Clone)]
pub struct Imports {
    /// Resolved addresses for imported functions.
    pub functions: BoxedSlice<FunctionIndex, VMFunctionImport>,

    /// Initializers for host function environments. This is split out from `functions`
    /// because the generated code never needs to touch this and the extra wasted
    /// space may affect Wasm runtime performance due to increased cache pressure.
    ///
    /// We make it optional so that we can free the data after use.
    pub host_function_env_initializers: Option<
        BoxedSlice<FunctionIndex, (Option<ImportInitializerFuncPtr>, *mut std::ffi::c_void)>,
    >,

    /// Resolved addresses for imported tables.
    pub tables: BoxedSlice<TableIndex, VMTableImport>,

    /// Resolved addresses for imported memories.
    pub memories: BoxedSlice<MemoryIndex, VMMemoryImport>,

    /// Resolved addresses for imported globals.
    pub globals: BoxedSlice<GlobalIndex, VMGlobalImport>,
}

impl Imports {
    /// Construct a new `Imports` instance.
    pub fn new(
        function_imports: PrimaryMap<FunctionIndex, VMFunctionImport>,
        host_function_env_initializers: PrimaryMap<
            FunctionIndex,
            (Option<ImportInitializerFuncPtr>, *mut std::ffi::c_void),
        >,
        table_imports: PrimaryMap<TableIndex, VMTableImport>,
        memory_imports: PrimaryMap<MemoryIndex, VMMemoryImport>,
        global_imports: PrimaryMap<GlobalIndex, VMGlobalImport>,
    ) -> Self {
        Self {
            functions: function_imports.into_boxed_slice(),
            host_function_env_initializers: Some(host_function_env_initializers.into_boxed_slice()),
            tables: table_imports.into_boxed_slice(),
            memories: memory_imports.into_boxed_slice(),
            globals: global_imports.into_boxed_slice(),
        }
    }

    /// Construct a new `Imports` instance with no imports.
    pub fn none() -> Self {
        Self {
            functions: PrimaryMap::new().into_boxed_slice(),
            host_function_env_initializers: Some(PrimaryMap::new().into_boxed_slice()),
            tables: PrimaryMap::new().into_boxed_slice(),
            memories: PrimaryMap::new().into_boxed_slice(),
            globals: PrimaryMap::new().into_boxed_slice(),
        }
    }

    /// Get the `WasmerEnv::init_with_instance` function pointers and the pointers
    /// to the envs to call it on.
    ///
    /// This function can only be called once, it deletes the data it returns after
    /// returning it to ensure that it's not called more than once.
    pub fn get_import_initializers(&mut self) -> ImportInitializerThunks {
        let result = if let Some(inner) = &self.host_function_env_initializers {
            inner
                .values()
                .cloned()
                .map(|(func_init, env_ptr)| {
                    let host_env = if func_init.is_some() {
                        env_ptr
                    } else {
                        std::ptr::null_mut()
                    };
                    (func_init, host_env)
                })
                .collect()
        } else {
            vec![]
        };
        // ensure we only call these functions once and free this now useless memory.
        self.host_function_env_initializers = None;
        result
    }
}
