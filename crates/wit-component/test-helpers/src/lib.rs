use wit_parser::abi::{AbiVariant, WasmType};
use wit_parser::{Document, Function};

pub fn dummy_module(doc: &Document) -> Vec<u8> {
    let world = doc.default_world().unwrap();
    let world = &doc.worlds[world];
    let mut wat = String::new();
    wat.push_str("(module\n");
    for (name, import) in world.imports.iter() {
        for func in doc.interfaces[*import].functions.iter() {
            let sig = doc.wasm_signature(AbiVariant::GuestImport, func);

            wat.push_str(&format!("(import \"{name}\" \"{}\" (func", func.name));
            push_tys(&mut wat, "param", &sig.params);
            push_tys(&mut wat, "result", &sig.results);
            wat.push_str("))\n");
        }
    }

    for (name, export) in world.exports.iter() {
        for func in doc.interfaces[*export].functions.iter() {
            let name = func.core_export_name(Some(name));
            push_func(&mut wat, &name, doc, func);
        }
    }

    if let Some(default) = world.default {
        for func in doc.interfaces[default].functions.iter() {
            push_func(&mut wat, &func.name, doc, func);
        }
    }

    wat.push_str("(memory (export \"memory\") 0)\n");
    wat.push_str(
        "(func (export \"cabi_realloc\") (param i32 i32 i32 i32) (result i32) unreachable)\n",
    );
    wat.push_str(")\n");

    return wat::parse_str(&wat).unwrap();

    fn push_func(wat: &mut String, name: &str, doc: &Document, func: &Function) {
        let sig = doc.wasm_signature(AbiVariant::GuestExport, func);
        wat.push_str(&format!("(func (export \"{name}\")"));
        push_tys(wat, "param", &sig.params);
        push_tys(wat, "result", &sig.results);
        wat.push_str(" unreachable)\n");

        if doc.guest_export_needs_post_return(func) {
            wat.push_str(&format!("(func (export \"cabi_post_{name}\")"));
            push_tys(wat, "param", &sig.results);
            wat.push_str(")\n");
        }
    }

    fn push_tys(dst: &mut String, desc: &str, params: &[WasmType]) {
        if params.is_empty() {
            return;
        }
        dst.push_str(" (");
        dst.push_str(desc);
        for ty in params {
            dst.push_str(" ");
            match ty {
                WasmType::I32 => dst.push_str("i32"),
                WasmType::I64 => dst.push_str("i64"),
                WasmType::F32 => dst.push_str("f32"),
                WasmType::F64 => dst.push_str("f64"),
            }
        }
        dst.push_str(")");
    }
}
