use std::collections::HashMap;

use rayon::iter::ParallelIterator;
use walrus::{
    ir::{Instr, Value, VisitorMut},
    ActiveDataLocation, DataKind, InstrLocId,
};

pub fn embed_spec(module: &mut walrus::Module) -> anyhow::Result<()> {
    //
    module
        .producers
        .add_processed_by("mitex-spec", env!("CARGO_PKG_VERSION"));

    let functions = module
        .funcs
        .par_iter()
        .filter_map(|f| {
            f.name.as_ref().and_then(|name| {
                if name.contains("mitex_global_command_spec") {
                    Some(("mitex_global_command_spec", f.id()))
                } else if name.contains("_ZN8dlmalloc8dlmalloc17Dlmalloc")
                    && name.contains("6malloc")
                {
                    Some(("malloc", f.id()))
                } else if name == "memcpy" {
                    Some(("memcpy", f.id()))
                } else {
                    None
                }
            })
        })
        .collect::<HashMap<_, _>>();

    // find rodata
    let max_existing_data = module
        .data
        .iter()
        .filter_map(|data| {
            // println!("data: {:#?}", data.kind);

            match &data.kind {
                DataKind::Active(active) => match active.location {
                    ActiveDataLocation::Absolute(ptr) => Some(ptr as usize + data.value.len()),
                    ActiveDataLocation::Relative(_) => unimplemented!(),
                },
                DataKind::Passive => None,
            }
        })
        .max()
        .unwrap_or_default();

    let export_info = module
        .exports
        .iter()
        .filter_map(|export| {
            // println!("export: {:#?}", export.name);
            if export.name.contains("__data_end") {
                Some(("__data_end", export.id()))
            } else if export.name.contains("__heap_base") {
                Some(("__heap_base", export.id()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    let data_end = export_info
        .get("__data_end")
        .ok_or_else(|| anyhow::anyhow!("__data_end not found"))?;

    let heap_base = export_info
        .get("__heap_base")
        .ok_or_else(|| anyhow::anyhow!("__heap_base not found"))?;

    let get_global = |exp: &walrus::Export| -> walrus::GlobalId {
        match &exp.item {
            walrus::ExportItem::Global(g) => *g,
            _ => unimplemented!(),
        }
    };

    let data_end = get_global(module.exports.get(*data_end));
    let heap_base = get_global(module.exports.get(*heap_base));

    let get_init_value = |global: &walrus::Global| -> i32 {
        match &global.kind {
            walrus::GlobalKind::Local(walrus::InitExpr::Value(walrus::ir::Value::I32(v))) => *v,
            _ => unimplemented!(),
        }
    };

    let data_end = module.globals.get_mut(data_end);
    // println!("data_end: {:#?}", data_end);
    let data_end_val = get_init_value(data_end);
    // println!("data_end: {:#?}", data_end_val);

    let spec_memory_start = data_end_val + 40960;
    let spec_data = r#"This specification data is replaced by mitex after compilation"#
        .as_bytes()
        .to_vec();

    let spec_data_start = spec_memory_start;
    let spec_data_size = spec_data.len() as i32;

    if max_existing_data > data_end_val as usize {
        panic!("data_end is too small");
    }

    if module.memories.len() != 1 {
        panic!("invalid memory count");
    }
    let memory_id = module.memories.iter().next().unwrap().id();

    // allocate new data
    module.data.add(
        walrus::DataKind::Active(walrus::ActiveData {
            memory: memory_id,
            location: ActiveDataLocation::Absolute((spec_memory_start) as u32),
        }),
        spec_data,
    );

    let spec_mem_inc_end = 16 + (spec_memory_start - data_end_val) + spec_data_size + 1024;

    data_end.kind = walrus::GlobalKind::Local(walrus::InitExpr::Value(walrus::ir::Value::I32(
        data_end_val + spec_mem_inc_end,
    )));

    let heap_base = module.globals.get_mut(heap_base);
    // println!("heap_base: {:#?}", heap_base);

    let heap_base_val = get_init_value(heap_base);

    let hbv = heap_base_val + spec_mem_inc_end;
    heap_base.kind = walrus::GlobalKind::Local(walrus::InitExpr::Value(walrus::ir::Value::I32(
        hbv + ((hbv + 7) & !7),
    )));

    let data_embed_function = *functions
        .get("mitex_global_command_spec")
        .ok_or_else(|| anyhow::anyhow!("mitex_global_command_spec not found"))?;

    let malloc_fn = *functions
        .get("malloc")
        .ok_or_else(|| anyhow::anyhow!("malloc not found"))?;

    let memcpy_fn = *functions
        .get("memcpy")
        .ok_or_else(|| anyhow::anyhow!("memcpy not found"))?;

    module
        .funcs
        .par_iter_local_mut()
        .map(|(id, func)| {
            if id != data_embed_function {
                return Ok(());
            }

            type LocatedConst = (InstrLocId, Value);

            struct AllocAnalyzer {
                malloc_fn: walrus::FunctionId,
                memcpy_fn: walrus::FunctionId,

                malloc_size_location: Option<LocatedConst>,
                memcpy_data_location: Option<LocatedConst>,
                memcpy_size_location: Option<LocatedConst>,
                return_size_location: Option<LocatedConst>,

                prev1: Option<LocatedConst>,
                prev2: Option<LocatedConst>,
            }

            impl VisitorMut for AllocAnalyzer {
                fn visit_instr_mut(
                    &mut self,
                    instr: &mut walrus::ir::Instr,
                    instr_loc: &mut walrus::InstrLocId,
                ) {
                    match instr {
                        Instr::Const(cv) => {
                            self.prev2 = self.prev1;
                            self.prev1 = Some((*instr_loc, cv.value));
                        }
                        Instr::Store(_) => {
                            if let Some(memcpy_size) = self.memcpy_size_location {
                                if let Some(prev1) = self.prev1 {
                                    let Value::I32(cv1) = memcpy_size.1 else {
                                        return;
                                    };
                                    let Value::I32(cv2) = prev1.1 else {
                                        return;
                                    };
                                    if cv1 == cv2 {
                                        self.return_size_location = Some(prev1);
                                    }
                                }
                            }
                        }
                        Instr::Call(_) => {
                            if let walrus::ir::Instr::Call(call) = instr {
                                if call.func == self.malloc_fn {
                                    self.malloc_size_location = self.prev1;
                                } else if call.func == self.memcpy_fn {
                                    self.memcpy_data_location = self.prev2;
                                    self.memcpy_size_location = self.prev1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            let mut analyzer = AllocAnalyzer {
                malloc_fn,
                memcpy_fn,

                malloc_size_location: None,
                memcpy_data_location: None,
                memcpy_size_location: None,
                return_size_location: None,

                prev1: None,
                prev2: None,
            };

            let entry = func.entry_block();
            walrus::ir::dfs_pre_order_mut(&mut analyzer, func, entry);

            let Some(malloc_size) = analyzer.malloc_size_location else {
                return Err(anyhow::anyhow!("malloc_size not found"));
            };
            let Some(memcpy_data) = analyzer.memcpy_data_location else {
                return Err(anyhow::anyhow!("memcpy_data not found"));
            };
            let Some(memcpy_size) = analyzer.memcpy_size_location else {
                return Err(anyhow::anyhow!("memcpy_size not found"));
            };
            let Some(return_size) = analyzer.return_size_location else {
                return Err(anyhow::anyhow!("return_size not found"));
            };

            // validate pattern
            let malloc_size_value = match malloc_size.1 {
                walrus::ir::Value::I32(v) => v,
                _ => return Err(anyhow::anyhow!("invalid type of malloc_size_value")),
            };
            let memcpy_data_value = match memcpy_data.1 {
                walrus::ir::Value::I32(v) => v,
                _ => return Err(anyhow::anyhow!("invalid type of memcpy_data_value")),
            };
            let memcpy_size_value = match memcpy_size.1 {
                walrus::ir::Value::I32(v) => v,
                _ => return Err(anyhow::anyhow!("invalid type of memcpy_size_value")),
            };
            if malloc_size_value != memcpy_size_value + 8 {
                return Err(anyhow::anyhow!(
                    "malloc_size_value + 8 != memcpy_size_value"
                ));
            }
            // todo: verify dummy data
            let _ = memcpy_data_value;

            struct ConstEditor {
                edit_as: Vec<Option<(InstrLocId, i32)>>,
            }

            impl VisitorMut for ConstEditor {
                fn visit_instr_mut(
                    &mut self,
                    instr: &mut walrus::ir::Instr,
                    instr_loc: &mut walrus::InstrLocId,
                ) {
                    let Instr::Const(cv) = instr else {
                        return;
                    };

                    for e in self.edit_as.iter_mut() {
                        if let Some((loc, v)) = e {
                            if loc != instr_loc {
                                continue;
                            }

                            cv.value = walrus::ir::Value::I32(*v);

                            std::mem::take(e);
                        }
                    }
                }
            }

            let mut editor = ConstEditor {
                edit_as: [
                    (malloc_size.0, spec_data_size + 8),
                    (memcpy_data.0, spec_data_start),
                    (memcpy_size.0, spec_data_size),
                    (return_size.0, spec_data_size),
                ]
                .map(Some)
                .into_iter()
                .collect::<Vec<_>>(),
            };
            let entry = func.entry_block();
            walrus::ir::dfs_pre_order_mut(&mut editor, func, entry);

            if editor.edit_as.iter().any(Option::is_some) {
                return Err(anyhow::anyhow!("some values doesn't get edited"));
            }

            Ok(())
        })
        .collect::<anyhow::Result<_>>()?;

    Ok(())
}
